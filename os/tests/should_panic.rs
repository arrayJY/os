#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    //Some operations should panic.
    serial_print!("1 == 0");
    assert_eq!(1, 0);
    serial_println!(" -> [not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!(" -> [ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
