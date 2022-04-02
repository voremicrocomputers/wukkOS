[bits 16]
[org 0x7c00]


xor ax, ax
mov ds, ax

; set graphics mode
mov ax, 00h
mov ah, 0x00
int 0x10

; write string to screen
mov bh, 0
mov bl, 0b00001111
mov al, 1
mov ah, 13h
mov cx, WINDOWS_STR_END - WINDOWS_STR
mov dl, 0
mov dh, 0
push cs
pop es
mov bp, WINDOWS_STR
int 0x10



WINDOWS_STR: db 'microsoft windows', 0
WINDOWS_STR_END:

times 510-($-$$) db 0
dw 0xAA55