#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fork, exec, yield_, wait};

#[no_mangle]
unsafe fn main() -> i32 {
    if fork() == 0 {
        exec("user_shell\0");
    } else {
        loop {
            let mut exit_code: isize = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 || pid == -2 {
                yield_();
                continue;
            }
            println!("[initproc] Released a zombie process, pid={}, exit_code={}",
                     pid, exit_code)
        }
    }
    0
}
