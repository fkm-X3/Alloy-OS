use alloc::boxed::Box;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, Ordering};

use alloy_kernel_hal::mem::AddressSpace;

/// Error returned when closing a file descriptor fails
#[derive(Debug)]
pub struct FdCloseError;

/// Kernel stack size per task. Must be large enough for the deepest call
/// chain (e.g. the socket table path builds a ~0x4800-byte frame on the
/// caller's stack); 16KB overflowed into adjacent heap free blocks.
pub const KERNEL_STACK_SIZE: usize = 64 * 1024;

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
    /// RAII owner of the task's page directory. Destroyed on drop; mirrors
    /// `context.cr3`/`context.ttbr0`, which the asm context switch reads.
    address_space: AddressSpace,
    #[allow(dead_code)]
    stack: Option<Box<[u8; KERNEL_STACK_SIZE]>>, // kernel stack
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

        // Allocate kernel stack
        let mut stack = Box::new([0u8; KERNEL_STACK_SIZE]);

        // Set up initial context
        let mut context = Box::new(CpuContext::new());

        // Stack grows downward, so ESP/RSP points to the end
        let stack_top = stack.as_mut_ptr() as usize + KERNEL_STACK_SIZE;
        #[cfg(feature = "x86_64")]
        {
            // context_switch does: mov rsp,[ctx.rsp]; push RIP; ret
            // which leaves RSP = ctx.rsp at function entry.  x86_64 ABI
            // requires RSP % 16 == 8 at entry (call pushes 8-byte RA).
            context.rsp = (stack_top - 8) as u64;
            context.rbp = stack_top as u64;
            context.rip = entry as usize as u64;
        }
        #[cfg(feature = "aarch64")]
        {
            // load_context ERETs with SP = ctx.sp, ELR = ctx.elr, SPSR =
            // ctx.spsr for fresh tasks (LR == 0 sentinel).  Preempted
            // tasks resume by `ret`-ing to the saved LR (after
            // `bl save_context`) and unwinding through the IRQ epilogue.
            // Run at EL1h with IRQs unmasked so the timer can preempt us.
            context.sp = (stack_top & !15) as u64;
            context.fp = context.sp;
            context.elr = entry as usize as u64;
            context.lr = 0; // 0 = fresh-task sentinel: load_context ERETs
            context.spsr = 0x5; // M[3:0]=EL1h, DAIF F/I/A/D all unmasked
            context.ttbr0 = AddressSpace::kernel().addr() as u64;
        }
        crate::print!("[Task] Created task with ID ");
        crate::println!("...");

        let mut task = Task {
            id,
            parent_id: None,
            exit_code: 0,
            state: TaskState::Ready,
            context,
            address_space: AddressSpace::kernel(),
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

    /// Set a file descriptor at a specific slot (used by dup2).
    pub fn set_fd_at(&mut self, fd: u32, vnode_id: u64, offset: usize) {
        if (fd as usize) < self.fds.len() {
            self.fds[fd as usize] = Some((vnode_id, offset));
        }
    }

    /// Get vnode id for a fd
    pub fn get_fd(&self, fd: u32) -> Option<u64> {
        if (fd as usize) < self.fds.len() {
            if let Some((vid, _off)) = self.fds[fd as usize] {
                Some(vid)
            } else {
                None
            }
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
        stack: Option<Box<[u8; KERNEL_STACK_SIZE]>>,
        name: String,
        fds: [Option<(u64, usize)>; 32],
        heap_break: u32,
        parent_id: Option<TaskId>,
        address_space: AddressSpace,
    ) -> Self {
        Task {
            id: TaskId::new(),
            parent_id,
            exit_code: 0,
            state: TaskState::Ready,
            context,
            address_space,
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

    /// Replace the task's address space (e.g. on execve). The previous
    /// address space is destroyed, returning its frames to the PMM.
    pub fn set_address_space(&mut self, aspace: AddressSpace) {
        self.address_space = aspace;
    }

    /// The task's current address space.
    pub fn address_space(&self) -> &AddressSpace {
        &self.address_space
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
        crate::println!("[Task] Dropping task");

        if !self.address_space.is_kernel() {
            crate::println!("[Task] Destroying task page directory");
        }
    }
}

/// Entry point for the idle task
extern "C" fn idle_task_entry() {
    loop {
        #[cfg(feature = "x86_64")]
        unsafe {
            core::arch::asm!("sti; hlt", options(nomem, nostack));
        }
        #[cfg(feature = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
