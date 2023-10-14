//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translated_byte_buffer, MapPermission, VPNRange, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_real_time,
        get_syscall_times, insert_framed_area, remove_framed_area, suspend_current_and_run_next,
        translate, TaskStatus,
    },
    timer::get_time_ms,
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
    let mut buffers =
        translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());

    let ts = unsafe { &mut *(buffers[0].as_mut_ptr() as *mut TimeVal) };

    let ms = get_time_ms();

    *ts = TimeVal {
        sec: ms / 1000,
        usec: ms * 1000,
    };
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let mut buffers = translated_byte_buffer(
        current_user_token(),
        _ti as *const u8,
        size_of::<TaskInfo>(),
    );

    let ti = unsafe { &mut *(buffers[0].as_mut_ptr() as *mut TaskInfo) };

    ti.status = TaskStatus::Running;
    ti.syscall_times = get_syscall_times();
    ti.time = get_real_time();
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);

    if start_va.page_offset() != 0 || _port & 0x7 == 0 || _port & !0x7 != 0 {
        return -1;
    }

    for vpn in VPNRange::new(start_va.floor(), end_va.ceil()) {
        let pte = translate(vpn);
        if pte.is_some() && pte.unwrap().is_valid() {
            return -1;
        }
    }

    if _len == 0 {
        return 0;
    }

    let mut map_perm = MapPermission::U;

    if _port & 0x1 != 0 {
        map_perm |= MapPermission::R;
    }

    if _port & 0x2 != 0 {
        map_perm |= MapPermission::W;
    }

    if _port & 0x4 != 0 {
        map_perm |= MapPermission::X;
    }

    insert_framed_area(start_va, end_va, map_perm);

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let start_va: VirtAddr = _start.into();
    let end_va: VirtAddr = (_start + _len).into();

    // validity check
    if start_va.page_offset() != 0 {
        return -1;
    }

    for vpn in VPNRange::new(start_va.floor(), end_va.ceil()) {
        let pte = translate(vpn);

        if pte.is_none() || !pte.unwrap().is_valid() {
            return -1;
        }
    }
    remove_framed_area(start_va, end_va);
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
