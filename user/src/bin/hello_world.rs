#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::write;

#[no_mangle]
unsafe fn main() -> i32 {
    write("Hello World!\n".as_bytes());
    0
}
