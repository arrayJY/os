use crate::process::pcb::ProcessControlBlock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::Mutex;

pub struct ProcessManager {
    ready_queue: VecDeque<Arc<ProcessControlBlock>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, process: Arc<ProcessControlBlock>) {
        self.ready_queue.push_back(process);
    }
    pub fn fetch(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref PROCESS_MANAGER: Mutex<ProcessManager> = Mutex::new(ProcessManager::new());
}

pub fn add_process(process: Arc<ProcessControlBlock>) {
    PROCESS_MANAGER.lock().add(process);
}
pub fn fetch_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESS_MANAGER.lock().fetch()
}
