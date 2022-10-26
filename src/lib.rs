#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![no_std]
#![no_main]

extern crate rlibc;

use lazy_static::lazy_static;
use core::panic::PanicInfo;
use multiboot2::MemoryAreaType;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::VirtAddr;
use crate::boot::KernelInfo;
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::*;
use crate::memory::active_level_4_table;
use crate::serial::terminal::ST;

mod font;
mod serial;
mod internals;
mod security;
mod boot;
mod memory;
mod macros;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(internals::errors::breakpoint_exception);
        idt.double_fault.set_handler_fn(internals::errors::double_fault);
        idt.page_fault.set_handler_fn(internals::errors::page_fault);
        idt
    };
}


const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("---KERNEL FUCKY WUKKY UWU (panic)---");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("panic payload: {s:?}")
    } else {
        println!("no panic payload")
    };
   if let Some(msg) = info.message() {
        println!("panic msg: {}", msg.as_str().unwrap_or("no message"))
    } else {
        println!("no message");
    }
    if let Some(location) = info.location() {
        println!("location: file {} line {}", location.file(), location.line());
    } else {
        println!("no location");
    };
    loop {}
}

#[repr(C)]
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
        println!("using serial port {} as console", i);
    }


    println!();
    println!();
    println!();
    println!("welcome to wukkOS!");
    println!("(c) 2022 Real Microsoft, LLC");
    print!("initialising memory maps...");
    let kern_info = KernelInfo::init_from_kernel_args(args);
    let mut mem_areas = kern_info.memory_areas();
    println!("[OK]");
    let l4_table = active_level_4_table(VirtAddr::new(0));

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 entry {}: {:?}", i, entry);
        }
    }

    loop {}
}