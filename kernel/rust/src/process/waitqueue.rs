use alloc::collections::VecDeque;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::process::task::Task;
use crate::sync::SpinLock;

pub struct WaitQueue {
    inner: SpinLock<WaitQueueInner>,
}

struct WaitQueueInner {
    tasks: VecDeque<Box<Task>>,
}

impl WaitQueue {
    pub const fn new() -> Self {
        WaitQueue {
            inner: SpinLock::new(WaitQueueInner {
                tasks: VecDeque::new(),
            }),
        }
    }

    pub fn enqueue(&self, task: Box<Task>) {
        self.inner.lock().tasks.push_back(task);
    }

    pub fn dequeue(&self) -> Option<Box<Task>> {
        self.inner.lock().tasks.pop_front()
    }

    pub fn dequeue_all(&self) -> Vec<Box<Task>> {
        let mut inner = self.inner.lock();
        let mut result = Vec::new();
        while let Some(task) = inner.tasks.pop_front() {
            result.push(task);
        }
        result
    }

    pub fn has_waiters(&self) -> bool {
        !self.inner.lock().tasks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().tasks.len()
    }
}
