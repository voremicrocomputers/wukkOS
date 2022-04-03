[bits 16]
[org 0x7c00]


xor ax, ax
mov ds, ax
mov es,ax

cli
mov ss,bx
mov sp,ax
sti

; set graphics mode
mov ax, 00h
mov ah, 0x00
int 0x10

; jump after bootloader
jmp 0x07E0:0x0000

times 510-($-$$) db 0
dw 0xAA55