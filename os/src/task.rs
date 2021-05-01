use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::paging::OffsetPageTable;

use crate::{loader::*, memory::memory_set::MemorySet};
use alloc::vec::Vec;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Stop,
}

pub struct TrapContext {
    pub user_stack: usize,
    pub entry_point: usize,
}

pub struct Task {
    pub id: usize,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub trap_context: TrapContext,
}

impl Task {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_stack, entry_point) = MemorySet::from_elf(elf_data);
        Self {
            id: app_id,
            task_status: TaskStatus::Ready,
            memory_set,
            trap_context: TrapContext {
                user_stack,
                entry_point,
            },
        }
    }
}
pub struct TaskManager {
    pub tasks: Vec<Task>,
    pub current_task: usize,
}

impl TaskManager {
    pub fn get_mut(&mut self) -> &mut Self {
        self
    }
    pub fn current_task(&self) -> &Task {
        &self.tasks[self.current_task]
    }
    pub fn current_task_mut(&mut self) -> &mut Task {
        &mut self.tasks[self.current_task]
    }
    pub fn current_task_page_table(&self) -> &OffsetPageTable {
        &self.current_task().memory_set.page_table
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = {
        Mutex::new(TaskManager {
            tasks: {
                let mut v = Vec::new();
                for i in 0..get_app_num() {
                    v.push(Task::new(get_app_data(i), i));
                }
                v
            },
            current_task: 0,
        })
    };
}
