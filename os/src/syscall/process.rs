//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::translated_byte_buffer,
    // syscall::SYSCALL_COUNT,
    task::{
        change_program_brk, current_task_mmap, current_task_munmap, current_user_token,
        exit_current_and_run_next, get_current_task_status, get_current_task_syscall_times,
        get_current_task_time, suspend_current_and_run_next, TaskStatus,
    },
    timer::get_time_us, // timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let token = current_user_token();
    let time = TimeVal {
        sec: get_time_us() / 1_000_000,
        usec: get_time_us() % 1_000_000,
    };
    let len = core::mem::size_of::<TimeVal>();
    // 将应用地址空间转换为内核可以直接访问的缓冲区，避免了虚地址跨页的问题
    let translated_buffer = translated_byte_buffer(token, _ts as *const u8, len);

    if translated_buffer.is_empty() {
        return -1;
    }

    for buffer in translated_buffer {
        unsafe {
            core::ptr::copy(
                &time as *const TimeVal as *const u8,
                buffer.as_mut_ptr(),
                len,
            );
        }
    }

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");

    let running_time = (get_time_us() - get_current_task_time()) / 1000;
    let task_info = TaskInfo {
        status: get_current_task_status(),
        syscall_times: get_current_task_syscall_times(),
        time: running_time,
    };

    let token = current_user_token();
    let len = core::mem::size_of::<TaskInfo>();
    let translate_buffer = translated_byte_buffer(token, _ti as *const u8, len);

    if translate_buffer.is_empty() {
        return -1;
    }

    for buffer in translate_buffer {
        unsafe {
            core::ptr::copy(
                &task_info as *const TaskInfo as *const u8,
                buffer.as_mut_ptr(),
                len,
            );
        }
    }

    0
}

// YOUR JOB: Implement mmap.
// 申请长度为 len 字节的物理内存（不要求实际物理内存位置，可以随便找一块），将其映射到 start 开始的虚存，内存页属性为 port
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    current_task_mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    current_task_munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
