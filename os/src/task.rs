use core::cell::RefCell;

use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;

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

pub fn run_first() {
    TASK_MANAGER.run_task(0)
}
pub fn stop_current_and_run_next() {
    TASK_MANAGER.stop_current();
    TASK_MANAGER.run_next();
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

unsafe impl Sync for TaskManager {}

pub struct TaskManager {
    task_num: usize,
    inner: RefCell<InnerTaskManager>,
    page_table: OffsetPageTable<'static>,
}
pub struct InnerTaskManager {
    pub tasks: Vec<Task>,
    pub current_task: usize,
}

impl TaskManager {
    fn run_task(&self, id: usize) {
        use crate::exec::jump_to_user_space;
        use crate::gdt::{Selectors, GDT};
        use x86_64::instructions::segmentation::{load_ds, load_es};

        let mut user_stack: usize = 0;
        let mut user_entry_point: usize = 0;
        let mut user_page_table: usize = 0;
        {
            let mut inner = self.inner.borrow_mut();
            let task = &mut inner.tasks[id];
            user_stack = task.trap_context.user_stack;
            user_entry_point = task.trap_context.entry_point;
            user_page_table = task.memory_set.page_table_address(&self.page_table);
        }
        // task.task_status = TaskStatus::Running;
        unsafe {
            let (_, Selectors { user_data_seg, .. }) = *GDT;
            load_ds(user_data_seg);
            load_es(user_data_seg);
            jump_to_user_space(user_stack, user_entry_point, user_page_table);
        }
    }

    fn stop_current(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.current_task_mut().task_status = TaskStatus::Stop;
    }
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        (current + 1..current + inner.tasks.len() + 1)
            .map(|id| id % inner.tasks.len())
            .find(|&id| inner.tasks[id].task_status == TaskStatus::Ready)
    }
    fn run_next(&self) {
        if let Some(next) = self.find_next_task() {
            self.run_task(next)
        } else {
            panic!("All applications completed!");
        }
    }
}

impl InnerTaskManager {
    pub fn current_task(&self) -> &Task {
        &self.tasks[self.current_task]
    }
    pub fn current_task_mut(&mut self) -> &mut Task {
        &mut self.tasks[self.current_task]
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        TaskManager {
            task_num: get_app_num(),
            inner: {
                use crate::memory;
                use x86_64::VirtAddr;
                RefCell::new(InnerTaskManager {
                    tasks: {
                        let mut v = Vec::new();
                        for i in 0..get_app_num() {
                            v.push(Task::new(get_app_data(i), i));
                        }
                        v
                    },
                    current_task: 0,
                })
            },
            page_table: unsafe {
                use crate::memory::{self, physical_memory_offset};
                memory::init(VirtAddr::new(physical_memory_offset()))
            },
        }
    };
}
