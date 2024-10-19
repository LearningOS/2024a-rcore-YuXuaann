//! File and filesystem-related syscalls
use crate::fs::nlink;
#[allow(unused_imports)]
use crate::fs::{
    find_inode, link, open_file, show_hard_link, show_inode_under_root, unlink, OpenFlags, Stat,
};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token, write_to_current_user_buffer};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        println!("[sys_open] open_file failed");
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!("kernel:pid[{}] sys_fstat", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    // let len = core::mem::size_of::<Stat>();
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[_fd] {
        let file = file.clone();
        drop(inner);

        let inode_id = file.inode_id();
        let stat = Stat {
            dev: 0,
            ino: inode_id as u64,
            mode: file.mode(),
            nlink: nlink(inode_id),
            pad: [0; 7],
        };
        let len = core::mem::size_of::<Stat>();
        write_to_current_user_buffer(_st as *const u8, &stat as *const Stat as *const u8, len)
    } else {
        -1
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_linkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let old_name = translated_str(token, _old_name);
    let new_name = translated_str(token, _new_name);
    let old_inode = find_inode(&old_name);
    let new_inode = find_inode(&new_name);

    println!(
        "[sys_linkat] old_inode: {:?}, new_inode: {:?}",
        old_name, new_name
    );

    if old_inode.is_none() {
        println!("[sys_linkat] old_inode is none");
        return -1;
    }

    if new_inode.is_some() {
        println!("[sys_linkat] new_inode is some");
        return -1;
    }

    link(&new_name, old_inode.unwrap().inode_id());
    // show_hard_link();
    0
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_unlinkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let name = translated_str(token, _name);
    let inode = find_inode(&name);
    let inode_id = inode.clone().unwrap().inode_id();

    println!("[sys_unlinkat] nlink: {}", inode_id);

    if inode.is_none() {
        println!("[sys_unlinkat] inode is none");
        return -1;
    }

    unlink(&name);
    let count = nlink(inode_id as usize);
    if count == 0 {
        println!("[sys_unlinkat] count is 0");
        inode.unwrap().clear();
    }
    0
}
