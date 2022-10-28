use core::arch::asm;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApicBuilder, xapic_base};
use x86_64::PhysAddr;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::PhysFrame;
use crate::{debug, print, println};
use crate::memory::{BootInfoFrameAllocator, read_phys_memory32, write_phys_memory32};
use crate::serial::{command, read};

// todo! maybe abstract this into different sections for different parts of cpu func?

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

pub fn enable_apic() {
    // we need to get the xapic region
    let phys_addr = unsafe { xapic_base() };

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(0x40)
        .error_vector(0x41)
        .spurious_vector(0x42)
        .set_xapic_base(phys_addr)
        .build()
        .unwrap_or_else(|e| panic!("failed to build local apic: {}", e));

    unsafe {
        lapic.enable();
    }
}

pub extern "x86-interrupt" fn timer(stack_frame: InterruptStackFrame) {
    println!("timer interrupt");
}

pub extern "x86-interrupt" fn error(stack_frame: InterruptStackFrame) {
    println!("error interrupt");
}

pub extern "x86-interrupt" fn spurious(stack_frame: InterruptStackFrame) {
    println!("spurious interrupt");
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
pub fn setup_ioapic(ioapicaddr: u32) {
    let mut ioapic = unsafe {
        IoApic::new(ioapicaddr as u64)
    };
    // setup keyboard interrupt
    unsafe {
        // init with irq offset
        ioapic.init(0x50);
        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
        entry.set_dest(0);
        ioapic.set_table_entry(1, entry);

        ioapic.enable_irq(1);
    }

}