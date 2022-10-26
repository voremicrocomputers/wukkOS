section .multiboot_header
header_start:
    align 4
    ; multiboot 2 magic
    dd 0xe85250d6
    dd 0x00000000
    dd header_end - header_start
    ; checksum
    dd 0x100000000 - (0xe85250d6 + 0x00000000 + (header_end - header_start))

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size

header_end: