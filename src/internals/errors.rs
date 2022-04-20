
use core::borrow::{BorrowMut};


use crate::{InterruptStackFrame, font, FACEBOOK};
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::{COMMUNIST_RED, CUM_WHITE, Colour};

fn draw_box(x: usize, y: usize, width: usize, height: usize, color: Colour, mut fb: &mut [u8]) {
    let pixelwidth = FACEBOOK.fb_pixelwidth.lock();
    let pitch = FACEBOOK.fb_pitch.lock();
    let colourtype = FACEBOOK.fb_colourtype.lock();

    for i in 0..width {
        for j in 0..height {
            let pixel = (y * &*pitch) + (x * &*pixelwidth) + (i * &*pixelwidth) + (j * &*pitch);
            if *colourtype == 0 { //BGR
                unsafe {
                    fb[pixel + 0] = color.b;
                    fb[pixel + 1] = color.g;
                    fb[pixel + 2] = color.r;
                }
            } else if *colourtype == 1 { //RGB
                unsafe {
                    fb[pixel + 0] = color.r;
                    fb[pixel + 1] = color.g;
                    fb[pixel + 2] = color.b;
                }
            } else {
                // average values
                let avg = (color.r as u16 + color.g as u16 + color.b as u16) / 3;
                unsafe {fb[pixel + 0] = avg as u8;}
            }
        }
    }

    drop(pixelwidth);
    drop(pitch);
    drop(colourtype);
}

fn put_pixel(x: usize, y: usize, color: Colour, mut fb: &mut [u8]) {
    let pixelwidth = FACEBOOK.fb_pixelwidth.lock();
    let pitch = FACEBOOK.fb_pitch.lock();
    let colourtype = FACEBOOK.fb_colourtype.lock();

    let pixel = (y * &*pitch) + (x * &*pixelwidth);

    if *colourtype == 0 { //BGR
        unsafe {
            fb[pixel + 0] = color.b;
            fb[pixel + 1] = color.g;
            fb[pixel + 2] = color.r;
        }
    } else if *colourtype == 1 { //RGB
        unsafe {
            fb[pixel + 0] = color.r;
            fb[pixel + 1] = color.g;
            fb[pixel + 2] = color.b;
        }
    } else {
        // average values
        let avg = (color.r as u16 + color.g as u16 + color.b as u16) / 3;
        unsafe {fb[pixel + 0] = avg as u8;}
    }

    drop(pixelwidth);
    drop(pitch);
    drop(colourtype);
}


fn draw_char(x: usize, y: usize, c: char, color: Colour, mut fb: &mut [u8]) {
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

fn draw_horizcentre_string(width: usize, y: usize, s: &str, color: Colour, fb: &mut [u8]) {
    let mut x_tmp = (width - s.len() * 8) / 2;
    let mut y_tmp = y;

    for c in s.chars() {
        if c == '\n' {
            x_tmp = (width - s.len() * 8) / 2;
            y_tmp += 8;
        } else {
            draw_char(x_tmp, y_tmp, c, color, fb.borrow_mut());
            x_tmp += 8;
        }
    }
}

pub extern "x86-interrupt" fn breakpoint_exception(stack_frame: InterruptStackFrame) {
    // cover the screen in a nice communist red (:
    let mut fb = FACEBOOK.fb_mutex.lock();
    let fb_width = FACEBOOK.fb_width.lock();
    let fb_height = FACEBOOK.fb_height.lock();

    draw_box(0,0,*fb_width,*fb_height, COMMUNIST_RED, fb.borrow_mut());
    // draw our funny text
    draw_horizcentre_string(*fb_width,(*fb_height / 2) - (14 * (8/2)), "OOPSY WOOPSY, THE KERNEL HAD A FUCKY WUCKY UWU", CUM_WHITE, fb.borrow_mut());
    draw_horizcentre_string(*fb_width,(*fb_height / 2) - (10 * (8/2)), "WHOEVER WAS PROGRAMMING THE KERNEL DECIDED TO LEAVE A BREAKPOINT IN IT, OOPS (:", CUM_WHITE, fb.borrow_mut());
    draw_horizcentre_string(*fb_width,(*fb_height / 2) - (4 * (8/2)), "THE KERNEL IS NOW HALTED, YOU CAN'T DO ANYTHING UNTIL YOU RESTART THE KERNEL", CUM_WHITE, fb.borrow_mut());

    drop(fb_width);
    drop(fb_height);
    drop(fb);
}