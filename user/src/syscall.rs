#[repr(usize)]
pub enum SystemCall {
    SysWrite = 1,
    SysExit,
    SysYield,
    SysFork,
    SysExec,
    SysWaitPID,
}

impl SystemCall {
    pub fn as_u64(self) -> u64 {
        self as u64
    }
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

pub fn sys_write(buffer: &[u8]) -> isize {
    unsafe {
        system_call(
            SystemCall::SysWrite,
            buffer.as_ptr() as usize, buffer.len(), 0,
        )
    }
}

pub fn sys_exit(exit_code: i32) -> isize {
    unsafe { system_call(SystemCall::SysExit, exit_code as usize, 0, 0) }
}

pub fn sys_yield() -> isize { unsafe { system_call(SystemCall::SysYield, 0, 0, 0) } }

pub fn sys_fork() -> isize {
    unsafe { system_call(SystemCall::SysFork, 0, 0, 0) }
}

pub fn sys_exec(path: &str) -> isize {
    unsafe { system_call(SystemCall::SysExec, path.as_ptr() as usize, 0, 0) }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut isize) -> isize {
    unsafe { system_call(SystemCall::SysWaitPID, pid as usize, exit_code_ptr as usize, 0) }
}


global_asm!("\
.globl system_call
system_call:
    movq %rdi, %rax
    movq %rsi, %rdi
    movq %rdx, %rsi
    movq %rcx, %rdx
    leaq 0x2(%rip), %rcx
    syscall
    retq
");

/*
    movl (%rsp), %ecx
    addq $0x4, %rsp
    movl %ecx, (%rsp)
*/

extern {
    fn system_call(syscall_id: SystemCall, arg0: usize, arg1: usize, arg2: usize) -> isize;
}

/*
unsafe extern "C" fn system_call(syscall_id: SystemCall, args: [usize; 3]) -> isize {
    let id = syscall_id.as_usize();
    let mut ret;
    unsafe {
        asm!("mov rdx, {}", in(reg) args[2]);
        asm!("mov rsi, {}", in(reg) args[1]);
        asm!("mov rdi, {}", in(reg) args[0]);
        asm!("mov rax, {}", in(reg) id);
        // asm!("int 0x80");
        asm!("syscall");
        asm!("mov {}, rax", out(reg) ret);
    }
    ret
}
 */
