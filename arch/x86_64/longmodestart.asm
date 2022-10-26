global long_mode_start
extern kernel_main

section .text
bits 64
long_mode_start:
    mov ax, 0x00
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    extern kernel_main
    call kernel_main

    hlt