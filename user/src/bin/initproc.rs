#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{fork, exec, yield_};

#[no_mangle]
unsafe fn main() -> i32 {
    if fork() == 0 {
        exec("hello_world\0");
    } else {
        loop {
            yield_();
        }
    }
    0
}
