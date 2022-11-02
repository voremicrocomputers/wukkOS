#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#![feature(const_slice_from_raw_parts_mut)]
#![no_std]
#![no_main]

extern crate rlibc;
extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::lazy_static;
use core::panic::PanicInfo;
use libfar::farlib;
use libfar::farlib::{FarArchive, FarFileInfo};
use limine::{LimineBootInfoRequest, LimineMemmapRequest, LimineTerminalRequest};
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PhysAddr, set_general_handler, VirtAddr};
use x86_64::registers::segmentation::{CS, Segment, SS};
use x86_64::structures::paging::Translate;
use crate::boot::{get_initwukko, get_ioapic_info, KERNEL_ADDRESS};
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

pub type InitWukko = FarArchive;

lazy_static! {
    //pub static ref KERN_INFO: Mutex<Option<KernelInfo>> = Mutex::new(None);
    static ref GDT: Mutex<GlobalDescriptorTable> = {
        let mut gdt = GlobalDescriptorTable::new();
        Mutex::new(gdt)
    };
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            use internals::errors::unhandled;
            set_general_handler!(&mut idt, unhandled);
            idt.breakpoint.set_handler_fn(internals::errors::breakpoint_exception).set_stack_index(0);
            idt.double_fault.set_handler_fn(internals::errors::double_fault).set_stack_index(0);
            idt.page_fault.set_handler_fn(internals::errors::page_fault).set_stack_index(0);
            idt[internals::cpu::TIMER_IRQ].set_handler_fn(internals::cpu::timer).set_stack_index(1);
            idt[internals::cpu::ERROR_IRQ].set_handler_fn(internals::cpu::error).set_stack_index(1);
            idt[internals::cpu::SPURIOUS_IRQ].set_handler_fn(internals::cpu::spurious).set_stack_index(1);
            idt[internals::cpu::FALLBACK_KEYBOARD_IRQ].set_handler_fn(internals::cpu::keyboard_irq).set_stack_index(1);
        }
        idt
    };

    static ref INITWUKKO: Mutex<Option<InitWukko>> = Mutex::new(None);
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
        println!("panic msg: {}", msg)
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

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    debug!("entry point");

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
            {
                tss.interrupt_stack_table[0] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.interrupt_stack_table[1] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.interrupt_stack_table[2] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.interrupt_stack_table[3] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.interrupt_stack_table[4] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.interrupt_stack_table[5] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.interrupt_stack_table[6] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
            }
            {
                tss.privilege_stack_table[0] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.privilege_stack_table[1] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
                tss.privilege_stack_table[2] = {
                    const STACK_SIZE: usize = 4096 * 5;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                    let stack_end = stack_start + STACK_SIZE;
                    stack_end
                };
            }
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

    // memory stuff
    {
        print!("initialising mapper...");
        let kernel_physical_address = KERNEL_ADDRESS.get_response().get().unwrap().physical_base;
        let kernel_virtual_address = KERNEL_ADDRESS.get_response().get().unwrap().virtual_base;
        debug!("kernel physical address: {:#x}", kernel_physical_address);
        debug!("kernel virtual address: {:#x}", kernel_virtual_address);
        let offset = (kernel_virtual_address as i64) as usize;// - kernel_physical_address as i64) as usize;
        MEM_MAPPER.lock().replace(unsafe { memory::init(VirtAddr::new(0)) });
        println!("[OK]");
        print!("initialising frame allocator...");
        FRAME_ALLOC.lock().replace(unsafe { memory::BootInfoFrameAllocator::init() });
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
        print!("initialising apic...");
        //internals::cpu::tell_pic8259a_to_f_off();
        let (addr, isos) = get_ioapic_info();
        unsafe { internals::cpu::enable_apic() };
        println!("[OK]");
        print!("setting up apic interrupts...");
        debug!("ioapicaddr: {:#x}", addr);
        unsafe { internals::cpu::setup_ioapic(addr, isos) };
        println!("[OK]");
        // enable interrupts
        //x86_64::instructions::interrupts::enable();
    }

    // initwukko stuff
    {
        print!("loading initwukko...");
        let initwukko_raw = get_initwukko();
        let ar = farlib::test(&initwukko_raw).expect("invalid initwukko");
        let ar = ar.load_file_data(&initwukko_raw);
        let mut initwukko_magic = None;
        for entry in &ar.file_data {
            let entry_name = &entry.name;
            if entry_name == "magic.wukk" {
                initwukko_magic = Some(entry.data.clone());
                debug!("magic: {}", String::from_utf8_lossy(&entry.data));
                break;
            }
        }
        const CORRECT_MAGIC: &[u8; 24] = b"WUKKOS_COMPLIANT_RAMDISK";
        if initwukko_magic.as_ref().unwrap_or(&vec![])[..CORRECT_MAGIC.len()] != CORRECT_MAGIC[..] {
            debug!("initwukko magic: {:?}", initwukko_magic);
            println!("[FAIL]");
            panic!("invalid initwukko");
        }
        println!("[OK]");
    }


    loop {
        x86_64::instructions::hlt();
    }
}