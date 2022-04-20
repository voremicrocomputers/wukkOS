#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]

use allocator::*;

extern crate alloc;

use alloc::vec::Vec;
use spin::Mutex;
use core::arch::asm;
use lazy_static::lazy_static;
use bootloader::{entry_point, BootInfo, boot_info};
use core::panic::PanicInfo;
use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::*;
use crate::serial::potential_serial_ports;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

mod font;
mod serial;
mod internals;
mod allocator;

pub struct FBInfo {
    pub fb_mutex: Mutex<Vec<u8>>,
    pub fb_pixelwidth: Mutex<usize>,
    pub fb_colourtype: Mutex<u8>,
    pub fb_pitch: Mutex<usize>,
    pub fb_width: Mutex<usize>,
    pub fb_height: Mutex<usize>,
}

// THIS IS THE ONLY GLOBAL VARIABLE WE WILL EVER HAVE, MARK THIS ON MY FUCKING GRAVE
//pub static mut FRAMEBUFFER: Option<FBInfo> = None;

lazy_static! {
    static ref IDT : InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(internals::errors::breakpoint_exception);
        idt
    };

    static ref FACEBOOK : FBInfo = {
        FBInfo {
            fb_mutex: Mutex::new(Vec::new()),
            fb_pixelwidth: Mutex::new(0),
            fb_colourtype: Mutex::new(0),
            fb_pitch: Mutex::new(0),
            fb_width: Mutex::new(0),
            fb_height: Mutex::new(0),
        }
    };
}


const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

fn KernelPanic(msg: KernelError, fb: &mut FrameBuffer) {
    // cover the screen in red
    for y in 0..fb.info().vertical_resolution {
        for x in 0..fb.info().horizontal_resolution {
            put_pixel(x, y, COMMUNIST_RED, fb);
        }
    }
}

pub fn put_pixel(x: usize, y: usize, color: Colour, fb: &mut FrameBuffer) {
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

pub fn draw_box(x: usize, y: usize, width: usize, height: usize, color: Colour, fb: &mut FrameBuffer) {
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

pub fn draw_string(x: usize, y: usize, s: &str, color: Colour, fb: &mut FrameBuffer) {
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

pub fn draw_horizcentre_string(y: usize, s: &str, color: Colour, fb: &mut FrameBuffer) {
    let mut x_tmp = (fb.info().horizontal_resolution - s.len() * 8) / 2;
    let mut y_tmp = y;

    for c in s.chars() {
        if c == '\n' {
            x_tmp = (fb.info().horizontal_resolution - s.len() * 8) / 2;
            y_tmp += 8;
        } else {
            draw_char(x_tmp, y_tmp, c, color, fb);
            x_tmp += 8;
        }
    }
}

pub fn draw_rainbow_string(x: usize, y: usize, s: &str, fb: &mut FrameBuffer) {
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
        // set up the framebuffer
        unsafe {
            let mut colourtype: u8;
            if framebuffer.info().pixel_format == PixelFormat::RGB {
                colourtype = 1;
            } else {
                colourtype = 0;
            }
            let mut temp_fb = FACEBOOK.fb_mutex.lock();
            *temp_fb = Vec::from_raw_parts(framebuffer.buffer_mut().as_mut_ptr(), framebuffer.buffer_mut().len(), framebuffer.buffer_mut().len());
            drop(temp_fb);

            let mut temp_fb_pixel_width = FACEBOOK.fb_pixelwidth.lock();
            *temp_fb_pixel_width = framebuffer.info().bytes_per_pixel;
            drop(temp_fb_pixel_width);

            let mut temp_fb_fb_colourtype = FACEBOOK.fb_colourtype.lock();
            *temp_fb_fb_colourtype = colourtype;
            drop(temp_fb_fb_colourtype);

            let mut temp_fb_fb_width = FACEBOOK.fb_width.lock();
            *temp_fb_fb_width = framebuffer.info().horizontal_resolution;
            drop(temp_fb_fb_width);

            let mut temp_fb_fb_height = FACEBOOK.fb_height.lock();
            *temp_fb_fb_height = framebuffer.info().vertical_resolution;
            drop(temp_fb_fb_height);

            let mut temp_fb_fb_pitch = FACEBOOK.fb_pitch.lock();
            *temp_fb_fb_pitch = framebuffer.info().stride * framebuffer.info().bytes_per_pixel;
            drop(temp_fb_fb_pitch);
        }
        // cover the screen in a nice blue
        draw_box(0, 0, framebuffer.info().horizontal_resolution, framebuffer.info().vertical_resolution, Colour{r:30,g:129,b:176}, framebuffer);

        let fb_width = framebuffer.info().horizontal_resolution;
        let fb_height = framebuffer.info().vertical_resolution;

        IDT.load();

        // draw a test string
        //draw_string(20, 20, "i love drinking cum\nnewline test", Colour { r: 255, g: 0, b: 255 }, framebuffer);
        //draw_rainbow_string(20, 40, "gay sex", framebuffer);

        //draw_string(20,20, "),:\n\n\n\nuh oh! windows error! your computer is not compatible with windows 12\n\ncontact billgate@realmicrosoft.com to fix this issue!", Colour { r: 255, g: 255, b: 255}, framebuffer);

        x86_64::instructions::interrupts::int3();
        /*

        draw_horizcentre_string(((fb_height/2)-4)-16, "welcome to windows 12! here is info:", CUM_WHITE, framebuffer);

        // time for some funny com port stuff
        let serial_ports = serial::init_serial();
        draw_horizcentre_string(((fb_height/2)-4)-8, "serial ports:", CUM_WHITE, framebuffer);

        for port in 0..serial_ports.ports_enabled.len() {
            if serial_ports.ports_enabled[port] {
                draw_horizcentre_string(((fb_height/2)-4)+(port as usize*8), serial_ports.ports[port].base.to_string(), CUM_WHITE, framebuffer);
            } else { // draw in grey
                draw_horizcentre_string(((fb_height/2)-4)+(port as usize*8), serial_ports.ports[port].base.to_string(), COMMUNIST_RED, framebuffer);
            }
        }


        //draw_rainbow_string((fb_width/2) - ((7*8)/2), (fb_height/2) - 4, "gay sex", framebuffer);

         */
    }


    loop{}
}