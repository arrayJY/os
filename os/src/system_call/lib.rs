
pub fn sys_write(buffer: *const u8, len: usize) -> isize {
    use crate::print;
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    let str = core::str::from_utf8(slice).unwrap();
    print!("{}", str);
    len as isize
}

pub fn sys_exit(exit_code: isize) -> ! {
    use crate::println;
    use crate::process::exit_current_and_run_next;
    println!("[kernel] Task exited with return code {}.", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("sys_exit never returns!");
}
pub fn sys_wait(pid: isize, exit_code: *mut isize) -> isize {
    todo!()
}
pub fn sys_fork() -> isize {
    use crate::process::{current_process, manager::add_process};
    let current_proc = current_process().unwrap();
    let new_proc = current_proc.fork();
    let new_pid = new_proc.getpid();
    let trap_frame = new_proc.get_trap_frame();
    trap_frame.rax = 0; // Child process return value is 0
    add_process(new_proc);
    new_pid as isize
}
pub fn sys_exec(app_name: *const u8) -> isize {
    use crate::loader::get_app_data_by_name;
    use crate::process::{current_process, manager::add_process};
    let length: usize;
    let mut p = app_name as usize;
    loop {
        if unsafe { *(p as *const u8) } == 0 {
            length = p - app_name as usize;
            break;
        }
        p += 1;
    }
    let path_slice = unsafe { core::slice::from_raw_parts(app_name, length) };
    let path = core::str::from_utf8(path_slice).unwrap();
    if let Some(data) = get_app_data_by_name(path) {
        let proc = current_process().unwrap();
        proc.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_yield() -> isize {
    use crate::process::suspend_current_and_run_next;
    suspend_current_and_run_next();
    0
}
