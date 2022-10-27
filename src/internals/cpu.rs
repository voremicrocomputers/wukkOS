use core::arch::asm;
use x86::apic::{xapic::XAPIC, ioapic::IoApic, ApicControl};
use x86_64::PhysAddr;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::PhysFrame;
use crate::{debug, KERN_INFO, print, println};
use crate::memory::{BootInfoFrameAllocator, read_phys_memory32, write_phys_memory32};
use crate::serial::{command, read};

// todo! maybe abstract this into different sections for different parts of cpu func?

pub struct WAPICManager {
    pub xapic: XAPIC,
    pub id: u32,
}

pub struct WIOAPICManager {
    pub ioapic: IoApic,
}

pub fn check_apic_compat() -> bool {
    unsafe {
        let mut eax: u32;
        let mut edx: u32;
        asm!("cpuid",
            in("eax") 1,
            lateout("eax") eax,
            lateout("edx") edx,
        );
        edx & (1 << 9) != 0
    }
}

pub fn enable_apic() -> WAPICManager {
    // we need to get the xapic region
    let mut XAPIC_REGION: &'static mut [u32] = unsafe {
        // region should be FFFE0000H to FFFE0FFFH
        let region_start = 0xFFFE_0000u32;
        let region_end = 0xFFFE_0FFFu32;
        let region_size = region_end - region_start;
        let region_size = region_size as usize;
        let region_start = region_start as *mut u32;
        // read to make sure it gets mapped
        let _ = read_phys_memory32(region_start as u32);
        let _ = read_phys_memory32(region_end as u32);
        core::slice::from_raw_parts_mut(region_start, region_size)
    };
    let mut xapic = unsafe { XAPIC::new(XAPIC_REGION) };
    xapic.attach();

    // get xapic id to ensure it's working
    let id = xapic.id();
    debug!("xapic id: {}", id);
    debug!("xapic version: {}", xapic.version());
    
    WAPICManager {
        xapic,
        id,
    }
}

// todo! in the future this will be removed, it is only for testing basic apic functionality
pub extern "x86-interrupt" fn keyboard_irq(stack_frame: InterruptStackFrame) {
    debug!("keyboard interrupt");
    unsafe { asm!("iretq"); }
    let scancode = read(0x60);

    print!("ksc: {},", scancode);

    // reset keyboard controller
    let mut a = read(0x61);
    a |= 0x82;
    command(0x61, a);
    a &= 0x7f;
    command(0x61, a);
}

// todo! we should abstract this away
pub fn setup_ioapic(ioapicaddr: u32) -> WIOAPICManager {
    let mut ioapic = unsafe { IoApic::new(ioapicaddr as usize) };
    let _ = read_phys_memory32(ioapicaddr);
    // assert that supported interrupts is greater than 1
    debug!("ioapic supported interrupts: {}", ioapic.supported_interrupts());
    // setup keyboard irq (interrupt 0x40)
    ioapic.enable(1, 0x40);

    // return
    WIOAPICManager {
        ioapic,
    }
}