#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate rlibc;
extern crate alloc;

use alloc::rc::Rc;
use alloc::vec;
use core::arch::asm;
use lazy_static::lazy_static;
use core::panic::PanicInfo;
use multiboot2::MemoryAreaType;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::segmentation::{CS, Segment, SS};
use x86_64::structures::paging::Translate;
use crate::boot::KernelInfo;
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::*;
use crate::memory::{FRAME_ALLOC, MEM_MAPPER};
use crate::serial::terminal::ST;

mod font;
mod serial;
mod internals;
mod security;
mod boot;
mod memory;
mod macros;

lazy_static! {
    static ref GDT: Mutex<GlobalDescriptorTable> = {
        let mut gdt = GlobalDescriptorTable::new();
        Mutex::new(gdt)
    };
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(internals::errors::breakpoint_exception);
        idt.double_fault.set_handler_fn(internals::errors::double_fault);
        idt.page_fault.set_handler_fn(internals::errors::page_fault);
        idt[40].set_handler_fn(internals::cpu::keyboard_irq);
        idt
    };
}


const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

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

    // temporarily disable interrupts
    x86_64::instructions::interrupts::disable();
    println!("debug: setup GDT");
    // load TSS
    static mut tss: TaskStateSegment = TaskStateSegment::new();
    {
        unsafe {
            tss.interrupt_stack_table[0] = {
                const STACK_SIZE: usize = 4096 * 5;
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                let stack_end = stack_start + STACK_SIZE;
                stack_end
            };
            // set word at offset 102 to 0x68 and last two bytes of the tss to 0xffff
            // this is a hack to make the tss valid
            let tss_ptr = &tss as *const TaskStateSegment as *mut u8;
            unsafe {
                *tss_ptr.add(102) = 0x68;
                *tss_ptr.add(104) = 0xff;
                *tss_ptr.add(105) = 0xff;
            }
        }
        let kcs = GDT.lock().add_entry(Descriptor::kernel_code_segment());
        let kds = GDT.lock().add_entry(Descriptor::kernel_data_segment());
        let tsss = unsafe { GDT.lock().add_entry(Descriptor::tss_segment(&tss)) };
        // load GDT
        unsafe {
            GDT.lock().load_unsafe();
        }
        println!("debug: GDT loaded");
        // set code segment to kernel code segment
        unsafe {
            CS::set_reg(kcs);
        }
        println!("debug: CS set");
        // set data segment to kernel data segment
        unsafe {
            SS::set_reg(kds);
        }
        println!("debug: SS set");
        // load TSS
        unsafe {
            x86_64::instructions::tables::load_tss(tsss);
        }
        println!("debug: TSS loaded");

        // load IDT
        IDT.load();
        println!("debug: IDT loaded");
        // enable interrupts
        x86_64::instructions::interrupts::enable();
    }


    println!();
    println!();
    println!();
    println!("welcome to wukkOS!");
    println!("(c) 2022 Real Microsoft, LLC");
    let kern_info = Mutex::new(KernelInfo::init_from_kernel_args(args));

    // memory stuff
    {
        print!("initialising mapper...");
        MEM_MAPPER.lock().replace(unsafe { memory::init(VirtAddr::new(0)) });
        println!("[OK]");
        print!("initialising frame allocator...");
        FRAME_ALLOC.lock().replace(unsafe { memory::BootInfoFrameAllocator::init(kern_info) });
        println!("[OK]");
        print!("initialising heap...");
        memory::allocator::init_heap(MEM_MAPPER.lock().as_mut().unwrap(), FRAME_ALLOC.lock().as_mut().unwrap()).expect("heap init failed");
        println!("[OK]");

        print!("testing heap...");
        let reference_counted = Rc::new(vec![1, 2, 3]);
        let cloned = reference_counted.clone();
        let test_1 = Rc::strong_count(&reference_counted) == 2;
        drop(cloned);
        let test_2 = Rc::strong_count(&reference_counted) == 1;
        if test_1 && test_2 {
            println!("[OK]");
        } else {
            println!("[FAIL]");
        }
        drop(reference_counted);
    }

    // apic stuff
    {
        print!("checking for apic compatibility...");
        let apic_compatible = unsafe { internals::cpu::check_apic_compat() };
        if apic_compatible {
            println!("[OK]");
        } else {
            println!("[FAIL]");
            panic!("apic required at the moment");
        }
        print!("disabling PIC...");
        unsafe { internals::cpu::disable_pic() };
        println!("[OK]");
        print!("initialising apic...");
        unsafe { internals::cpu::enable_apic() };
        println!("[OK]");
        print!("setting up apic interrupts...");
        unsafe { internals::cpu::setup_apic_interrupts(kern_info.lock().acpi_get_ioapic_addr()) };
        println!("[OK]");
        // enable interrupts
        x86_64::instructions::interrupts::enable();
    }

    loop {
    }
}