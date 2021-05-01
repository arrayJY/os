pub mod lib;

use crate::task::{TaskStatus, TASK_MANAGER};
use alloc::vec::Vec;
use x86_64::{
    structures::paging::{mapper::TranslateResult, OffsetPageTable, Translate},
    VirtAddr,
};

use crate::memory::{phsyical_memory_offset, PAGE_SIZE};

pub fn sysexec(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        1 => sys_write(args[0] as *const u8, args[1]),
        2 => sys_exit(args[0] as isize),
        _ => panic!("Unsupported system call."),
    }
}

pub fn sys_write(buffer: *const u8, len: usize) -> isize {
    use crate::print;
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    let str = core::str::from_utf8(slice).unwrap();
    print!("{}", str);
    len as isize
}

pub fn sys_exit(exit_code: isize) -> isize {
    use crate::println;
    let mut task_manager = TASK_MANAGER.lock();
    println!("Task {} exited with return code {}.", task_manager.current_task, exit_code);
    let task = task_manager.current_task_mut();
    task.task_status = TaskStatus::Stop;
    0
}
