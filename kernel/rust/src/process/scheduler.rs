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
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            // Put current task back in ready queue if it's still runnable
            if let Some(mut current) = sched.current_task.take() {
                match current.state() {
                    TaskState::Running => {
                        // Task is still runnable, put it back in ready queue
                        current.set_state(TaskState::Ready);
                        sched.ready_queue.push_back(current);
                    }
                    TaskState::Terminated => {
                        // Task is done, drop it
                        unsafe {
                            ffi::serial_print(b"[Scheduler] Task terminated\n\0".as_ptr());
                        }
                        drop(current);
                    }
                    _ => {
                        // Blocked or other state - for now, just put back in queue
                        current.set_state(TaskState::Ready);
                        sched.ready_queue.push_back(current);
                    }
                }
            }
            
            // Pick next task
            if let Some(next_task) = sched.pick_next() {
                unsafe {
                    ffi::serial_print(b"[Scheduler] Switching to next task\n\0".as_ptr());
                }
                sched.current_task = Some(next_task);
            } else {
                unsafe {
                    ffi::serial_print(b"[Scheduler] No more tasks in queue\n\0".as_ptr());
                }
            }
        }
    }
    
    /// Yield CPU to another task (for cooperative multitasking)
    pub fn yield_cpu() {
        unsafe {
            ffi::serial_print(b"[Scheduler] Task yielding CPU\n\0".as_ptr());
        }
        
        // For now, we just schedule without context switching
        Self::schedule();
        
        // Execute the next task directly (simplified - no real context switching yet)
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref task) = sched.current_task {
                let entry = task.context().eip;
                drop(scheduler); // Release lock
                
                unsafe {
                    ffi::serial_print(b"[Scheduler] Executing task\n\0".as_ptr());
                }
                
                let entry_fn: extern "C" fn() = unsafe { core::mem::transmute(entry) };
                entry_fn();
            }
        }
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
        
        // Schedule and run first task
        Self::schedule();
        
        // Execute first task
        let mut scheduler = SCHEDULER.lock();
        if let Some(ref mut sched) = *scheduler {
            if let Some(ref task) = sched.current_task {
                let entry = task.context().eip;
                drop(scheduler); // Release lock before jumping
                
                unsafe {
                    ffi::serial_print(b"[Scheduler] Jumping to first task\n\0".as_ptr());
                }
                
                // Jump to first task (direct call for now, no context switching)
                let entry_fn: extern "C" fn() = unsafe { core::mem::transmute(entry) };
                entry_fn();
            }
        }
        
        // Should never reach here
        unsafe {
            ffi::serial_print(b"[Scheduler] ERROR: Scheduler returned!\n\0".as_ptr());
        }
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }
}
