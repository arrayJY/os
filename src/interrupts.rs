use crate::gdt::ISTIndex;
use crate::{print, println};
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
    Keyborard,
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
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(ISTIndex::DoubleFault.as_u16());
        }
        idt[Interrupt::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[Interrupt::Keyborard.as_usize()].set_handler_fn(keyboard_handler);
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: Breakpoint\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_handler(_stack_frame: &mut InterruptStackFrame) {
    // print!(".");
    Interrupt::Timer.end_of_interrupt();
}
extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore) //Ignore control now.
        );
    };

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let code: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(code) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(ch) => print!("{}", ch),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    Interrupt::Keyborard.end_of_interrupt();
}

#[allow(unused_variables)]
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: Double Fault\n{:#?}", stack_frame);
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
