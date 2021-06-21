#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fork, exec, yield_, waitpid};
use user_lib::console::getchar;

const LF: u8 = '\n' as u8;

#[no_mangle]
unsafe fn main() -> i32 {
    print!(">> ");
    let mut line = [0u8; 256];
    let mut len = 0;
    loop {
        let mut c = getchar();
        match c {
            LF => {
                println!("");
                line[len] = 0;
                let path = core::str::from_utf8(&line[..=len]).unwrap();
                let pid = fork();
                if pid == 0 {
                    if exec(path) == -1 {
                        println!("Error when executing!");
                        return -1;
                    };
                    unreachable!();
                } else {
                    let mut exit_code: isize = 0;
                    let exit_pid = waitpid(pid as usize, &mut exit_code);
                    assert_eq!(pid, exit_pid);
                    println!("[shell] Process {} exited with code {}.", pid, exit_code);
                }
                len = 0;
                print!(">> ");
            }
            _ => {
                line[len] = c;
                len += 1;
                print!("{}", c as char)
            }
        }
    }
    0
}
