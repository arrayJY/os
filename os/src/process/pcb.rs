use super::{kernel_stack::KernelStack, pid::PidHandle};
use crate::memory::memory_set::MemorySet;
use crate::process::pid::alloc_pid;
use crate::process::ProcessorInner;
use crate::system_call::TrapFrame;
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::{Mutex, MutexGuard};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessStatus {
    Ready,
    Running,
    Zombie,
}

#[repr(C)]
pub struct ProcessContext {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    rbx: usize,
    rbp: usize,
    rip: usize,
}

impl ProcessContext {
    pub fn return_from_trap() -> Self {
        use crate::system_call::trap_ret;
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            rbx: 0,
            rbp: 0,
            rip: trap_ret as usize,
        }
    }
}

pub struct ProcessControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    inner: Mutex<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub memory_set: MemorySet,
    pub process_status: ProcessStatus,
    pub process_context_ptr: usize,
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: isize,
}

impl ProcessControlBlock {
    pub fn inner_lock(&self) -> MutexGuard<ProcessControlBlockInner> {
        self.inner.lock()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn get_trap_frame(&self) -> &'static mut TrapFrame {
        unsafe {
            &mut *((self.kernel_stack.get_top() - core::mem::size_of::<TrapFrame>())
                as *mut TrapFrame)
        }
    }
    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_set, user_stack, entry_point) = MemorySet::from_elf(elf_data);
        let pid = alloc_pid();
        let kernel_stack = KernelStack::new(&pid);
        // Push trap frame
        let mut trap_frame = TrapFrame::new();
        trap_frame.rsp = user_stack as u64; // User stack
        trap_frame.rcx = entry_point as u64; // Return address from syscall
        trap_frame.r11 = 0x203; // RFlags
        kernel_stack.push_to_top(trap_frame, 0);
        // Push process context
        let process_context_ptr = kernel_stack.push_to_top(
            ProcessContext::return_from_trap(),
            core::mem::size_of::<TrapFrame>(),
        );
        let task_control_block = Self {
            pid,
            kernel_stack,
            inner: Mutex::new(ProcessControlBlockInner {
                memory_set,
                process_status: ProcessStatus::Ready,
                process_context_ptr: process_context_ptr as usize,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
            }),
        };
        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        let mut inner = self.inner_lock();
        inner.memory_set.remove_all_areas();
        let (user_stack, entry_point) = inner.memory_set.read_elf(elf_data);
        let trap_frame = self.get_trap_frame();
        trap_frame.rsp = user_stack as u64; // User stack
        trap_frame.rcx = entry_point as u64; // Return address from syscall
        trap_frame.r11 = 0x203; // RFlags
    }
    pub fn fork(self: &Arc<ProcessControlBlock>) -> Arc<ProcessControlBlock> {
        use crate::println;
        let mut parent_inner = self.inner_lock();
        let memory_set = MemorySet::from(&parent_inner.memory_set);
        let pid = alloc_pid();
        let kernel_stack = KernelStack::new(&pid);
        let trap_frame_size = core::mem::size_of::<TrapFrame>();
        let process_context_ptr =
            kernel_stack.push_to_top(ProcessContext::return_from_trap(), trap_frame_size);
        let parent_trap_frame = self.get_trap_frame();
        kernel_stack.push_to_top(parent_trap_frame.clone(), 0);
        let process_control_block = Arc::new(ProcessControlBlock {
            pid,
            kernel_stack,
            inner: Mutex::new(ProcessControlBlockInner {
                memory_set,
                process_status: ProcessStatus::Ready,
                process_context_ptr: process_context_ptr as usize,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                exit_code: 0,
            }),
        });
        parent_inner.children.push(process_control_block.clone());
        process_control_block
    }
}

impl ProcessControlBlockInner {
    pub fn process_status(&self) -> ProcessStatus {
        self.process_status
    }
    pub fn get_process_context_ptr2(&self) -> *const usize {
        &self.process_context_ptr as *const usize
    }
    pub fn is_zombie(&self) -> bool {
        self.process_status == ProcessStatus::Zombie
    }
}
