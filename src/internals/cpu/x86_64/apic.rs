use alloc::vec::Vec;
use core::arch::asm;
use acpi::platform::interrupt::InterruptSourceOverride;
use lazy_static::lazy_static;
use spin::Mutex;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApic, LocalApicBuilder, xapic_base};
use x86_64::PhysAddr;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::PhysFrame;
use crate::{debug, print, println};
use crate::memory::{BootInfoFrameAllocator, read_phys_memory32, write_phys_memory32};
use crate::serial::{command, read};
use crate::serial::simplifiers::handle_scancode;

// todo! maybe abstract this into different sections for different parts of cpu func?

pub const APIC_INTERRUPT_OFFSET: usize = 32;

pub const TIMER_IRQ: usize = 0 + APIC_INTERRUPT_OFFSET;
pub const ERROR_IRQ: usize = 1 + APIC_INTERRUPT_OFFSET;
pub const SPURIOUS_IRQ: usize = 2 + APIC_INTERRUPT_OFFSET;

pub const IOAPIC_IRQ_OFFSET: usize = 42;
pub const FALLBACK_KEYBOARD_IRQ: usize = 1 + IOAPIC_IRQ_OFFSET;


lazy_static!{
    static ref LAPIC: Mutex<LocalApic> = {
        // we need to get the xapic region
        let phys_addr = unsafe { xapic_base() };

        let mut lapic = LocalApicBuilder::new()
            .timer_vector(TIMER_IRQ as usize)
            .spurious_vector(SPURIOUS_IRQ as usize)
            .error_vector(ERROR_IRQ as usize)
            .set_xapic_base(phys_addr)
            .build()
            .unwrap_or_else(|e| panic!("failed to build local apic: {}", e));
        Mutex::new(lapic)
    };
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

pub fn tell_pic8259a_to_f_off() {
    unsafe {
        asm!("cli");
        asm!("out dx, al", in("dx") 0x20, in("al") 0x11i8);
        asm!("out dx, al", in("dx") 0xA0, in("al") 0x11i8);
        asm!("out dx, al", in("dx") 0x21, in("al") 0x20i8);
        asm!("out dx, al", in("dx") 0xA1, in("al") 0x28i8);
        asm!("out dx, al", in("dx") 0x21, in("al") 0x04i8);
        asm!("out dx, al", in("dx") 0xA1, in("al") 0x02i8);
        asm!("out dx, al", in("dx") 0x21, in("al") 0x01i8);
        asm!("out dx, al", in("dx") 0xA1, in("al") 0x01i8);
        asm!("out dx, al", in("dx") 0x21, in("al") 0x0i8);
        asm!("out dx, al", in("dx") 0xA1, in("al") 0x0i8);
        asm!("sti");
    }
}

pub fn enable_apic() {
    unsafe {
        LAPIC.lock().enable();
    }
}

pub extern "x86-interrupt" fn timer(stack_frame: InterruptStackFrame) {
    end_of_interupt();
}

pub extern "x86-interrupt" fn error(stack_frame: InterruptStackFrame) {
    println!("error interrupt");
    end_of_interupt();
}

pub extern "x86-interrupt" fn spurious(stack_frame: InterruptStackFrame) {
    println!("spurious interrupt");
    end_of_interupt();
}

fn end_of_interupt() {
    unsafe {
        LAPIC.lock().end_of_interrupt();
    }
}

// todo! in the future this will be removed, it is only for testing basic apic functionality
pub extern "x86-interrupt" fn keyboard_irq(stack_frame: InterruptStackFrame) {
    let scancode = read(0x60);

    handle_scancode(scancode);

    // reset keyboard controller
    let mut a = read(0x61);
    a |= 0x82;
    command(0x61, a);
    a &= 0x7f;
    command(0x61, a);

    end_of_interupt();
}

// todo! we should abstract this away
pub fn setup_ioapic(ioapicaddr: u32, isos: Vec<InterruptSourceOverride>) {
    let mut ioapic = unsafe {
        IoApic::new(ioapicaddr as u64)
    };
    // setup keyboard interrupt
    unsafe {
        // FIXME! only for testing that this works, abstract ASAP!
        let gsi_keyboard = isos.iter().find(|iso| iso.isa_source == 1)
            .map(|iso| iso.global_system_interrupt).unwrap_or(1);
        // init with irq offset
        ioapic.init(IOAPIC_IRQ_OFFSET as u8);
        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
        entry.set_dest(0);
        entry.set_vector(gsi_keyboard as u8 + IOAPIC_IRQ_OFFSET as u8);
        ioapic.set_table_entry(1, entry);

        ioapic.enable_irq(1);
    }

}