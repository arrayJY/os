#![no_std]
#![no_main]
#![feature(exclusive_range_pattern)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga;
use core::panic::PanicInfo;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0x0f4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    // println!("Running {} test", tests.len());
    serial_println!("Running {} tests", tests.len());
    tests.iter().for_each(|&test| test());
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trival_assertion() {
    // print!("Trival assertion.");
    serial_print!("trivial assertion");
    assert_eq!(1, 1);
    serial_print!(" -> ");
    serial_println!("[ok]");
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    #[cfg(test)]
    test_main();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!(" -> [failed]");
    serial_println!("Error: {}\n", _info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
