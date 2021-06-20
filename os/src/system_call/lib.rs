pub fn sys_write(buffer: *const u8, len: usize) -> isize {
    use crate::print;
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    let str = core::str::from_utf8(slice).unwrap();
    print!("{}", str);
    len as isize
}


pub fn sys_read(buffer: *mut u8, len: usize) -> isize {
    assert_eq!(len, 1, "Only support read len 1.");
    use crate::process::suspend_current_and_run_next;
    use crate::interrupts::STDIN_BUFFER;
    use x86_64::instructions::interrupts::without_interrupts;
    let mut c: u8 = 0;
    loop {
        without_interrupts(|| {
            if let Some(ch) = STDIN_BUFFER.lock().pop() {
                c = ch;
            }
        });
        if c == 0 {
            suspend_current_and_run_next();
            continue;
        } else {
            break;
        }
    }
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, len) };
    buffer[0] = c;
    1
}

pub fn sys_exit(exit_code: isize) -> ! {
    use crate::println;
    use crate::process::exit_current_and_run_next;
    println!("[kernel] Task exited with return code {}.", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("sys_exit never returns!");
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

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut isize) -> isize {
    use crate::process::current_process;
    let proc = current_process().unwrap();
    let mut inner = proc.inner_lock();
    if inner
        .children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none()
    {
        return -1;
    }
    let r = inner
        .children
        .iter()
        .enumerate()
        .find(|(_, p)| p.inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid()));
    if let Some((idx, _)) = r {
        let child = inner.children.remove(idx);
        use alloc::sync::Arc;
        assert_eq!(Arc::strong_count(&child), 1);
        let pid = child.getpid();
        let exit_code = child.inner_lock().exit_code;
        unsafe { *exit_code_ptr = exit_code };
        pid as isize
    } else {
        -2
    }
}
