#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use os::gdt;
use os::{exit_qemu, serial_print, serial_println, QemuExitCode};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow");
    gdt::init();
    test_idt_init();
    stack_overflow();
    panic!(" -> [failed]: not panic after stack overflow.");
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(&0u8).read();
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(gdt::ISTIndex::DoubleFault as u16);
        }
        idt
    };
}
fn test_idt_init() {
    TEST_IDT.load();
}

#[allow(dead_code)]
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!(" -> [ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
