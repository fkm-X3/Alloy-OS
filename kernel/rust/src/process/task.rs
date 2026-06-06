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

/// CPU context structure matching C++ cpu_context
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    // General purpose registers
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,
    
    // Instruction pointer
    pub eip: u32,
    
    // Segment registers
    pub cs: u32,
    pub ds: u32,
    pub es: u32,
    pub fs: u32,
    pub gs: u32,
    pub ss: u32,
    
    // EFLAGS register
    pub eflags: u32,
    // CR3 - page directory physical address
    pub cr3: u32,
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuContext {
    /// Create a zeroed context
    pub fn new() -> Self {
        // Default CR3 to kernel page directory
        let kernel_cr3 = unsafe { crate::ffi::paging_get_kernel_directory_phys() };

        CpuContext {
            eax: 0, ebx: 0, ecx: 0, edx: 0,
            esi: 0, edi: 0, ebp: 0, esp: 0,
            eip: 0,
            cs: 0x08,  // Kernel code segment
            ds: 0x10,  // Kernel data segment
            es: 0x10,
            fs: 0x10,
            gs: 0x10,
            ss: 0x10,  // Kernel stack segment
            eflags: 0x202,  // IF (interrupt enable) flag set
            cr3: kernel_cr3,
        }
    }
}

/// Represents a schedulable task
pub struct Task {
    id: TaskId,
    state: TaskState,
    context: Box<CpuContext>,
    #[allow(dead_code)]
    stack: Option<Box<[u8; 4096]>>,  // 4KB kernel stack
    name: String,
    // Simple file descriptor table (map fd -> (vnode id, offset)). None means free.
    fds: [Option<(u64, usize)>; 32],
    // Program break for userland heap (brk/sbrk)
    heap_break: u32,
}


impl Task {
    /// Create a new task with the given entry point
    pub fn new(entry: extern "C" fn(), name: &str) -> Self {
        let id = TaskId::new();
        
        // Allocate kernel stack (4KB)
        let mut stack = Box::new([0u8; 4096]);
        
        // Set up initial context
        let mut context = Box::new(CpuContext::new());
        
        // Stack grows downward, so ESP points to the end
        let stack_top = stack.as_mut_ptr() as usize + 4096;
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
            state: TaskState::Ready,
            context,
            stack: Some(stack),
            name: String::from(name),
            fds: [None; 32],
            heap_break: 0x01000000,
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
    
    /// Create the idle task (special task with no real work)
    pub fn new_idle() -> Self {
        Self::new(idle_task_entry, "idle")
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
}

impl Drop for Task {
    fn drop(&mut self) {
        unsafe {
            ffi::serial_print(c"[Task] Dropping task\n".as_ptr() as *const u8);
        }

        // If this task has its own page directory (CR3) different from the kernel's,
        // destroy it and free all user pages and page tables.
        let pd = self.context.cr3;
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
            // Enable interrupts then halt — an interrupt (e.g. timer)
            // will wake us and the scheduler can pick a real task.
            core::arch::asm!("sti; hlt");
        }
    }
}
