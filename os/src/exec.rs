pub fn user_init() {
    use crate::gdt::{Selectors, GDT};
    use x86_64::instructions::segmentation::{load_ds, load_es};
    //TODO: Init user_stack pointer and user_entry_point pointer;
    let user_stack = 0u64;
    let user_entry_point = 0u64;
    unsafe {
        let (_, Selectors { user_data_seg, .. }) = *GDT;
        load_ds(user_data_seg);
        load_es(user_data_seg);
        jump_to_user_space(user_stack, user_entry_point);
    }
}

#[naked]
unsafe extern "C" fn jump_to_user_space(_user_stack: u64, _user_entry_point: u64) {
    asm!(
        "push 0x1b",  // User data segment
        "push rdi",   // User space stack
        "push 0x202", // RFlags
        "push 0x23",  // User code segment
        "push rsi",   // User entry point
        "iretq",
        options(noreturn)
    );
}
