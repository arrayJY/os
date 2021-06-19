use super::ProcessControlBlock;
use x86_64::structures::paging::{PageTable, Translate};
use x86_64::VirtAddr;
global_asm!(include_str!("switch.S"));

pub fn switch_page_table(current_process: &ProcessControlBlock, target_process: &ProcessControlBlock) -> usize {
    let offset = crate::memory::physical_memory_offset();
    let translator = &mut current_process.inner_lock().memory_set.page_table;
    let target_page_table: *const PageTable = target_process.inner_lock().memory_set.page_table.level_4_table();
    let phys_addr = translator.translate_addr(VirtAddr::new(target_page_table as u64)).unwrap();
    (phys_addr.as_u64() + offset) as usize
}

extern "C" {
    pub fn switch_mm(page_table_addr: usize);
    pub fn switch_to(
        current_task_context: *const usize,
        target_task_context: usize,
    );
}
