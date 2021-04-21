#[repr(usize)]
pub enum SystemCall {
    SysWrite = 1,
    SysExit = 2,
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
    system_call(
        SystemCall::SysWrite,
        [buffer.as_ptr() as usize, buffer.len(), 0],
    )
}

pub fn sys_exit(exit_code: i32) -> isize {
    system_call(SystemCall::SysExit, [exit_code as usize, 0, 0])
}

pub fn write(buffer: &[u8]) -> isize {
    let ptr = buffer.as_ptr() as usize;
    let len = buffer.len();
    let args = [ptr, len, 0];
    system_call(SystemCall::SysWrite, args)
}

fn system_call(syscall_id: SystemCall, args: [usize; 3]) -> isize {
    let id = syscall_id.as_usize();
    unsafe {
        asm!("mov rdx, {}", in(reg) args[2], options(nostack));
        asm!("mov rsi, {}", in(reg) args[1], options(nostack));
        asm!("mov rdi, {}", in(reg) args[0], options(nostack));
        asm!("mov rax, {}", in(reg) id, options(nostack));
        asm!("int 0x80");
    }
    1 //TODO: Dealing with return value
}
