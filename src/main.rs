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
use spin::Mutex;
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::*;

mod font;
mod serial;
mod internals;
mod security;
mod boot;
mod memory;
mod macros;

pub type InitWukko = FarArchive;

lazy_static! {
    static ref INITWUKKO: Mutex<Option<InitWukko>> = Mutex::new(None);
}


const RAINBOW : [Colour; 6] = [Colour{r:255,g:0,b:0}, Colour{r:255,g:127,b:0}, Colour{r:255,g:255,b:0}, Colour{r:0,g:255,b:0}, Colour{r:0,g:255,b:255}, Colour{r:0,g:0,b:255}];

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic_wrapper(info: &PanicInfo) -> ! {
    panic(info)
}

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

#[cfg(target_arch = "powerpc")]
use ieee1275::prom_init;
#[cfg(target_arch = "powerpc")]
use ieee1275::services::Args;

#[cfg(target_arch = "powerpc")]
#[no_mangle]
#[link_section = ".text"]
pub extern "C" fn _start(_r3: u32, _r4: u32, entry: extern "C" fn(*mut Args) -> usize) -> isize {
    use internals::cpu::ppc32::PROMHNDL;
    {
        let mut prom = PROMHNDL.lock();
        prom.set_prom(prom_init(entry));
    }

    kernel_main();

    PROMHNDL.lock().get().exit()
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // ppc32 stuff for now
    //#[cfg(target_arch = "powerpc")]
    {

    }

    debug!("entry point");

    #[cfg(target_arch = "x86_64")]
    {
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

        internals::cpu::x86_64::init();
    }


    println!();
    println!();
    println!();
    println!("welcome to wukkOS!");
    println!("(c) 2022 Real Microsoft, LLC");

    /*

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
        //apic::instructions::interrupts::enable();
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

     */

    loop {
        #[cfg(target_arch = "apic")]
        {
            x86_64::instructions::hlt();
        }
    }
}