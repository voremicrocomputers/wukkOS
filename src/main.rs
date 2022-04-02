#!no_std

fn print_vga_buffer(string : &str) {
    // get address of VGA buffer
    let mut vga_buffer = 0xb8000 as *mut u8;

    // get length of string
    let len = string.len();

    // write string to VGA buffer
    for i in 0..len {
        unsafe {
            *vga_buffer = string.as_bytes()[i];
            vga_buffer = vga_buffer.offset(2);
        }
    }
}


fn main() {
    print_vga_buffer("microsoft");
}