use crate::task::{stop_current_and_run_next, TaskStatus, TASK_MANAGER};
use x86_64::{
    registers,
    structures::paging::{mapper::TranslateResult, OffsetPageTable, Translate},
    VirtAddr,
};

use crate::memory::{physical_memory_offset, PAGE_SIZE};

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        1 => sys_write(args[0] as *const u8, args[1]),
        2 => sys_exit(args[0] as isize),
        _ => panic!("Unsupported system call."),
    }
}

pub fn sys_write(buffer: *const u8, len: usize) -> isize {
    use crate::print;
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    let str = core::str::from_utf8(slice).unwrap();
    print!("{}", str);
    len as isize
}

pub fn sys_exit(exit_code: isize) -> ! {
    use crate::println;
    println!("[kernel] Task exited with return code {}.", exit_code);
    stop_current_and_run_next();
    panic!("sys_exit never returns!");
}

pub fn trap_init() {
    use crate::gdt::{Selectors, GDT};
    use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star};
    let (
        _,
        Selectors {
            kernel_code_seg,
            kernel_data_seg,
            user_code_seg,
            user_data_seg,
            ..
        },
    ) = *GDT;
    let mut efer_flags = Efer::read();
    efer_flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
    unsafe {
        Efer::write(efer_flags);
        Star::write(
            user_code_seg,
            user_data_seg,
            kernel_code_seg,
            kernel_data_seg,
        );
        LStar::write(x86_64::VirtAddr::new(trap_handler as u64))
    };
}

/*
#[naked]
extern "C" fn trap_handler() {
    asm!(
            "push r11",
            "push r10",
            "push r9",
            "push r8",
            "push rdi",
            "push rsi",
            "push rdx",
            "push rcx",
            "push rax",
            "sub rsp, 0x40"
    )

}
*/
#[naked]
fn trap_handler() {
    unsafe {
        asm!(
            "sub rsp, 0x28",
            "mov [rsp+0x20], rcx",  // Save user rip
            "mov [rsp+0x18], rcx",  // Save user rip
            "mov [rsp+0x10], rdx",  // 3th arg
            "mov [rsp+0x8], rsi",   // 2rd arg
            "mov [rsp], rdi",       // 1st arg
            "mov rdi, rax",         // system call id
            "mov rsi, rsp",
        );
        //Call syscall
        asm!(
            "call {}", in (reg) syscall as u64
        );
        asm!(
            "add rsp, 0x18",
            "pop rcx",
            "sysret"
        );
    }
}
