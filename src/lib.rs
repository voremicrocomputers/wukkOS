#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]

use alloc::format;
use alloc::sync::Arc;
use allocator::*;

extern crate alloc;
extern crate rlibc;

use alloc::vec::Vec;
use spin::Mutex;
use core::arch::asm;
use core::ops::Deref;
use lazy_static::lazy_static;
use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::*;
use crate::serial::{Port, potential_serial_ports, terminal_helpers, terminal::ST};

mod font;
mod serial;
mod internals;
mod allocator;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(internals::errors::breakpoint_exception);
        idt
    };
}


const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern fn kernel_main() -> ! {
    // initialise serial
    let mut serial_ports = serial::init_serial();
    let mut console_port = None;
    for (i, enabled) in serial_ports.ports_enabled.iter().enumerate() {
        if *enabled {
            console_port = Some(i);
        }
    }

    if let Some(i) = console_port {
        let port = &serial_ports.ports[i];
        ST.init_from_port(*port);
    }

    ST.logln("");
    ST.logln("");
    ST.logln("");
    ST.logln("welcome to wukkOS!");
    ST.logln("(c) 2022 Real Microsoft, LLC");

    loop {}
}