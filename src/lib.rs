#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;
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
use crate::boot::KernelInfo;
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::*;
use crate::serial::{Port, potential_serial_ports, terminal_helpers, terminal::ST};

mod font;
mod serial;
mod internals;
mod allocator;
mod security;
mod boot;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(internals::errors::breakpoint_exception);
        idt
    };
}


const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ST.logln("---KERNEL FUCKY WUKKY UWU (panic)---");
    ST.logln(if let Some(s) = info.payload().downcast_ref::<&str>() {
        format!("panic payload: {s:?}")
    } else {
        format!("no panic payload")
    }.as_str());
    ST.logln(if let Some(msg) = info.message() {
        format!("panic msg: {}", msg.as_str().unwrap_or("no message"))
    } else {
        "no message".to_string()
    }.as_str());
    ST.logln(if let Some(location) = info.location() {
        format!("location: file {} line {}", location.file(), location.line())
    } else {
        "no location".to_string()
    }.as_str());
    loop {}
}

pub struct KernelArgs {
    #[cfg(feature = "f_multiboot2")]
    multiboot_information_address: usize
}

#[no_mangle]
pub extern fn kernel_main(args: KernelArgs) -> ! {
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

    let kern_info: Box<dyn boot::KernelInfo> = {
        #[cfg(feature = "f_multiboot2")]
        {
            let mut kern_info = boot::multiboot2::Multiboot2Bootloader::default();
            kern_info.init_from_kernel_args(args);
            Box::new(kern_info)
        }
    };

    ST.logln("");
    ST.logln("");
    ST.logln("");
    ST.logln("welcome to wukkOS!");
    ST.logln("(c) 2022 Real Microsoft, LLC");
    ST.log("initialising memory maps...");
    let mem_areas = kern_info.get_memory_areas();
    ST.logln("[OK]");
    ST.logln("memory map:");
    for area in mem_areas {
        ST.logln(format!("{:x} - {:x} : {}", area.start, area.end, match area.area_type {
            boot::MemoryType::Available => "Available",
            boot::MemoryType::Reserved => "Reserved",
            boot::MemoryType::AcpiReclaimable => "ACPI Reclaimable",
            boot::MemoryType::Nvs => "NVS",
            boot::MemoryType::BadMemory => "Bad Memory",
            boot::MemoryType::Kernel => "Kernel",
            boot::MemoryType::Bootloader => "Bootloader",
            boot::MemoryType::Unknown(_) => "Unknown"
        }).as_str());
    }

    loop {}
}