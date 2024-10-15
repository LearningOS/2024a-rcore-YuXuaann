//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of `PID_ALLOCATOR` allocates pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod id;
mod manager;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    loader::get_app_data_by_name,
    mm::MapPermission,
    syscall::TaskInfo,
    timer::get_time_us,
};
use alloc::sync::Arc;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager};
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
pub use manager::add_task;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    write_to_current_user_buffer, Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        panic!("All applications completed!");
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("ch5b_initproc").unwrap()
    ));
}

///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

/// Update syscall times
pub fn count_syscall_times(syscall_id: usize) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.syscall_times[syscall_id] += 1;
}

/// Get status of current task
pub fn get_current_task_status() -> TaskStatus {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    inner.get_status()
}

/// Get syscall_times of current task
pub fn get_current_task_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    inner.syscall_times
}

/// Get start_time of current task
pub fn get_current_task_start_time() -> usize {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    inner.start_time
}

/// Get current task info
pub fn get_current_task_info() -> TaskInfo {
    let running_time = get_time_us() - get_current_task_start_time();
    TaskInfo {
        status: get_current_task_status(),
        syscall_times: get_current_task_syscall_times(),
        time: running_time / 1000,
    }
}

/// Add mmap to the current task
pub fn current_task_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    // start 没有按页大小对齐
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    // port & !0x7 != 0 (port 其余位必须为0)
    if _port & !0x7 != 0 {
        return -1;
    }
    // port & 0x7 = 0
    if _port & 0x7 == 0 {
        return -1;
    }

    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let mut map_perm = MapPermission::empty();

    if _port & (1 << 0) != 0 {
        map_perm |= MapPermission::R;
    }
    if _port & (1 << 1) != 0 {
        map_perm |= MapPermission::W;
    }
    if _port & (1 << 2) != 0 {
        map_perm |= MapPermission::X;
    }
    // don't forget to add user permission
    map_perm |= MapPermission::U;

    inner.memory_set.mmap(_start, _len, map_perm)
}

/// Unmmap to the current task
pub fn current_task_munmap(_start: usize, _len: usize) -> isize {
    // start 没有按页大小对齐
    if _start % PAGE_SIZE != 0 {
        return -1;
    }

    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.memory_set.unmap(_start, _len)
}
