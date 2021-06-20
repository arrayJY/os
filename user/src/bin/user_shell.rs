#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fork, exec, yield_, wait};
use user_lib::console::getchar;

#[no_mangle]
unsafe fn main() -> i32 {
    print!(">> ");
    let mut line = [0u8; 256];
    let mut len = 0;
    loop {
        let mut c = getchar();
        match c {
            _ => {
                line[len] = c;
                len += 1;
                print!("{}", c as char)
            }
        }
    }
    0
}
