#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo, boot_info};
use core::panic::PanicInfo;
use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, PixelFormat};

mod font;


#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy)]
struct Colour {
    r: u8,
    g: u8,
    b: u8,
}



const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn put_pixel(x: usize, y: usize, color: Colour, fb: &mut FrameBuffer) {
    let pixelwidth = fb.info().bytes_per_pixel;
    let pitch = fb.info().stride * pixelwidth;

    let pixel = (y * pitch) + (x * pixelwidth);

    if fb.info().pixel_format == PixelFormat::BGR {
        fb.buffer_mut()[pixel + 0] = color.b;
        fb.buffer_mut()[pixel + 1] = color.g;
        fb.buffer_mut()[pixel + 2] = color.r;
    } else if fb.info().pixel_format == PixelFormat::RGB {
        fb.buffer_mut()[pixel + 0] = color.r;
        fb.buffer_mut()[pixel + 1] = color.g;
        fb.buffer_mut()[pixel + 2] = color.b;
    } else {
        // average values
        let avg = (color.r as u16 + color.g as u16 + color.b as u16) / 3;
        fb.buffer_mut()[pixel + 0] = avg as u8;
    }
}

fn draw_box(x: usize, y: usize, width: usize, height: usize, color: Colour, fb: &mut FrameBuffer) {
    let pixelwidth = fb.info().bytes_per_pixel;
    let pitch = fb.info().stride * pixelwidth;

    for i in 0..width {
        for j in 0..height {
            let pixel = (y * pitch) + (x * pixelwidth) + (i * pixelwidth) + (j * pitch);
            if fb.info().pixel_format == PixelFormat::BGR {
                fb.buffer_mut()[pixel + 0] = color.b;
                fb.buffer_mut()[pixel + 1] = color.g;
                fb.buffer_mut()[pixel + 2] = color.r;
            } else if fb.info().pixel_format == PixelFormat::RGB {
                fb.buffer_mut()[pixel + 0] = color.r;
                fb.buffer_mut()[pixel + 1] = color.g;
                fb.buffer_mut()[pixel + 2] = color.b;
            } else {
                // average values
                let avg = (color.r as u16 + color.g as u16 + color.b as u16) / 3;
                fb.buffer_mut()[pixel + 0] = avg as u8;
            }
        }
    }
}

fn draw_char(x: usize, y: usize, c: char, color: Colour, fb: &mut FrameBuffer) {
    let font = font::BASIC_LEGACY;
    // font is 8x8, stored in a 2d array of bytes
    let char_width = 8;
    let char_height = 8;

    let char_index = c as usize;
    let char_data = font[char_index];

    for row in 0..char_height {
        for col in 0..char_width {
            let bit = (char_data[row] >> col) & 1;
            if bit >= 1 {
                put_pixel(x + col, y + row, color, fb);
            }
        }
    }
}

fn draw_string(x: usize, y: usize, s: &str, color: Colour, fb: &mut FrameBuffer) {
    let mut x_tmp = x;
    let mut y_tmp = y;

    for c in s.chars() {
        if c == '\n' {
            x_tmp = x;
            y_tmp += 8;
        } else {
            draw_char(x_tmp, y_tmp, c, color, fb);
            x_tmp += 8;
        }
    }
}

fn draw_rainbow_string(x: usize, y: usize, s: &str, fb: &mut FrameBuffer) {
    let mut x_tmp = x;
    let mut y_tmp = y;

    let mut i = 0;

    for c in s.chars() {
        if c == '\n' {
            x_tmp = x;
            y_tmp += 8;
        } else {
            let color = RAINBOW[i % RAINBOW.len() as usize];
            draw_char(x_tmp, y_tmp, c, color, fb);
            x_tmp += 8;
            i += 1;
        }
    }
}

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        // cover the screen in a nice blue
        draw_box(0, 0, framebuffer.info().horizontal_resolution, framebuffer.info().vertical_resolution, Colour{r:30,g:129,b:176}, framebuffer);

        let fb_width = framebuffer.info().horizontal_resolution;
        let fb_height = framebuffer.info().vertical_resolution;

        // draw a test string
        //draw_string(20, 20, "i love drinking cum\nnewline test", Colour { r: 255, g: 0, b: 255 }, framebuffer);
        //draw_rainbow_string(20, 40, "gay sex", framebuffer);

        draw_string(20,20, "),:\n\n\n\nuh oh! windows error! your computer is not compatible with windows 12\n\ncontact billgate@realmicrosoft.com to fix this issue!", Colour { r: 255, g: 255, b: 255}, framebuffer);

        draw_rainbow_string((fb_width/2) - ((7*8)/2), (fb_height/2) - 4, "gay sex", framebuffer);

    }

    loop{}
}