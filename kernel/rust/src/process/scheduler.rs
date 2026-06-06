use alloc::collections::VecDeque;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use crate::process::task::{Task, TaskState};
use crate::sync::SpinLock;
use crate::ffi;

/// Global scheduler instance
static SCHEDULER: SpinLock<Option<Scheduler>> = SpinLock::new(None);

const NUM_PRIORITIES: usize = 4;
const QUANTA: [u32; NUM_PRIORITIES] = [5, 10, 20, 40];
const BOOST_INTERVAL: u64 = 100;

/// Multi-level feedback queue scheduler
pub struct Scheduler {
    ready_queues: Vec<VecDeque<Box<Task>>>,
    current_task: Option<Box<Task>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        unsafe {
            ffi::serial_print(c"[Scheduler] Initializing MLFQ scheduler\n".as_ptr() as *const u8);
        }

        let mut ready_queues = Vec::with_capacity(NUM_PRIORITIES);
        for _ in 0..NUM_PRIORITIES {
            ready_queues.push(VecDeque::new());
        }

        Scheduler {
            ready_queues,
            current_task: None,
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
            unsafe {
                ffi::serial_print(c"[Scheduler] Adding task to ready queue\n".as_ptr() as *const u8);
            }
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

            unsafe {
                ffi::serial_print(c"[Scheduler] Preparing context switch\n".as_ptr() as *const u8);
            }

            drop(scheduler_lock);

            if let Some(mut old_box) = old_box_opt {
                let old_ctx_ptr: *mut crate::process::task::CpuContext = old_box.context_mut() as *mut _;

                unsafe {
                    ffi::serial_print(c"[Scheduler] Calling context_switch\n".as_ptr() as *const u8);
                    ffi::context_switch(old_ctx_ptr, new_ctx_ptr);
                    ffi::serial_print(c"[Scheduler] Returned from context_switch (old context)\n".as_ptr() as *const u8);
                }

                let mut scheduler_lock = SCHEDULER.lock();
                if let Some(ref mut sched) = *scheduler_lock {
                    match old_box.state() {
                        TaskState::Running | TaskState::Ready => {
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
                            sched.ready_queues[old_box.priority() as usize].push_back(old_box);
                        }
                        TaskState::Terminated => {
                            unsafe { ffi::serial_print(c"[Scheduler] Old task terminated after switch\n".as_ptr() as *const u8); }
                            drop(old_box);
                        }
                        _ => {
                            old_box.reset_ticks_used();
                            old_box.set_state(TaskState::Ready);
                            sched.ready_queues[0].push_back(old_box);
                        }
                    }
                }
            } else {
                unsafe { ffi::serial_print(c"[Scheduler] No old task, initial run\n".as_ptr() as *const u8); }
            }
        }
    }

    pub fn yield_cpu() {
        unsafe {
            ffi::serial_print(c"[Task] Yielding CPU\n".as_ptr() as *const u8);
        }
        Self::schedule();
    }

    pub fn clone_task(entry: u32, stack: u32, arg: u32) -> u32 {
        let mut scheduler = SCHEDULER.lock();
        let sched = match scheduler.as_mut() {
            Some(s) => s,
            None => return u32::MAX,
        };
        let mut ctx = Box::new(crate::process::task::CpuContext::new());
        ctx.eip = entry;
        ctx.esp = stack;
        ctx.ebp = stack;
        ctx.eax = arg;
        ctx.cs = 0x1B;
        ctx.ds = 0x23;
        ctx.es = 0x23;
        ctx.fs = 0x23;
        ctx.gs = 0x23;
        ctx.ss = 0x23;

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
        sched.ready_queues[0].push_back(child);
        pid
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
    pub extern "C" fn rust_handle_page_fault(_addr: u32, _err: u32) {
        unsafe {
            crate::ffi::serial_print(c"[Scheduler] rust_handle_page_fault invoked\n".as_ptr() as *const u8);
        }

        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref mut task) = sched.current_task {
                task.set_state(TaskState::Terminated);
                unsafe { crate::ffi::serial_print(c"[Scheduler] Marked current task as Terminated\n".as_ptr() as *const u8); }
            }
        }

        Self::schedule();
    }

    pub fn start() -> ! {
        unsafe {
            ffi::serial_print(c"[Scheduler] Starting scheduler\n".as_ptr() as *const u8);
            ffi::vga_println(c"\nStarting multitasking...\n".as_ptr() as *const u8);
        }

        Self::schedule();

        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref mut task) = sched.current_task {
                let mut kernel_ctx = crate::process::task::CpuContext::new();

                let new_ctx_ptr: *mut crate::process::task::CpuContext = task.context_mut() as *mut _;

                drop(scheduler);

                unsafe {
                    ffi::serial_print(c"[Scheduler] Performing initial context_switch to first task\n".as_ptr() as *const u8);
                    ffi::context_switch(&mut kernel_ctx as *mut _, new_ctx_ptr);
                    ffi::serial_print(c"[Scheduler] Returned from initial context_switch\n".as_ptr() as *const u8);
                }

                let mut scheduler = SCHEDULER.lock();
                if let Some(ref mut sched) = *scheduler {
                    if let Some(mut current) = sched.current_task.take() {
                        current.set_state(TaskState::Terminated);
                        drop(current);
                    }
                    Self::schedule();
                }
            }
        }

        unsafe {
            ffi::serial_print(c"[Scheduler] ERROR: Scheduler returned!\n".as_ptr() as *const u8);
        }
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
}
