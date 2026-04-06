use alloc::boxed::Box;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::ffi;

// Task ID counter for unique task IDs
static NEXT_TASK_ID: AtomicU32 = AtomicU32::new(1);

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u32);

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
}

impl CpuContext {
    /// Create a zeroed context
    pub fn new() -> Self {
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
        }
    }
}

/// Represents a schedulable task
pub struct Task {
    id: TaskId,
    state: TaskState,
    context: Box<CpuContext>,
    stack: Option<Box<[u8; 4096]>>,  // 4KB kernel stack
    name: String,
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
        context.eip = entry as u32;
        
        unsafe {
            ffi::serial_print(b"[Task] Created task with ID \0".as_ptr());
            // Print simple message without trying to print the name (causes issues)
            ffi::serial_print(b"...\n\0".as_ptr());
        }
        
        Task {
            id,
            state: TaskState::Ready,
            context,
            stack: Some(stack),
            name: String::from(name),
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
            ffi::serial_print(b"[Task] Dropping task\n\0".as_ptr());
        }
    }
}

/// Entry point for the idle task
extern "C" fn idle_task_entry() {
    loop {
        // HLT instruction to save power when idle
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
