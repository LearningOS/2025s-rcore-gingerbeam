//! File and filesystem-related syscalls
use crate::fs::{find, link, link_count, open_file, link_count_id, unlink, OSInode, OpenFlags, Stat, StatMode};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use core::any::Any;
use core::slice::{from_raw_parts_mut};

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
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat",
        current_task().unwrap().pid.0
    );

    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }

    if let Some(file) = &inner.fd_table[fd] { // Option<Arc<OSInode>>
        let temp: &dyn Any = file.as_any();
        let file_osinode = temp.downcast_ref::<OSInode>().unwrap();
        let inner = file_osinode.inner_exclusive_access();

        let stat = Stat {
            dev: 0,
            ino: inner.get_inode_id() as u64, // the inode number of the inode (block_id, in which block in the inode area)
            mode: StatMode::FILE, // the type of the inode, only FILE are allowed
            nlink: link_count_id(inner.get_inode_id() as u32) as u32,
            pad: [0;7],
        };

        let mut stat_slice = unsafe {
            from_raw_parts_mut(
                &stat as *const Stat as *mut u8,
                core::mem::size_of::<Stat>()
            )
        };

        // the address for st is in the user space\
        let st_buf = translated_byte_buffer(
            token,
            st as *const u8,
            core::mem::size_of::<Stat>()
        );

        for buf in st_buf {
            buf.copy_from_slice(&stat_slice[0..buf.len()]);
            stat_slice = &mut stat_slice[buf.len()..];
        }

        return 0;
    }

    -1
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat",
        current_task().unwrap().pid.0
    );

    // let task = current_task().unwrap();
    let token = current_user_token();
    let old_path = translated_str(token, old_name);
    let new_path = translated_str(token, new_name);
    
    if old_path.as_str() != new_path.as_str() {
        if link(old_path.as_str(), new_path.as_str()).is_some() {
            debug!(
                "kernel: sys_linkat success for {} to {}!",
                new_path,
                old_path
            );
            debug!(
                "kernel: sys_linkat counter is now {}!",
                link_count(new_path.as_str())
            );
            return 0;
        }
    }

    -1
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat",
        current_task().unwrap().pid.0
    );

    let path = translated_str(current_user_token(), name);

    // debug!("Unlink path: {}", path);

    // if this is the last link count
    if let Some(inode) = find(path.as_str()) {
        if link_count(path.as_str()) == 1 {
            inode.clear();
        }
        // unlink
        return unlink(path.as_str());
    }

    -1
}
