use crate::memory::memory_set::KERNEL_SPACE;
use crate::process::pid::PidHandle;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

#[derive(Debug, Clone)]
pub struct KernelStack {
    pid: usize,
}

fn kernel_stack_address(app_id: usize) -> (u64, u64) {
    use crate::memory::{GUARD_SIZE, KERNEL_STACK_END, KERNEL_STACK_SIZE};
    let top = KERNEL_STACK_END - (app_id as u64) * (KERNEL_STACK_SIZE + GUARD_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

impl KernelStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;

        let (bottom, top) = kernel_stack_address(pid);
        KERNEL_SPACE.lock().insert(
            VirtAddr::new(bottom),
            VirtAddr::new(top),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            None,
        );
        Self { pid }
    }
    pub fn get_top(&self) -> usize {
        let (_, top) = kernel_stack_address(self.pid);
        top as usize
    }

    pub fn push_to_top<T>(&self, data: T, offset: usize) -> *mut T
    where
        T: Sized,
    {
        let top = self.get_top() - offset;
        let ptr = (top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *ptr = data;
        }
        ptr
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (bottom, _) = kernel_stack_address(self.pid);
        KERNEL_SPACE
            .lock()
            .remove_area_with_start_addr(VirtAddr::new(bottom));
    }
}
