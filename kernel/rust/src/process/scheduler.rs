use alloc::collections::VecDeque;
use alloc::boxed::Box;
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

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        unsafe {
            ffi::serial_print(b"[Scheduler] Initializing round-robin scheduler\n\0".as_ptr());
        }
        
        Scheduler {
            ready_queue: VecDeque::new(),
            current_task: None,
        }
    }
    
    /// Initialize the global scheduler
    pub fn init() {
        let scheduler = Self::new();
        *SCHEDULER.lock() = Some(scheduler);
    }
    
    /// Add a task to the ready queue
    pub fn add_task(task: Box<Task>) {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            unsafe {
                ffi::serial_print(b"[Scheduler] Adding task to ready queue\n\0".as_ptr());
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
            let mut old_box_opt = old_opt; // may be None for first run

            unsafe {
                ffi::serial_print(b"[Scheduler] Preparing context switch\n\0".as_ptr());
            }

            // Drop lock before performing context switch
            drop(scheduler_lock);

            // If there is an old context, perform context switch
            if let Some(mut old_box) = old_box_opt {
                // Get pointer to old context
                let old_ctx_ptr: *mut crate::process::task::CpuContext = old_box.context_mut() as *mut _;

                unsafe {
                    ffi::serial_print(b"[Scheduler] Calling context_switch\n\0".as_ptr());
                    // This will save registers into old_ctx and restore new_ctx, jumping to new task.
                    ffi::context_switch(old_ctx_ptr, new_ctx_ptr);
                    // When we return here, we are back in the old context.
                    ffi::serial_print(b"[Scheduler] Returned from context_switch (old context)\n\0".as_ptr());
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
                            unsafe { ffi::serial_print(b"[Scheduler] Old task terminated after switch\n\0".as_ptr()); }
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
                unsafe { ffi::serial_print(b"[Scheduler] No old task, initial run\n\0".as_ptr()); }
            }
        }
    }
    
    /// Yield CPU to another task (for cooperative multitasking)
    pub fn yield_cpu() {
        unsafe {
            ffi::serial_print(b"[Scheduler] Task yielding CPU\n\0".as_ptr());
        }

        // Use schedule() which performs proper context switching between tasks
        Self::schedule();
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
    pub extern "C" fn rust_handle_page_fault(addr: u32, err: u32) {
        unsafe {
            crate::ffi::serial_print(b"[Scheduler] rust_handle_page_fault invoked\n\0".as_ptr());
        }

        // Mark current task as terminated
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref mut task) = sched.current_task {
                task.set_state(TaskState::Terminated);
                unsafe { crate::ffi::serial_print(b"[Scheduler] Marked current task as Terminated\n\0".as_ptr()); }
            }
        }

        // Schedule next task
        Self::schedule();
    }
    
    /// Start the scheduler (never returns)
    pub fn start() -> ! {
        unsafe {
            ffi::serial_print(b"[Scheduler] Starting scheduler with \0".as_ptr());
            ffi::vga_println(b"\nStarting multitasking demo...\n\0".as_ptr());
        }
        
        // Get queue size
        let queue_size = {
            let scheduler = SCHEDULER.lock();
            if let Some(ref sched) = *scheduler {
                sched.ready_queue.len()
            } else {
                0
            }
        };
        
        unsafe {
            ffi::serial_print(b" tasks in queue\n\0".as_ptr());
        }
        
        if queue_size == 0 {
            unsafe {
                ffi::serial_print(b"[Scheduler] ERROR: No tasks to run!\n\0".as_ptr());
                ffi::vga_set_color(12, 0); // Red
                ffi::vga_println(b"ERROR: No tasks in scheduler!\0".as_ptr());
            }
            loop {
                unsafe { core::arch::asm!("hlt"); }
            }
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
                    ffi::serial_print(b"[Scheduler] Performing initial context_switch to first task\n\0".as_ptr());
                    ffi::context_switch(&mut kernel_ctx as *mut _, new_ctx_ptr);
                    // When we return here, the task has finished or yielded back to kernel_ctx
                    ffi::serial_print(b"[Scheduler] Returned from initial context_switch\n\0".as_ptr());
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
            ffi::serial_print(b"[Scheduler] ERROR: Scheduler returned!\n\0".as_ptr());
        }
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
}
