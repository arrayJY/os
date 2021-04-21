#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
unsafe fn main() -> i32 {
    println!("Hello World!");
    0
}