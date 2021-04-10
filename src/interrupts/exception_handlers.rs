use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
macro_rules! def_handler_func {
    ($name: tt, $text: expr) => {
        pub extern "x86-interrupt" fn $name(stack_frame: &mut InterruptStackFrame) {
            panic!("EXCEPTION: {}\n{:#?}", $text, stack_frame);
        }
    };
}
macro_rules! def_handler_func_with_errorcode {
    ($name: tt, $text: expr) => {
        pub extern "x86-interrupt" fn $name(
            stack_frame: &mut InterruptStackFrame,
            error_code: u64,
        ) {
            panic!(
                "EXCEPTION: {}\n{:#?}\nErrorCode: {:x}",
                $text, stack_frame, error_code
            );
        }
    };
}

def_handler_func!(divide_error_handler, "Divide Error");
def_handler_func!(debug_handler, "Debug");
def_handler_func!(overflow_handler, "Overflow");
def_handler_func!(device_not_available_handler, "Device Not Available");
def_handler_func!(non_maskable_interrupt_handler, "Non-maskable Interrupt");
def_handler_func!(bound_range_exceeded_handler, "Bound Range Exceeded");
def_handler_func!(invalid_opcode_handler, "Invalid Opcode");
def_handler_func!(x87_floating_point_handler, "x87 Floating Point");
def_handler_func!(simd_floating_point_handler, "SIMD Floating Point");
def_handler_func!(virtualization_handler, "Virtualization");

def_handler_func_with_errorcode!(invalid_tss_handler, "Invalid TSS");
def_handler_func_with_errorcode!(alignment_check_handler, "Alignment Check");
def_handler_func_with_errorcode!(segment_not_present_handler, "Segment not Present");
def_handler_func_with_errorcode!(stack_segment_fault_handler, "Stack Segment Fault");
def_handler_func_with_errorcode!(security_handler, "Security");
def_handler_func_with_errorcode!(general_protection_fault_handler, "General Protection Fault");

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    crate::println!("EXCEPTION: Breakpoint\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: Double Fault\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn machine_check_handler(stack_frame: &mut InterruptStackFrame) -> ! {
    panic!("EXCEPTION: Machine Check\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!("EXCEPTION: Page Fault\n{:#?}\nErrorCode: {:#?}", stack_frame, error_code);
}
