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
use core::panic::PanicInfo;
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::*;
use crate::serial::potential_serial_ports;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

mod font;
mod serial;
mod internals;
mod allocator;

// THIS IS THE ONLY GLOBAL VARIABLE WE WILL EVER HAVE, MARK THIS ON MY FUCKING GRAVE
//pub static mut FRAMEBUFFER: Option<FBInfo> = None;

lazy_static! {
    static ref IDT : InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(internals::errors::breakpoint_exception);
        idt
    };
}


const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

fn KernelPanic(msg: KernelError, fb: &mut FrameBuffer) {
    // cover the screen in red
}

#[no_mangle]
fn kernel_main() -> ! {

    loop{}
}