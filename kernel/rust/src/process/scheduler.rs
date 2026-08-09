use alloc::collections::VecDeque;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU32, Ordering};
use crate::process::task::{Task, TaskState};
use crate::process::WaitQueue;
use crate::sync::SpinlockIRQ;
use crate::ffi;

/// Global scheduler instance — must use SpinlockIRQ because timer interrupts
/// call rust_timer_tick() which acquires this lock from interrupt context.
static SCHEDULER: SpinlockIRQ<Option<Scheduler>> = SpinlockIRQ::new(None);

const NUM_PRIORITIES: usize = 4;
const QUANTA: [u32; NUM_PRIORITIES] = [5, 10, 20, 40];
const BOOST_INTERVAL: u64 = 100;

/// Depth counter for save_context resume detection.
/// Incremented before save_context, checked after.
/// Zeroed after each complete schedule cycle.
static SCHEDULE_DEPTH: AtomicU32 = AtomicU32::new(0);

/// Wait queue for keyboard input — tasks block here waiting for keypresses.
pub static KEYBOARD_WAIT: WaitQueue = WaitQueue::new();

/// Wait queue for mouse input — tasks block here waiting for mouse events.
pub static MOUSE_WAIT: WaitQueue = WaitQueue::new();

/// Wait queue for child process exit — parents block here waiting for children.
pub static CHILD_WAIT: WaitQueue = WaitQueue::new();

/// Multi-level feedback queue scheduler
pub struct Scheduler {
    ready_queues: Vec<VecDeque<Box<Task>>>,
    current_task: Option<Box<Task>>,
    /// Maps parent_id -> Vec<(child_pid, exit_code)> for terminated children
    children_exit_status: BTreeMap<u32, Vec<(u32, u32)>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        // Initializing MLFQ scheduler

        let mut ready_queues = Vec::with_capacity(NUM_PRIORITIES);
        for _ in 0..NUM_PRIORITIES {
            ready_queues.push(VecDeque::new());
        }

