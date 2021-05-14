mod lib;
use lib::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        1 => sys_write(args[0] as *const u8, args[1]),
        2 => sys_exit(args[0] as isize),
        _ => panic!("Unsupported system call."),
    }
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


#[naked]
fn trap_handler() {
    use crate::task::TASK_MANAGER;
    unsafe {
        asm!(
        "sub rsp, 0x48",
        "mov [rsp+0x40], rsp",  // stack pointer
        "mov [rsp+0x38], rcx",  // save user rip
        "mov [rsp+0x30], rdx",  // 3th arg
        "mov [rsp+0x28], rsi",  // 2rd arg
        "mov [rsp+0x20], rdi",  // 1st arg
        "mov [rsp+0x18], rax",  // system call id
        "mov rbx, rsp",
        );
        asm!(
        "mov rsp, {}",
        "mov rax, [rbx+0x40]",
        "push rax", // stack pointer
        "mov rax, [rbx+0x38]",
        "push rax", // user rip
        "mov rax, [rbx+0x30]",
        "push rax", // 3th arg
        "mov rax, [rbx+0x28]",
        "push rax", // 2rd arg
        "mov rax, [rbx+0x20]",
        "push rax", // 1st arg
        "mov rdi, [rbx+0x18]",
        "mov rsi, rsp", // system call id
        in(reg) TASK_MANAGER.current_task_kernel_stack()
        );
        //Call syscall
        asm!(
        "call {}", in (reg) syscall as u64
        );
        asm!(
        "mov rcx, [rsp+0x18]",
        "mov rbx, [rsp+0x20]",
        "add rsp, 0x20",
        "mov rsp, rbx",
        "add rsp, 0x48",
        "sysretq"
        );
    }
}
