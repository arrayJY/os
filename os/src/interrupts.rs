pub mod exception_handlers;
use crate::gdt::ISTIndex;
use crate::print;
use exception_handlers::*;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Interrupt {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Trap = 0x80,
}

impl Interrupt {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    pub fn as_usize(self) -> usize {
        self as usize
    }
    pub fn end_of_interrupt(self) {
        unsafe { PIC.lock().notify_end_of_interrupt(self.as_u8()) }
    }
}

pub static PIC: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        unsafe {
            idt.debug
                .set_handler_fn(debug_handler)
                .set_stack_index(ISTIndex::Debug.as_u16());
            idt.non_maskable_interrupt
                .set_handler_fn(non_maskable_interrupt_handler)
                .set_stack_index(ISTIndex::NonMaskableInterrupt.as_u16());
        };
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available
            .set_handler_fn(device_not_available_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(ISTIndex::DoubleFault.as_u16());
        }
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point
            .set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point
            .set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.security_exception.set_handler_fn(security_handler);
        idt[Interrupt::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[Interrupt::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt
    };
}

extern "x86-interrupt" fn timer_handler(_stack_frame: &mut InterruptStackFrame) {
    // print!(".");
    Interrupt::Timer.end_of_interrupt();
}

use alloc::vec::Vec;
lazy_static! {
    pub static ref STDIN_BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());
}
extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore) //Ignore control now.
        );
    };

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let code: u8 = unsafe { port.read() };
    let mut lock = STDIN_BUFFER.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(code) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(ch) => lock.push(ch as u8),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
    Interrupt::Keyboard.end_of_interrupt();
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pic() {
    unsafe {
        PIC.lock().initialize();
    }
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}