        Scheduler {
            ready_queues,
            current_task: None,
            children_exit_status: BTreeMap::new(),
        }
    }

    pub fn init() {
        let mut scheduler = Self::new();
        let idle = Box::new(Task::new_idle());
        let prio = idle.priority() as usize;
        scheduler.ready_queues[prio].push_back(idle);
        *SCHEDULER.lock() = Some(scheduler);
    }

    pub fn add_task(task: Box<Task>) {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            sched.ready_queues[0].push_back(task);
        }
    }

    fn pick_next(&mut self) -> Option<Box<Task>> {
        for level in 0..NUM_PRIORITIES {
            if let Some(mut task) = self.ready_queues[level].pop_front() {
                task.set_state(TaskState::Running);
                return Some(task);
            }
        }
        None
    }

    pub fn schedule() {
        let mut scheduler_lock = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler_lock {
            let old_opt: Option<Box<Task>> = sched.current_task.take();

            let next_opt: Option<Box<Task>> = sched.pick_next();
            if next_opt.is_none() {
                if let Some(old) = old_opt {
                    sched.current_task = Some(old);
                }
                return;
            }

            let mut next = next_opt.unwrap();
            next.set_state(TaskState::Running);

            let new_ctx_ptr: *mut crate::process::task::CpuContext = next.context_mut() as *mut _;

            sched.current_task = Some(next);

            let old_box_opt = old_opt;

            // Release scheduler lock (IRQs stay disabled — lock() did cli)
            scheduler_lock.release_no_irq_restore();

            // Depth counter: incremented before save_context, checked after.
            // On the initial path, depth+1 == SCHEDULE_DEPTH because we just
            // incremented it.  On the resume path (via load_context), the
            // saved context has the old depth, but SCHEDULE_DEPTH was reset
            // to 0 by the initial path, so the check fails.
            let depth = SCHEDULE_DEPTH.fetch_add(1, Ordering::Relaxed);

            match old_box_opt {
                None => {
                    // No old task to save — just load the new one
                    SCHEDULE_DEPTH.store(0, Ordering::Relaxed);
                    unsafe { ffi::load_context(new_ctx_ptr); }
                }
                Some(mut old_box) => {
                    let old_ctx_ptr = old_box.context_mut() as *mut _;

                    // Step 1: Save old context — returns normally
                    unsafe { ffi::save_context(old_ctx_ptr); }

                    // Step 2: Check if this is the initial path or a resume
                    let current_depth = SCHEDULE_DEPTH.load(Ordering::Relaxed);

                    if current_depth == depth + 1 {
                        // ── Initial path ─────────────────────────────────
                        // The SCHEDULER lock was released above, but IRQs
                        // remain disabled.  Re-acquire long enough to push
                        // the old task back into the ready queue.
                        let mut re_lock = SCHEDULER.lock();
                        if let Some(ref mut re_sched) = *re_lock {
                            if old_box.state() == TaskState::Terminated {
                                // A terminated task must never be re-queued:
                                // it would resume after sys_exit and keep
                                // running (in aarch64's shared address space
                                // it could even corrupt the kernel heap).
                                // Leak the Box instead of dropping it: we are
                                // currently executing on this very task's
                                // kernel stack, and the heap lock may be held
                                // by a preempted task — both make a free()
                                // here unsafe.
                                core::mem::forget(old_box);
                            } else {
                                let had_full_quantum =
                                    old_box.ticks_used() >= QUANTA[old_box.priority() as usize];
                                if had_full_quantum {
                                    let new_p =
                                        (old_box.priority() + 1).min(NUM_PRIORITIES as u8 - 1);
                                    old_box.set_priority(new_p);
                                } else {
                                    let new_p = old_box.priority().saturating_sub(1);
                                    old_box.set_priority(new_p);
                                }
                                old_box.reset_ticks_used();
                                old_box.set_state(TaskState::Ready);
                                re_sched.ready_queues[old_box.priority() as usize].push_back(old_box);
                            }
                        }
                        // Release re_lock explicitly BEFORE load_context.
                        // load_context is `-> !` (aarch64 erets / x86_64 jmp),
                        // so a guard Drop placed after it is unreachable and
                        // eliminated by the compiler — the lock would leak.
                        // IRQs stay disabled (original RFLAGS had IF=0 from
                        // the outer lock() that saved IF=0 after fetch_add).
                        re_lock.release_no_irq_restore();

                        // Reset depth and load new context — never returns
                        SCHEDULE_DEPTH.store(0, Ordering::Relaxed);
                        unsafe { ffi::load_context(new_ctx_ptr); }
                    } else {
                        // ── Resume path ─────────────────────────────────
                        // Task was already re-enqueued on the initial path.
                        // old_box is a stale pointer — forget it to prevent
                        // double-free (the real Box is in the ready queue).
                        SCHEDULE_DEPTH.store(0, Ordering::Relaxed);
                        core::mem::forget(old_box);
                        // Re-enable interrupts before returning so that the
                        // IRQ handler (which will iretq back to the task)
                        // runs with correct IF state.
                        crate::sync::irq_enable();
                    }
                }
            }
        }
    }

    pub fn yield_cpu() {
        Self::schedule();
    }

    /// Block the current task on a wait queue.
    pub fn block_current_on(wait_queue: &WaitQueue) {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(mut task) = sched.current_task.take() {
                task.set_state(TaskState::Blocked);
                task.reset_ticks_used();
                wait_queue.enqueue(task);
            }
        }
        drop(scheduler);
        Self::schedule();
    }

    /// Wake up to `count` tasks from a wait queue, putting them into the
    /// highest-priority ready queue.
    pub fn wake_waiters(wait_queue: &WaitQueue, count: usize) {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            for _ in 0..count {
                if let Some(mut task) = wait_queue.dequeue() {
                    task.set_state(TaskState::Ready);
                    sched.ready_queues[0].push_back(task);
                } else {
                    break;
                }
            }
        }
    }

    pub fn clone_task(entry: u32, stack: u32, arg: u32) -> u32 {
        let mut scheduler = SCHEDULER.lock();
        let sched = match scheduler.as_mut() {
            Some(s) => s,
            None => return u32::MAX,
        };
        let mut ctx = Box::new(crate::process::task::CpuContext::new());
        let kernel_pd = unsafe { ffi::paging_get_kernel_directory_phys() };

        #[cfg(feature = "x86_64")]
        {
            ctx.rip = entry as u64;
            ctx.rsp = stack as u64;
            ctx.rbp = stack as u64;
            ctx.rax = arg as u64;
            ctx.cs = 0x23;
            ctx.ds = 0x1B;
            ctx.es = 0x1B;
            ctx.fs = 0x1B;
            ctx.gs = 0x1B;
            ctx.ss = 0x1B;

            let current_cr3: usize = sched.current_task.as_ref()
                .map(|t| t.context().cr3 as usize)
                .unwrap_or(kernel_pd);
            ctx.cr3 = if current_cr3 != kernel_pd {
                let new_pd = unsafe { ffi::paging_clone_directory(current_cr3) };
                if new_pd == 0 { return u32::MAX; }
                new_pd as u64
            } else {
                kernel_pd as u64
            };
        }

        let child = Box::new(Task::from_parts(
            ctx,
            Some(Box::new([0u8; crate::process::task::KERNEL_STACK_SIZE])),
            String::from("clone"),
            [None; 32],
            0x01000000,
            None,
        ));

        let pid = child.id().as_u32();
        sched.ready_queues[0].push_back(child);
        pid
    }

    /// Fork the current task — creates a child with COW-shared address space,
    /// inherited fd table, and proper parent-child tracking.
    pub fn fork_current() -> u32 {
        let mut scheduler = SCHEDULER.lock();
        let sched = match scheduler.as_mut() {
            Some(s) => s,
            None => return u32::MAX,
        };

        let parent_task = match sched.current_task.as_ref() {
            Some(t) => t,
            None => return u32::MAX,
        };

        let parent_ctx = parent_task.context();
        let kernel_pd = unsafe { ffi::paging_get_kernel_directory_phys() };

        // Clone the CPU context for the child
        let mut child_ctx = Box::new(*parent_ctx);

        #[cfg(feature = "x86_64")]
        {
            // Child fork returns 0
            child_ctx.rax = 0;

            // Use COW-based address space cloning
            if parent_ctx.cr3 as usize != kernel_pd {
                let child_pd = unsafe { ffi::paging_fork_directory(parent_ctx.cr3 as usize) };
                if child_pd == 0 { return u32::MAX; }
                child_ctx.cr3 = child_pd as u64;
            }
        }

        // Inherit fd table
        let child_fds = parent_task.clone_fds();

        // Inherit heap break
        let child_heap = parent_task.heap_break();

        // Set parent_id
        let parent_id = parent_task.id();

        let child = Box::new(Task::from_parts(
            child_ctx,
            None,
            String::from("fork"),
            child_fds,
            child_heap,
            Some(parent_id),
        ));

        let child_pid = child.id().as_u32();
        sched.ready_queues[0].push_back(child);
        child_pid
    }

    /// Mark the current task as terminated and notify its parent (if any).
    pub fn terminate_current(exit_code: u32) {
        let mut scheduler = SCHEDULER.lock();
        let sched = match scheduler.as_mut() {
            Some(s) => s,
            None => return,
        };

        // Extract info from current task before any mutable access
        let (pid, parent) = if let Some(ref task) = sched.current_task {
            (task.id().as_u32(), task.parent_id())
        } else {
            return;
        };

        // Now mark terminated (separate step to avoid borrow conflicts)
        if let Some(ref mut task) = sched.current_task {
            task.set_state(TaskState::Terminated);
            task.set_exit_code(exit_code);
        }

        // Record exit status for parent
        if let Some(parent_id) = parent {
            let parent_u32 = parent_id.as_u32();
            sched.children_exit_status
                .entry(parent_u32)
                .or_insert_with(Vec::new)
                .push((pid, exit_code));
        }

        drop(scheduler);

        // Wake parent if there is one
        if parent.is_some() {
            Self::wake_waiters(&CHILD_WAIT, usize::MAX);
        }

        Self::schedule();
    }

    /// Terminate a task by PID. Returns 0 on success, u32::MAX on error.
    pub fn terminate_pid(target_pid: u32, exit_code: u32) -> u32 {
        let mut scheduler = SCHEDULER.lock();
        let sched = match scheduler.as_mut() {
            Some(s) => s,
            None => return u32::MAX,
        };

        let current_pid = sched.current_task.as_ref().map(|t| t.id().as_u32());

        // Check if target is the current task
        if current_pid == Some(target_pid) {
            drop(scheduler);
            Self::terminate_current(exit_code);
            return 0;
        }

        // Search ready queues
        for queue in &mut sched.ready_queues {
            if let Some(task) = queue.iter_mut().find(|t| t.id().as_u32() == target_pid) {
                task.set_state(TaskState::Terminated);
                task.set_exit_code(exit_code);
                return 0;
            }
        }

        u32::MAX
    }

    /// Wait for any child process to exit. Returns (child_pid, exit_code)
    /// or u32::MAX if no children.
    pub fn wait_for_child() -> (u32, u32) {
        loop {
            let mut scheduler = SCHEDULER.lock();
            let sched = match scheduler.as_mut() {
                Some(s) => s,
                None => return (u32::MAX, 0),
            };

            let current_pid = sched.current_task.as_ref()
                .map(|t| t.id().as_u32())
                .unwrap_or(u32::MAX);

            // Check if any children have exited
            if let Some(children) = sched.children_exit_status.get_mut(&current_pid) {
                if let Some((child_pid, exit_code)) = children.pop() {
                    if children.is_empty() {
                        sched.children_exit_status.remove(&current_pid);
                    }
                    return (child_pid, exit_code);
                }
            }

            // No exited children — block until one does
            drop(scheduler);
            Self::block_current_on(&CHILD_WAIT);
        }
    }

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

    pub fn with_current_task<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&Task) -> R,
    {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref task) = sched.current_task {
                return Some(f(task));
            }
        }
        None
    }

    fn boost_priorities() {
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            let mut boosted: Vec<Box<Task>> = Vec::new();
            for level in 1..NUM_PRIORITIES {
                while let Some(mut task) = sched.ready_queues[level].pop_front() {
                    task.set_priority(0);
                    boosted.push(task);
                }
            }
            for task in boosted {
                sched.ready_queues[0].push_back(task);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn rust_timer_tick() {
        let ticks = unsafe { ffi::timer_get_ticks_ffi() };

        if ticks > 0 && ticks % BOOST_INTERVAL == 0 {
            Self::boost_priorities();
        }

        let need_preempt = Self::with_current_task_mut(|task| {
            task.increment_ticks();
            task.ticks_used() >= QUANTA[task.priority() as usize]
        }).unwrap_or(false);

        if need_preempt {
            Self::schedule();
        }
    }

    #[no_mangle]
    pub extern "C" fn rust_handle_page_fault(_addr: usize, _err: u32) {
        unsafe {
            crate::ffi::serial_print(c"[Scheduler] rust_handle_page_fault invoked — terminating task\n".as_ptr() as *const u8);
        }

        Self::terminate_current(1);
    }

    #[no_mangle]
    pub extern "C" fn rust_keyboard_wake() {
        Self::wake_waiters(&KEYBOARD_WAIT, 1);
    }

    #[no_mangle]
    pub extern "C" fn rust_mouse_wake() {
        Self::wake_waiters(&MOUSE_WAIT, 1);
    }

    pub fn start() -> ! {
        // First schedule picks the highest-priority task and saves it as
        // the current task.  Then load_context switches to it permanently.
        Self::schedule();

        // If schedule() returns, no task was available — halt.
        loop {
            #[cfg(feature = "x86_64")]
            unsafe { core::arch::asm!("hlt"); }
            #[cfg(feature = "aarch64")]
            unsafe { core::arch::asm!("wfi"); }
        }
    }
}
