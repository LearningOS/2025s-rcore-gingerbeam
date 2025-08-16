//! Process management syscalls
use crate::config::PAGE_SIZE;
use crate::task::{change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
                  read_syscall_count, get_current_task_pte, create_new_map_area, unmap_area};
use crate::mm::{translated_byte_buffer};
use crate::mm::{VirtAddr, MapPermission, VPNRange};
use core::slice::{from_raw_parts_mut};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // TimeVal is in the user space
    // can not write directly with the bare pointer from user space
    // refer to sys_write
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,  
    };
    let mut tv_slice = unsafe {
        from_raw_parts_mut(
            &time_val as *const TimeVal as *mut u8,
            core::mem::size_of::<TimeVal>()
        )
    };

    let tv_buf = translated_byte_buffer(
        current_user_token(), 
        ts as *const u8,
        core::mem::size_of::<TimeVal>()
    );

    // U mode memory may not be continuous
    // i.e. the TimeVal object may not be on the same page
    for buf in tv_buf {
        // copy from tv_slice to buf byte by byte
        buf.copy_from_slice(&tv_slice[0..buf.len()]);
        tv_slice = &mut tv_slice[buf.len()..];
    }

    // debug!("debug: sys_get_time: finished");
    
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    // debug!("debug: sys_trace: {:?}, {:?}, {:?}", trace_request, id, data);
    let va = VirtAddr::from(id);
    let vpn = va.floor();
    let offset = va.page_offset();
    assert!(offset < PAGE_SIZE);

    match trace_request {
        0 => {
            // TODO: read a usize from current task at address _id as *const u8
            // return the read value
            // check whether the address is visible to U mode
            if let Some(pte) = get_current_task_pte(vpn) {
                // debug!("debug: sys_trace: address valid");
                if pte.is_valid() && pte.user_accessible() && pte.readable() {
                    // debug!("debug: sys_trace: address readable");
                    let ppn = pte.ppn();
                    // we only need one page here so we use get_bytes_array
                    // rather than translate_byte_buf
                    let res: isize =  ppn.get_bytes_array()[offset] as isize;
                    // debug!("debug: sys_trace: read value: {:?}", res);
                    return res;
                } else {
                    return -1;
                }
            } else {
                // address invalid
                return -1;
            }
        },
        1 => {
            // TODO: write _data to current task at address _id as *const u8
            // check whether the address is visible to U mode
            if let Some(pte) = get_current_task_pte(vpn) {
                if pte.is_valid() && pte.user_accessible() && pte.writable() {
                    let page_bytes = pte.ppn().get_bytes_array();
                    page_bytes[offset] = data as u8;
                    return 0;
                } else {
                    return -1;
                }
            } else {
                // address invalid
                return -1;
            }
        },
        2 => {
            return read_syscall_count(id) as isize;
        },
        _ => {
            return -1;
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    // debug!("debug: sys_mmap: {:#x}, {:?}, {:#x}", start, len, port);
    // allocate physical memory in len bytes
    // for virtual memory from start
    // MapPermission = RWXU
    if start % PAGE_SIZE != 0 || // start must be aligned with PAEG_SIZE
       port & !0x7 != 0 || // only 0,1,2 bits can the non-zero
       port & 0x7 == 0 { // useless memory
        return -1;
    }
    // should not run out of memory
    // how to do this?

    // check if a page in [start, start + len) is already mapped
    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    let vpn_range = VPNRange::new(start_vpn, end_vpn);
    for vpn in vpn_range {
        if let Some(pte) = get_current_task_pte(vpn) {
            if pte.is_valid() {
                return -1;
            }
        }
    }

    // convert MapPermission
    let map_permission = MapPermission::from_bits_truncate((port << 1) as u8) | MapPermission::U;
    // create new MapArea
    // allocating physical frames is integrated with creating MapAreas
    create_new_map_area(start_vpn.into(), end_vpn.into(), map_permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    // debug!("debug: sys_munmap: {:#x}, {:?}", start, len);
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    // check if a virtual page in [start, start + len) is not mapped
    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    let vpn_range = VPNRange::new(start_vpn, end_vpn);
    for vpn in vpn_range {
        if let Some(pte) = get_current_task_pte(vpn) {
            // check if the pte is valid
            if !pte.is_valid() {
                return -1;
            } else {
                // unmap this page
                // pagetable_from_token(get_current_task_token()).unmap(vpn);
                unmap_area(vpn);
            }
        } else {
            // simply not mapped
            return -1;
        }
    }

    // unmap 

    0
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
