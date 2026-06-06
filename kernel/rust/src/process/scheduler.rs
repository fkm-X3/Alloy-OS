use alloc::collections::VecDeque;
use alloc::boxed::Box;
use alloc::string::String;
use crate::process::task::{Task, TaskState};
use crate::sync::SpinLock;
use crate::ffi;

/// Global scheduler instance
static SCHEDULER: SpinLock<Option<Scheduler>> = SpinLock::new(None);

/// Round-robin scheduler
pub struct Scheduler {
    ready_queue: VecDeque<Box<Task>>,
    current_task: Option<Box<Task>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        unsafe {
            ffi::serial_print(c"[Scheduler] Initializing round-robin scheduler\n".as_ptr() as *const u8);
        }
        
        Scheduler {
            ready_queue: VecDeque::new(),
            current_task: None,
        }
    }
    
    /// Initialize the global scheduler with an idle task
    pub fn init() {
        let mut scheduler = Self::new();
        let idle = Box::new(Task::new_idle());
        scheduler.ready_queue.push_back(idle);
        *SCHEDULER.lock() = Some(scheduler);
    }
    
    /// Add a task to the ready queue
    pub fn add_task(task: Box<Task>) {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            unsafe {
                ffi::serial_print(c"[Scheduler] Adding task to ready queue\n".as_ptr() as *const u8);
            }
            sched.ready_queue.push_back(task);
        }
    }
    
    /// Get the next task to run (round-robin)
    fn pick_next(&mut self) -> Option<Box<Task>> {
        if let Some(mut task) = self.ready_queue.pop_front() {
            task.set_state(TaskState::Running);
            Some(task)
        } else {
            None
        }
    }
    
    /// Schedule next task (round-robin)
    pub fn schedule() {
        // Attempt to perform a proper context switch between current and next task.
        // Strategy:
        // 1. Take current task out of scheduler (old_opt).
        // 2. Pick next task from ready queue (next_opt).
        // 3. If next exists, set it as current in scheduler and obtain raw context pointers.
        // 4. Drop scheduler lock and call context_switch(old_ctx, new_ctx) if old existed.
        // 5. After context_switch returns (we are back in old context), re-acquire lock and push old back if runnable.

        let mut scheduler_lock = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler_lock {
            // Take current task out
            let old_opt: Option<Box<Task>> = sched.current_task.take();

            // Pick next task
            let next_opt: Option<Box<Task>> = sched.pick_next();
            if next_opt.is_none() {
                // No next task - put old back if present
                if let Some(old) = old_opt {
                    sched.current_task = Some(old);
                }
                return;
            }

            // We have a next task
            let mut next = next_opt.unwrap();
            next.set_state(TaskState::Running);

            // Obtain raw pointer to new context before placing into scheduler
            let new_ctx_ptr: *mut crate::process::task::CpuContext = next.context_mut() as *mut _;

            // Place next into scheduler as current
            sched.current_task = Some(next);

            // Prepare old boxed task to be used for context switching
            let old_box_opt = old_opt; // may be None for first run

            unsafe {
                ffi::serial_print(c"[Scheduler] Preparing context switch\n".as_ptr() as *const u8);
            }

            // Drop lock before performing context switch
            drop(scheduler_lock);

            // If there is an old context, perform context switch
            if let Some(mut old_box) = old_box_opt {
                // Get pointer to old context
                let old_ctx_ptr: *mut crate::process::task::CpuContext = old_box.context_mut() as *mut _;

                unsafe {
                    ffi::serial_print(c"[Scheduler] Calling context_switch\n".as_ptr() as *const u8);
                    // This will save registers into old_ctx and restore new_ctx, jumping to new task.
                    ffi::context_switch(old_ctx_ptr, new_ctx_ptr);
                    // When we return here, we are back in the old context.
                    ffi::serial_print(c"[Scheduler] Returned from context_switch (old context)\n".as_ptr() as *const u8);
                }

                // After returning, re-acquire scheduler lock and push old task back if runnable
                let mut scheduler_lock = SCHEDULER.lock();
                if let Some(ref mut sched) = *scheduler_lock {
                    match old_box.state() {
                        TaskState::Running => {
                            old_box.set_state(TaskState::Ready);
                            sched.ready_queue.push_back(old_box);
                        }
                        TaskState::Terminated => {
                            unsafe { ffi::serial_print(c"[Scheduler] Old task terminated after switch\n".as_ptr() as *const u8); }
                            drop(old_box);
                        }
                        _ => {
                            old_box.set_state(TaskState::Ready);
                            sched.ready_queue.push_back(old_box);
                        }
                    }
                }
            } else {
                // No old context: this is the initial switch into first task. Simply return and let the new task execute.
                // Control flow: caller should arrange to jump into the new task. For simplicity, do nothing here.
                unsafe { ffi::serial_print(c"[Scheduler] No old task, initial run\n".as_ptr() as *const u8); }
            }
        }
    }
    
    /// Yield CPU to another task (for cooperative multitasking)
    pub fn yield_cpu() {
        unsafe {
            ffi::serial_print(c"[Scheduler] Task yielding CPU\n".as_ptr() as *const u8);
        }

        // Use schedule() which performs proper context switching between tasks
        Self::schedule();
    }

    /// Clone — create a new task running `entry(arg)` with given stack.
    /// If the current task has its own page directory, it is cloned.
    pub fn clone_task(entry: u32, stack: u32, arg: u32) -> u32 {
        let mut scheduler = SCHEDULER.lock();
        let sched = match scheduler.as_mut() {
            Some(s) => s,
            None => return u32::MAX,
        };
        // Build a context for the new task
        let mut ctx = Box::new(crate::process::task::CpuContext::new());
        ctx.eip = entry;
        ctx.esp = stack;
        ctx.ebp = stack;
        // Pass arg in EAX (child sees it as return value)
        ctx.eax = arg;
        // Set user-mode segments
        ctx.cs = 0x1B;
        ctx.ds = 0x23;
        ctx.es = 0x23;
        ctx.fs = 0x23;
        ctx.gs = 0x23;
        ctx.ss = 0x23;

        // Clone page directory if current has its own
        let kernel_pd = unsafe { ffi::paging_get_kernel_directory_phys() };
        let current_cr3 = sched.current_task.as_ref()
            .map(|t| t.context().cr3)
            .unwrap_or(kernel_pd);
        ctx.cr3 = if current_cr3 != kernel_pd {
            let new_pd = unsafe { ffi::paging_clone_directory(current_cr3) };
            if new_pd == 0 { return u32::MAX; }
            new_pd
        } else {
            kernel_pd
        };

        let child = Box::new(Task::from_parts(
            ctx,
            Some(Box::new([0u8; 4096])),
            String::from("clone"),
            [None; 32],
            0x01000000,
        ));

        let pid = child.id().as_u32();
        sched.ready_queue.push_back(child);
        pid
    }

    /// Convenience helper to operate on the current task under the scheduler lock.
    /// The closure receives a mutable reference to the current Task if present.
    pub fn with_current_task_mut<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&mut Task) -> R,
    {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref mut task) = sched.current_task {
                return Some(f(task));
            }
        }
        None
    }

    /// External hook for page fault handling from C++
    #[no_mangle]
    pub extern "C" fn rust_handle_page_fault(_addr: u32, _err: u32) {
        unsafe {
            crate::ffi::serial_print(c"[Scheduler] rust_handle_page_fault invoked\n".as_ptr() as *const u8);
        }

        // Mark current task as terminated
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref mut task) = sched.current_task {
                task.set_state(TaskState::Terminated);
                unsafe { crate::ffi::serial_print(c"[Scheduler] Marked current task as Terminated\n".as_ptr() as *const u8); }
            }
        }

        // Schedule next task
        Self::schedule();
    }
    
    /// Start the scheduler (never returns)
    pub fn start() -> ! {
        unsafe {
            ffi::serial_print(c"[Scheduler] Starting scheduler\n".as_ptr() as *const u8);
            ffi::vga_println(c"\nStarting multitasking...\n".as_ptr() as *const u8);
        }
        
        // Schedule and prepare first task
        Self::schedule();

        // For the initial run, perform a context switch from a kernel-stored context into the first task
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref mut task) = sched.current_task {
                // Prepare a local kernel context to save current CPU state
                let mut kernel_ctx = crate::process::task::CpuContext::new();

                // Get pointer to the new task context
                let new_ctx_ptr: *mut crate::process::task::CpuContext = task.context_mut() as *mut _;

                // Drop scheduler lock before switching
                drop(scheduler);

                unsafe {
                    ffi::serial_print(c"[Scheduler] Performing initial context_switch to first task\n".as_ptr() as *const u8);
                    ffi::context_switch(&mut kernel_ctx as *mut _, new_ctx_ptr);
                    // When we return here, the task has finished or yielded back to kernel_ctx
                    ffi::serial_print(c"[Scheduler] Returned from initial context_switch\n".as_ptr() as *const u8);
                }

                // Re-acquire scheduler and continue scheduling loop
                let mut scheduler = SCHEDULER.lock();
                if let Some(ref mut sched) = *scheduler {
                    // If kernel_ctx indicates the task returned, treat it as terminated
                    // For simplicity, mark current task terminated and schedule next
                    if let Some(mut current) = sched.current_task.take() {
                        current.set_state(TaskState::Terminated);
                        drop(current);
                    }
                    Self::schedule();
                }
            }
        }

        // Should never reach here; if it does, halt
        unsafe {
            ffi::serial_print(c"[Scheduler] ERROR: Scheduler returned!\n".as_ptr() as *const u8);
        }
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
}
