//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
// use alloc::collections::VecDeque;
use alloc::{collections::binary_heap::BinaryHeap, sync::Arc};
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    // ready_queue: VecDeque<Arc<TaskControlBlock>>, // init
    ready_queue: BinaryHeap<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        // self.ready_queue.push_back(task); // init
        self.ready_queue.push(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // self.ready_queue.pop_front() // init
        if self.ready_queue.is_empty() {
            return None;
        }

        let fetched_task = self.ready_queue.pop();
        let fetched_task_unwarp = fetched_task.unwrap();
        fetched_task_unwarp.inner_exclusive_access().update_stride();
        Some(fetched_task_unwarp)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
