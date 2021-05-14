use x86_64::{
    registers::control::Cr3Flags,
    structures::paging::{OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate},
    PhysAddr, VirtAddr,
};

use crate::task::{run_first, Task};

pub fn task_page_table_address(current_page_table: &OffsetPageTable, task: &mut Task) -> usize {
    let lv4_table: *const PageTable = task.memory_set.page_table.level_4_table();
    let page_table_phys_addr = current_page_table
        .translate_addr(VirtAddr::new(lv4_table as u64))
        .unwrap();
    page_table_phys_addr.as_u64() as usize
}

pub fn user_init() {
    run_first();
}

#[naked]
pub unsafe extern "C" fn jump_to_user_space(
    _user_stack: usize,
    _user_entry_point: usize,
    _user_page_table: usize,
) {
    asm!(
        "mov cr3, rdx", // Switch page table
        "push 0x1b",    // User data segment
        "push rdi",     // User space stack
        "push 0x202",   // RFlags
        "push 0x23",    // User code segment
        "push rsi",     // User entry point
        "iretq",
        options(noreturn)
    );
}
