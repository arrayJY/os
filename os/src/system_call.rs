mod lib;
use lib::*;

#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        1 => sys_write(args[0] as *const u8, args[1]),
        2 => sys_exit(args[0] as isize),
        3 => sys_fork(),
        4 => sys_exec(args[0] as *const u8),
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
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rsp: u64,
}

impl TrapFrame {
    pub fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rbp: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp: 0,
        }
    }
}

global_asm!(include_str!("system_call/trap.S"));

extern "C" {
    #[inline]
    fn trap_start();
    #[inline]
    pub fn trap_ret();
}

#[no_mangle]
pub fn current_kernel_stack() -> usize {
    use crate::process::current_kernel_stack;
    current_kernel_stack()
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
