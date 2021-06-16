.global switch_to
switch_to:
    # %rdi = (pointer to current context: **mut ProcessContext)
    # %rsi = (target context: *mut ProcessContext)

    # save current registers
    pushq %rbp
    pushq %rbx
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    # switch stack
    movq %rsp, (%rdi)
    movq %rsi, %rsp

    # restore target registers
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %rbx
    popq %rbp

    ret

.global switch_mm
switch_mm:
    # %rdi = page table address
    mov %rdi, %cr3
