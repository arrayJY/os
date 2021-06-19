.globl trap_start
trap_start:
    pushq %r15
    pushq %r14
    pushq %r13
    pushq %r12
    pushq %r11
    pushq %r10
    pushq %r9
    pushq %r8
    pushq %rdi
    pushq %rsi
    pushq %rbp
    pushq %rdx
    pushq %rcx
    pushq %rbx
    pushq %rax
    movq  %rsp, %rbx
    callq current_kernel_stack
    movq %rax, %rsp     #switch to kernel stack
    leaq 0x78(%rbx), %rbx
    pushq %rbx          #save user space pointer
    movq -0x08(%rbx), %rax
    pushq %rax
    movq -0x10(%rbx), %rax
    pushq %rax
    movq -0x18(%rbx), %rax
    pushq %rax
    movq -0x20(%rbx), %rax
    pushq %rax
    movq -0x28(%rbx), %rax
    pushq %rax
    movq -0x30(%rbx), %rax
    pushq %rax
    movq -0x38(%rbx), %rax
    pushq %rax
    movq -0x40(%rbx), %rax
    pushq %rax
    movq -0x48(%rbx), %rax
    pushq %rax
    movq -0x50(%rbx), %rax
    pushq %rax
    movq -0x58(%rbx), %rax
    pushq %rax
    movq -0x60(%rbx), %rax
    pushq %rax
    movq -0x68(%rbx), %rax
    pushq %rax
    movq -0x70(%rbx), %rax
    pushq %rax
    movq -0x78(%rbx), %rax
    pushq %rax
    movq  %rsp, %rdi
    call trap_handler

.globl trap_ret
trap_ret:
    popq %rax
    popq %rbx
    popq %rcx
    popq %rdx
    popq %rbp
    popq %rsi
    popq %rdi
    popq %r8
    popq %r9
    popq %r10
    popq %r11
    popq %r12
    popq %r13
    popq %r14
    popq %r15
    popq %rsp
    sysretq

.globl trap_handler
trap_handler:
    call trap_syscall
    add $0x10, %rsp
    pushq %rax # save return value on the stack
    jmp trap_ret
