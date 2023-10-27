//! Process management syscalls
use core::{mem::size_of, ptr::slice_from_raw_parts};

use crate::{config::{MAX_SYSCALL_NUM, self}, mm::*, task::*, timer};

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
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
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

fn set_value<T>(ptr: *mut T, v: &T) {
    let token = current_user_token();
    let len = size_of::<T>();
    unsafe {
        let buff = translated_byte_buffer(token, ptr as _, len);
        let ts_ptr = v as *const T as *const u8;
        let ts_buff = &*slice_from_raw_parts(ts_ptr, len);
        let mut i = 0;
        for page in buff {
            for b in page {
                *b = ts_buff[i];
                i += 1;
            }
        }
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let t = timer::get_time_us();
    let ts = TimeVal {
        sec: t / 1000_000,
        usec: t % 1000_000,
    };

    set_value(_ts, &ts);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let info = get_task_info();
    set_value(_ti, &info);
    0
}
// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    debug!("mmap _start: {:x} _len: {:x} _port: {:x}", _start, _len, _port);

    if _start > config::MEMORY_END{
        return  -1;
    }

    if _port & !0x7 != 0 {
        return -1;
    }
    if _port & 0x7 == 0 {
        return -1;
    }
    let pte = _port << 1;
    let r =current_mmap(_start, _len, pte as u8);
    debug!("r: {r}");
    r
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    debug!("munmap: {:x} {:x}", _start, _len);
    // current_mmap(start, len, ple)
    current_munmap(_start, _len)
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
