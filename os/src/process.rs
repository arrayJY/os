pub mod kernel_stack;
pub mod manager;
pub mod pcb;
pub mod pid;
pub mod switch;

use crate::loader::get_app_data;
use crate::process::manager::{add_process, fetch_process, ProcessManager};
use crate::process::pcb::{ProcessControlBlock, ProcessStatus};
use crate::process::switch::switch_to;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cell::RefCell;
use lazy_static::lazy_static;
use spin::Mutex;

pub struct Processor {
    inner: RefCell<ProcessorInner>,
}

pub struct ProcessorInner {
    current: Option<Arc<ProcessControlBlock>>,
    idle_process_context_ptr: usize,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_process_context_ptr: 0,
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
                process_inner.process_status = ProcessStatus::Running;
                drop(process_inner);
                self.inner.borrow_mut().current = Some(process);
                unsafe {
                    switch_to(idle_task_cx_ptr2, next_process_context);
                }
            }
        }
    }

    pub fn current(&self) -> Option<Arc<ProcessControlBlock>> {
        self.inner.borrow().current.as_ref().map(|v| Arc::clone(v))
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
    pub static ref INITPROC: Arc<ProcessControlBlock> =
        Arc::new(ProcessControlBlock::new(get_app_data(0)));
}

pub fn add_initproc() {
    use manager::add_process;
    add_process(INITPROC.clone());
}

pub fn run_processes() {
    PROCESSOR.run()
}

fn take_current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.take_current()
}

pub fn suspend_current_and_run_next() {
    let process = take_current_process().unwrap();
    let mut inner = process.inner_lock();
    let task_context_ptr2 = inner.get_process_context_ptr2();
    inner.process_status = ProcessStatus::Ready;
    drop(inner);
    add_process(process);
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
