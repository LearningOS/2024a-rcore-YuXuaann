//! Types related to task management & Functions for completely changing TCB

use super::id::TaskUserRes;
use super::{kstack_alloc, KernelStack, ProcessControlBlock, TaskContext};
use crate::trap::TrapContext;
use crate::{mm::PhysPageNum, sync::UPSafeCell};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

/// Task control block structure
pub struct TaskControlBlock {
    /// immutable
    pub process: Weak<ProcessControlBlock>,
    /// Kernel stack corresponding to PID
    pub kstack: KernelStack,
    /// mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

impl TaskControlBlock {
    /// Get the mutable reference of the inner TCB
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    /// Get the address of app's page table
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// The type of banker algorithm
pub enum BankerType {
    /// mutex
    Mutex,
    /// semaphore
    Sem,
}

pub struct TaskBanker {
    /// allocated list
    pub allocated_list: Vec<usize>,
    /// need list
    pub need_list: Vec<usize>,
}

impl TaskBanker {
    pub fn new() -> Self {
        Self {
            allocated_list: Vec::new(),
            need_list: Vec::new(),
        }
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    /// The physical page number of the frame where the trap context is placed
    pub trap_cx_ppn: PhysPageNum,
    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,
    /// It is set when active exit or execution error occurs
    pub exit_code: Option<i32>,

    pub mutex_banker: TaskBanker,
    pub sem_banker: TaskBanker,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn get_allocated_list(&self, type_: BankerType) -> &Vec<usize> {
        match type_ {
            BankerType::Mutex => &self.mutex_banker.allocated_list,
            BankerType::Sem => &self.sem_banker.allocated_list,
        }
    }

    pub fn get_need_list(&mut self, type_: BankerType) -> &mut Vec<usize> {
        match type_ {
            BankerType::Mutex => &mut self.mutex_banker.need_list,
            BankerType::Sem => &mut self.sem_banker.need_list,
        }
    }

    pub fn alloc(&mut self, res_id: usize, type_: BankerType) {
        let banker = match type_ {
            BankerType::Mutex => &mut self.mutex_banker,
            BankerType::Sem => &mut self.sem_banker,
        };
        assert_eq!(banker.allocated_list.len(), banker.need_list.len());
        assert!(res_id < banker.allocated_list.len());
        banker.allocated_list[res_id] += 1;
        banker.need_list[res_id] = 0;
    }

    pub fn dealloc_res(&mut self, res_id: usize, type_: BankerType) {
        let banker = match type_ {
            BankerType::Mutex => &mut self.mutex_banker,
            BankerType::Sem => &mut self.sem_banker,
        };
        assert_eq!(banker.allocated_list.len(), banker.need_list.len());
        // if res_id >= banker.allocated_list.len() {
        //     trace!("tid: {}", self.res.as_ref().unwrap().tid);
        //     trace!("dealloc_res: res_id is {}, res_type is {:?}", res_id, type_);
        //     trace!("allocated_list: {:?}", banker.allocated_list);
        //     assert!(false);
        //     return;
        // }
        assert!(res_id < banker.allocated_list.len());
        banker.allocated_list[res_id] -= 1;
    }

    /// Align the list len
    pub fn extend_list(&mut self, len: usize, type_: BankerType) {
        let banker = match type_ {
            BankerType::Mutex => &mut self.mutex_banker,
            BankerType::Sem => &mut self.sem_banker,
        };
        assert_eq!(banker.allocated_list.len(), banker.need_list.len());
        while banker.allocated_list.len() < len {
            banker.allocated_list.push(0);
            banker.need_list.push(0);
        }
    }
}

impl TaskControlBlock {
    /// Create a new task
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                    mutex_banker: TaskBanker::new(),
                    sem_banker: TaskBanker::new(),
                })
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// The execution status of the current process
pub enum TaskStatus {
    /// ready to run
    Ready,
    /// running
    Running,
    /// blocked
    Blocked,
}
