pub mod kernel_stack;
pub mod manager;
pub mod pcb;
pub mod pid;
pub mod switch;

use crate::loader::{get_app_data, get_app_data_by_name};
use crate::process::manager::{add_process, fetch_process, ProcessManager};
use crate::process::pcb::{ProcessControlBlock, ProcessStatus};
use crate::process::switch::{switch_mm, switch_to};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cell::RefCell;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::paging::mapper::{TranslateResult, MappedFrame};
use x86_64::structures::paging::{OffsetPageTable, PageTable, PageTableIndex, Translate, PhysFrame};
use x86_64::VirtAddr;
use crate::system_call::TrapFrame;

pub struct Processor {
    inner: RefCell<ProcessorInner>,
}

pub struct ProcessorInner {
    current: Option<Arc<ProcessControlBlock>>,
    idle_process_context_ptr: usize,
    idle_page_table: OffsetPageTable<'static>,
}

impl Processor {
    pub fn new() -> Self {
        use crate::memory::current_offset_page_table;
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_process_context_ptr: 0,
                idle_page_table: unsafe { current_offset_page_table() },
            }),
        }
    }

    fn get_idle_process_context_ptr(&self) -> usize {
        self.inner.borrow().idle_process_context_ptr
    }

    fn get_idle_process_context_ptr2(&self) -> *const usize {
        let inner = self.inner.borrow();
        &inner.idle_process_context_ptr as *const usize
    }

    pub fn run(&self) {
        use switch::switch_to;
        loop {
            if let Some(process) = fetch_process() {
                let idle_task_cx_ptr2 = self.get_idle_process_context_ptr2();
                let mut process_inner = process.inner_lock();
                let next_process_context = process_inner.process_context_ptr;

                let page_table = {
                    let page_table = &mut process_inner.memory_set.page_table;
                    let page_table_virt =
                        VirtAddr::new(page_table.level_4_table() as *mut PageTable as u64);
                    let paget_table_phys= page_table.translate_addr(page_table_virt).unwrap();
                    PhysFrame::containing_address(paget_table_phys)
                };
                process_inner.process_status = ProcessStatus::Running;
                drop(process_inner);
                self.inner.borrow_mut().current = Some(process);

                unsafe {
                    use x86_64::registers::control::Cr3;
                    let (_, flags) = Cr3::read();
                    Cr3::write(page_table, flags);
                    // switch_mm(page_table);
                    switch_to(idle_task_cx_ptr2, next_process_context);
                }
            }
        }
    }

    pub fn current(&self) -> Option<Arc<ProcessControlBlock>> {
        self.inner.borrow().current.as_ref().cloned()
    }

    pub fn take_current(&self) -> Option<Arc<ProcessControlBlock>> {
        self.inner.borrow_mut().current.take()
    }
}

unsafe impl Sync for Processor {}

lazy_static! {
    pub static ref PROCESSOR: Processor = Processor::new();
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = Arc::new(ProcessControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

pub fn add_initproc() {
    use manager::add_process;
    add_process(INITPROC.clone());
}

pub fn run_processes() {
    PROCESSOR.run()
}

pub fn take_current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.take_current()
}
pub fn current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.current()
}

pub fn suspend_current_and_run_next() {
    let process = take_current_process().unwrap();
    let mut inner = process.inner_lock();
    let task_context_ptr2 = inner.get_process_context_ptr2();
    inner.process_status = ProcessStatus::Ready;
    drop(inner);
    add_process(process);
    schedule(task_context_ptr2);
}

pub fn exit_current_and_run_next(exit_code: isize) {
    let process = take_current_process().unwrap();
    let mut inner = process.inner_lock();
    inner.process_status = ProcessStatus::Zombie;
    inner.exit_code = exit_code;
    {
        let mut initproc_inner = INITPROC.inner_lock();
        for child in inner.children.iter() {
            child.inner_lock().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    inner.children.clear();
    drop(inner);
    drop(process);
    let _unused: usize = 0;
    schedule(&_unused as *const _);
}

pub fn schedule(switched_process_context_ptr2: *const usize) {
    let idle_process_context_ptr = PROCESSOR.get_idle_process_context_ptr();
    unsafe {
        switch_to(switched_process_context_ptr2, idle_process_context_ptr);
    }
}

pub fn current_kernel_stack() -> usize {
    let c = PROCESSOR.current().unwrap();
    c.kernel_stack.get_top()
}
