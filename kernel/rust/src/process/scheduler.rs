use crate::process::task::{Task, TaskState};
use crate::process::WaitQueue;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use alloy_kernel_hal::mem::AddressSpace;
use alloy_kernel_hal::sync::SpinLockIrq;
use core::sync::atomic::{AtomicU32, Ordering};

/// Global scheduler instance — must use SpinLockIrq because timer interrupts
/// call rust_timer_tick() which acquires this lock from interrupt context.
static SCHEDULER: SpinLockIrq<Option<Scheduler>> = SpinLockIrq::new(None);

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
                #[cfg(feature = "x86_64")]
                if task.name() == "display-server" {
                    crate::render_trace!(
                        "[T9] display-server PICKED at prio {} (was q{})",
                        task.priority(),
                        level
                    );
                }
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
                    alloy_kernel_hal::load_context(new_ctx_ptr);
                }
                Some(mut old_box) => {
                    let old_ctx_ptr = old_box.context_mut() as *mut _;

                    // Step 1: Save old context — returns normally
                    alloy_kernel_hal::save_context(old_box.context_mut());

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
                                #[cfg(feature = "x86_64")]
                                crate::render_trace!(
                                    "[T9] task exit+leak: {} id={}",
                                    old_box.name(),
                                    old_box.id().as_u32()
                                );
                                #[cfg(feature = "x86_64")]
                                if old_box.name() == "display-server" {
                                    crate::render_trace!(
                                        "[T9] !!! display-server TERMINATED+LEAKED id={}",
                                        old_box.id().as_u32()
                                    );
                                }
                                core::mem::forget(old_box);
                            } else {
                                let t_name = String::from(old_box.name());
                                let t_id = old_box.id().as_u32();
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
                                let t_prio = old_box.priority();
                                old_box.reset_ticks_used();
                                old_box.set_state(TaskState::Ready);
                                re_sched.ready_queues[t_prio as usize].push_back(old_box);
                                #[cfg(feature = "x86_64")]
                                if t_name == "display-server" {
                                    crate::render_trace!(
                                        "[T9] display-server REQUEUED prio={} full_q={}",
                                        t_prio,
                                        had_full_quantum
                                    );
                                }
                                #[cfg(feature = "x86_64")]
                                {
                                    static REQUEUE_N: core::sync::atomic::AtomicU32 =
                                        core::sync::atomic::AtomicU32::new(0);
                                    if crate::fusion::wayland::trace::every_nth(&REQUEUE_N, 200) {
                                        crate::render_trace!(
                                            "[T9] requeue: {} id={} prio={} full_q={}",
                                            t_name,
                                            t_id,
                                            t_prio,
                                            had_full_quantum
                                        );
                                    }
                                }
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
                        alloy_kernel_hal::load_context(new_ctx_ptr);
                    } else {
                        // ── Resume path ─────────────────────────────────
                        // Task was already re-enqueued on the initial path.
                        // old_box is a stale pointer — forget it to prevent
                        // double-free (the real Box is in the ready queue).
                        SCHEDULE_DEPTH.store(0, Ordering::Relaxed);
                        core::mem::forget(old_box);
                    }
                }
            }
        }
    }

    pub fn yield_cpu() {
        Self::schedule();
        // schedule() returns with IRQs disabled (SpinLockIrq held IRQs off).
        // Re-enable them so the task can receive timer interrupts again.
        alloy_kernel_hal::sync::irq_enable();
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
        // schedule() returns with IRQs disabled (SpinLockIrq held IRQs off).
        // Re-enable them so the resumed task can receive timer interrupts again.
        alloy_kernel_hal::sync::irq_enable();
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
        let kernel_pd = AddressSpace::kernel();
        let mut child_as = AddressSpace::kernel();

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

            let current_cr3: usize = sched
                .current_task
                .as_ref()
                .map(|t| t.context().cr3 as usize)
                .unwrap_or(kernel_pd.addr());
            if current_cr3 != kernel_pd.addr() {
                let Some(new_as) = AddressSpace::clone_of(current_cr3) else {
                    return u32::MAX;
                };
                ctx.cr3 = new_as.addr() as u64;
                child_as = new_as;
            } else {
                ctx.cr3 = kernel_pd.addr() as u64;
            }
        }

        let child = Box::new(Task::from_parts(
            ctx,
            Some(Box::new([0u8; crate::process::task::KERNEL_STACK_SIZE])),
            String::from("clone"),
            [None; 32],
            0x01000000,
            None,
            child_as,
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

        #[cfg(feature = "aarch64")]
        let parent_ctx = parent_task.context();
        let kernel_pd = AddressSpace::kernel();

        // The child resumes in user mode immediately after the fork syscall
        // returns (RAX = 0), so its context is rebuilt from the current syscall
        // frame (GS save area) rather than copied from the parent's stored
        // (kernel-mode) context — copying that would make the child restart the
        // entry point and re-fork.
        #[cfg(feature = "x86_64")]
        let frame = alloy_kernel_hal::current_user_syscall_frame();

        #[cfg(feature = "x86_64")]
        let mut child_ctx = {
            let mut ctx = Box::new(crate::process::task::CpuContext::new());
            ctx.rax = 0; // fork returns 0 to the child
            ctx.rip = frame.rip; // resume after `syscall` (RCX)
            ctx.rflags = frame.rflags | 0x200; // keep interrupt flag set
            ctx.rsp = frame.user_rsp; // same user stack (COW-shared)
            ctx.rbx = frame.rbx;
            ctx.rbp = frame.rbp;
            ctx.r12 = frame.r12;
            ctx.r13 = frame.r13;
            ctx.r14 = frame.r14;
            ctx.r15 = frame.r15;
            ctx.cs = 0x23; // user code selector
            ctx.ds = 0x1B; // user data selectors
            ctx.es = 0x1B;
            ctx.fs = 0x1B;
            ctx.gs = 0x1B;
            ctx.ss = 0x1B;
            ctx.fs_base = 0;
            ctx
        };
        #[cfg(feature = "aarch64")]
        let mut child_ctx = Box::new(*parent_ctx);
        let mut child_as = AddressSpace::kernel();

        #[cfg(feature = "x86_64")]
        {
            // Use COW-based address space cloning
            if frame.user_cr3 as usize != kernel_pd.addr() {
                let Some(child_pd) = AddressSpace::fork_of(frame.user_cr3 as usize) else {
                    return u32::MAX;
                };
                child_ctx.cr3 = child_pd.addr() as u64;
                child_as = child_pd;
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
            child_as,
        ));

        let child_pid = child.id().as_u32();
        // Run the child first (before any other ready task) so a freshly forked
        // child is scheduled promptly even with cooperative-style scheduling.
        sched.ready_queues[0].push_front(child);
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
            sched
                .children_exit_status
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
        // schedule() returns with IRQs disabled (SpinLockIrq held IRQs off).
        // Re-enable them so the resumed task can receive timer interrupts again.
        alloy_kernel_hal::sync::irq_enable();
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

            let current_pid = sched
                .current_task
                .as_ref()
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
            #[cfg(feature = "x86_64")]
            if !boosted.is_empty() {
                let names: Vec<String> =
                    boosted.iter().map(|t| String::from(t.name())).collect();
                crate::render_trace!(
                    "[T9] boost: {} task(s) -> q0: {}",
                    names.len(),
                    names.join(",")
                );
            }
            for task in boosted {
                sched.ready_queues[0].push_back(task);
            }
        }
    }

    /// Timer-tick handler, registered via
    /// `set_timer_tick_handler` at boot and invoked from IRQ context on
    /// every timer interrupt.
    pub fn rust_timer_tick() {
        let ticks = crate::SystemTimer::ticks();

        // [T9] Periodic scheduler sample: current task + per-level queue
        // depths.  Proves (or refutes) that a given task — e.g. the display
        // server — is still schedulable, vs. vanished from all queues.
        #[cfg(feature = "x86_64")]
        if ticks > 0 && ticks % 200 == 0 {
            let sched_lock = SCHEDULER.lock();
            if let Some(ref sched) = *sched_lock {
                let cur = match sched.current_task {
                    Some(ref t) => alloc::format!("{}#{}", t.name(), t.id().as_u32()),
                    None => String::from("none"),
                };
                let q0 = queue_names(&sched.ready_queues[0]);
                let q1 = queue_names(&sched.ready_queues[1]);
                let q2 = queue_names(&sched.ready_queues[2]);
                let q3 = queue_names(&sched.ready_queues[3]);
                crate::render_trace!(
                    "[T9] tick {}: cur={} q0=[{}] q1=[{}] q2=[{}] q3=[{}]",
                    ticks,
                    cur,
                    q0,
                    q1,
                    q2,
                    q3
                );
            }
        }

        if ticks > 0 && ticks % BOOST_INTERVAL == 0 {
            Self::boost_priorities();
        }

        let need_preempt = Self::with_current_task_mut(|task| {
            task.increment_ticks();
            task.ticks_used() >= QUANTA[task.priority() as usize]
        })
        .unwrap_or(false);

        if need_preempt {
            Self::schedule();
        }
    }

    /// Page-fault handler, registered via `set_page_fault_handler` at boot
    /// and invoked from exception context for user-mode faults.
    pub fn rust_handle_page_fault(_addr: usize, _err: u32) -> alloy_kernel_hal::FaultAction {
        crate::println!("[Scheduler] rust_handle_page_fault invoked — terminating task");

        Self::terminate_current(1);

        alloy_kernel_hal::FaultAction::Terminate
    }

    /// Wake one task blocked on the keyboard wait queue. Registered as the
    /// keyboard-wake handler at boot (unsafe-core invokes it from IRQ
    /// context on every buffered keypress).
    pub fn rust_keyboard_wake() {
        Self::wake_waiters(&KEYBOARD_WAIT, 1);
    }

    /// Wake one task blocked on the mouse wait queue. Registered as the
    /// mouse-wake handler at boot (unsafe-core invokes it from IRQ context
    /// on every buffered mouse event).
    pub fn rust_mouse_wake() {
        Self::wake_waiters(&MOUSE_WAIT, 1);
    }

    pub fn start() -> ! {
        // First schedule picks the highest-priority task and saves it as
        // the current task.  Then load_context switches to it permanently.
        Self::schedule();

        // If schedule() returns, no task was available — halt.
        loop {
            alloy_kernel_hal::cpu_halt();
        }
    }
}

/// [T9] diagnostic: comma-joined task names in one ready queue.
#[cfg(feature = "x86_64")]
fn queue_names(queue: &VecDeque<Box<Task>>) -> String {
    let mut out = String::new();
    for t in queue.iter() {
        if !out.is_empty() {
            out.push(',');
        }
        out.push_str(t.name());
        out.push('#');
        core::fmt::write(&mut out, format_args!("{}", t.id().as_u32()))
            .ok();
    }
    out
}
