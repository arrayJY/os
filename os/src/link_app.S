
    .align 4
    .section .data
    .global _num_app
_num_app:
    .quad 1
    .quad app_0_start
    .quad app_0_end

    .section .data
    .global app_0_start
    .global app_0_end
    .align 4
app_0_start:
    .incbin "../user/target/x86_64-os/release/hello_world"
app_0_end:
