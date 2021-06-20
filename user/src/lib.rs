#![feature(asm)]
#![feature(linkage)]
#![no_std]
#![feature(panic_info_message)]
#![feature(global_asm)]

mod syscall;
pub mod console;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

use console::*;
use syscall::*;

pub fn write(buffer: &[u8]) -> isize {
    sys_write(buffer)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn fork() -> isize { sys_fork() }

pub fn exec(path: &str) -> isize { sys_exec(path) }

pub fn yield_() -> isize { sys_yield() }

pub fn wait(exit_code_ptr: &mut isize) -> isize { sys_waitpid(-1, exit_code_ptr as *mut isize) }

pub fn waitpid(pid: usize, exit_code_ptr: *mut isize) -> isize { sys_waitpid(pid as isize, exit_code_ptr) }

pub fn read(buffer: &mut [u8]) -> isize { sys_read(buffer) }

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message().unwrap();
    if let Some(location) = panic_info.location() {
        println!("Panicked at {}:{}, {}", location.file(), location.line(), err);
    } else {
        println!("Panicked: {}", err);
    }
    loop {}
}
