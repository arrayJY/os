use x86_64::{
    registers::control::Cr3Flags,
    structures::paging::{OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate},
    PhysAddr, VirtAddr,
};

use crate::task::Task;

pub fn task_page_table_address(current_page_table: &OffsetPageTable, task: &mut Task) -> usize {
    let lv4_table: *const PageTable = task.memory_set.page_table.level_4_table();
    let page_table_phsy_addr = current_page_table
        .translate_addr(VirtAddr::new(lv4_table as u64))
        .unwrap();
    page_table_phsy_addr.as_u64() as usize
}

pub fn user_init(page_table: &OffsetPageTable) {
    use crate::gdt::{Selectors, GDT};
    use x86_64::instructions::segmentation::{load_ds, load_es};
    use x86_64::registers::control::Cr3;

    use crate::task::TASK_MANAGER;
    let mut task_manager = TASK_MANAGER.lock();
    let userinit = task_manager.current_task_mut();
    let user_stack = userinit.trap_context.user_stack;
    let user_entry_point = userinit.trap_context.entry_point;
    let user_page_table = task_page_table_address(page_table, userinit);
    drop(task_manager);
    unsafe {
        let (_, Selectors { user_data_seg, .. }) = *GDT;
        load_ds(user_data_seg);
        load_es(user_data_seg);
        jump_to_user_space(user_stack, user_entry_point, user_page_table);
    }
}

#[naked]
unsafe extern "C" fn jump_to_user_space(
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
