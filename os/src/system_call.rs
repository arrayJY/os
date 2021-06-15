mod lib;
use lib::*;

#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        1 => sys_write(args[0] as *const u8, args[1]),
        2 => sys_exit(args[0] as isize),
        3 => sys_fork(),
        // 4 => sys_exec(),
        // 5 => sys_wait(),
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
        )
        .unwrap();
        LStar::write(x86_64::VirtAddr::new(trap_start as u64))
    };
}

#[repr(C)]
pub struct TrapFrame {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rbp: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rsp: u64,
}

global_asm!(include_str!("system_call/trap.S"));

extern "C" {
    #[inline]
    fn trap_start();
    #[inline]
    fn trap_ret();
}

#[no_mangle]
pub fn current_kernel_stack() -> usize {
    use crate::task::TASK_MANAGER;
    TASK_MANAGER.current_task_kernel_stack() as usize
}


#[no_mangle]
fn trap_syscall(trap_frame: &TrapFrame) -> isize {
    syscall(
        trap_frame.rax as usize, //syscall id
        [
            trap_frame.rdi as usize, // arg 1
            trap_frame.rsi as usize, // arg 2
            trap_frame.rdx as usize, // arg 3
        ],
    )
}
