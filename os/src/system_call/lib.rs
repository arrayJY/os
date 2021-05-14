pub fn sys_write(buffer: *const u8, len: usize) -> isize {
    use crate::print;
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    let str = core::str::from_utf8(slice).unwrap();
    print!("{}", str);
    len as isize
}

pub fn sys_exit(exit_code: isize) -> ! {
    use crate::task::{stop_current_and_run_next};
    use crate::println;
    println!("[kernel] Task exited with return code {}.", exit_code);
    stop_current_and_run_next();
    panic!("sys_exit never returns!");
}

pub fn sys_wait(pid: isize, exit_code: *mut isize) -> isize {
    todo!()
}
pub fn sys_fork() -> isize {
    todo!()
}
pub fn sys_exec(path: &str) -> isize {
    todo!()
}
