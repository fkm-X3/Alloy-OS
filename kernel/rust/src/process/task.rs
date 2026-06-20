use alloc::boxed::Box;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::ffi;

/// Error returned when closing a file descriptor fails
#[derive(Debug)]
pub struct FdCloseError;

// Task ID counter for unique task IDs
static NEXT_TASK_ID: AtomicU32 = AtomicU32::new(1);

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u32);

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskId {
    /// Generate a new unique task ID
    pub fn new() -> Self {
        TaskId(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Task execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,      // Ready to run
    Running,    // Currently executing
    Blocked,    // Waiting for something
    Terminated, // Finished execution
}

/// CPU context for task switching.
///
/// Type is re-exported from the HAL crate, which defines an architecture-specific
/// layout matching the C `cpu_context` struct (see `kernel/hal/src/arch/mod.rs`).
pub use alloy_kernel_hal::CpuContext;

/// Represents a schedulable task
pub struct Task {
    id: TaskId,
    parent_id: Option<TaskId>,
    exit_code: u32,
    state: TaskState,
    context: Box<CpuContext>,
    #[allow(dead_code)]
    stack: Option<Box<[u8; 16384]>>,  // 16KB kernel stack
    name: String,
    // Simple file descriptor table (map fd -> (vnode id, offset)). None means free.
    fds: [Option<(u64, usize)>; 32],
    // Program break for userland heap (brk/sbrk)
    heap_break: u32,
    // MLFQ priority (0 = highest, larger = lower)
    priority: u8,
    // Timer ticks consumed in current quantum
    ticks_used: u32,
}


impl Task {
    /// Create a new task with the given entry point
    pub fn new(entry: extern "C" fn(), name: &str) -> Self {
        let id = TaskId::new();
        
        // Allocate kernel stack (16KB)
        let mut stack = Box::new([0u8; 16384]);
        
        // Set up initial context
        let mut context = Box::new(CpuContext::new());
        
        // Stack grows downward, so ESP points to the end
        let stack_top = stack.as_mut_ptr() as usize + 16384;
        context.esp = stack_top as u32;
        context.ebp = stack_top as u32;
        
        // Set entry point
        context.eip = entry as usize as u32;
        
        unsafe {
            ffi::serial_print(c"[Task] Created task with ID ".as_ptr() as *const u8);
            // Print simple message without trying to print the name (causes issues)
            ffi::serial_print(c"...\n".as_ptr() as *const u8);
        }
        
        let mut task = Task {
            id,
            parent_id: None,
            exit_code: 0,
            state: TaskState::Ready,
            context,
            stack: Some(stack),
            name: String::from(name),
            fds: [None; 32],
            heap_break: 0x01000000,
            priority: 0,
            ticks_used: 0,
        };

        // Try to open /dev/console for stdout/stderr (fd 1 and 2) if available
        if let Ok(vnode_id) = crate::fs::vfs_open("/dev/console", 0, 0) {
            // allocate fd 1
            if let Some(fd1) = task.alloc_fd(vnode_id) {
                // If fd1 != 1, swap into slot 1
                if fd1 != 1 {
                    task.fds[1] = task.fds[fd1 as usize];
                    task.fds[fd1 as usize] = None;
                }
                // allocate fd 2 as duplicate
                if let Some(fd2) = task.alloc_fd(vnode_id) {
                    if fd2 != 2 {
                        task.fds[2] = task.fds[fd2 as usize];
                        task.fds[fd2 as usize] = None;
                    }
                }
            }
        }

        task
    }

    /// Allocate a file descriptor for the current task. Returns fd or None.
    pub fn alloc_fd(&mut self, vnode_id: u64) -> Option<u32> {
        for (i, slot) in self.fds.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some((vnode_id, 0usize));
                return Some(i as u32);
            }
        }
        None
    }

    /// Get vnode id for a fd
    pub fn get_fd(&self, fd: u32) -> Option<u64> {
        if (fd as usize) < self.fds.len() {
            if let Some((vid, _off)) = self.fds[fd as usize] {
                Some(vid)
            } else { None }
        } else {
            None
        }
    }

    /// Get mutable reference to fd entry (vnode_id, offset)
    pub fn get_fd_entry_mut(&mut self, fd: u32) -> Option<&mut (u64, usize)> {
        if (fd as usize) < self.fds.len() {
            self.fds[fd as usize].as_mut()
        } else {
            None
        }
    }

    /// Get heap break
    pub fn heap_break(&self) -> u32 {
        self.heap_break
    }

    /// Set heap break
    pub fn set_heap_break(&mut self, brk: u32) {
        self.heap_break = brk;
    }

    /// Close a file descriptor
    pub fn close_fd(&mut self, fd: u32) -> Result<(), FdCloseError> {
        if (fd as usize) < self.fds.len() {
            self.fds[fd as usize] = None;
            Ok(())
        } else {
            Err(FdCloseError)
        }
    }
    
    /// Create a task from raw parts (used by clone/fork)
    pub fn from_parts(
        context: Box<CpuContext>,
        stack: Option<Box<[u8; 16384]>>,
        name: String,
        fds: [Option<(u64, usize)>; 32],
        heap_break: u32,
        parent_id: Option<TaskId>,
    ) -> Self {
        Task {
            id: TaskId::new(),
            parent_id,
            exit_code: 0,
            state: TaskState::Ready,
            context,
            stack,
            name,
            fds,
            heap_break,
            priority: 0,
            ticks_used: 0,
        }
    }

    /// Create the idle task (special task with no real work, lowest priority)
    pub fn new_idle() -> Self {
        let mut task = Self::new(idle_task_entry, "idle");
        task.priority = 3;
        task
    }
    
    /// Get task ID
    pub fn id(&self) -> TaskId {
        self.id
    }
    
    /// Get current state
    pub fn state(&self) -> TaskState {
        self.state
    }
    
    /// Set task state
    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }
    
    /// Get task name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get mutable reference to CPU context
    pub fn context_mut(&mut self) -> &mut CpuContext {
        &mut self.context
    }
    
    /// Get immutable reference to CPU context
    pub fn context(&self) -> &CpuContext {
        &self.context
    }

    pub fn parent_id(&self) -> Option<TaskId> {
        self.parent_id
    }

    pub fn set_parent_id(&mut self, pid: Option<TaskId>) {
        self.parent_id = pid;
    }

    pub fn exit_code(&self) -> u32 {
        self.exit_code
    }

    pub fn set_exit_code(&mut self, code: u32) {
        self.exit_code = code;
    }

    pub fn clone_fds(&self) -> [Option<(u64, usize)>; 32] {
        self.fds
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }

    pub fn set_priority(&mut self, prio: u8) {
        self.priority = prio;
    }

    pub fn ticks_used(&self) -> u32 {
        self.ticks_used
    }

    pub fn increment_ticks(&mut self) {
        self.ticks_used = self.ticks_used.saturating_add(1);
    }

    pub fn reset_ticks_used(&mut self) {
        self.ticks_used = 0;
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        unsafe {
            ffi::serial_print(c"[Task] Dropping task\n".as_ptr() as *const u8);
        }

        let pd = self.context.cr3 as usize;
        let kernel_pd = unsafe { ffi::paging_get_kernel_directory_phys() };
        if pd != 0 && pd != kernel_pd {
            unsafe {
                ffi::serial_print(c"[Task] Destroying task page directory\n".as_ptr() as *const u8);
                ffi::paging_destroy_directory(pd);
            }
        }
    }
}

/// Entry point for the idle task
extern "C" fn idle_task_entry() {
    loop {
        unsafe {
            core::arch::asm!("sti; hlt", options(nomem, nostack));
        }
    }
}
