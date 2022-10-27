use core::arch::asm;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::OffsetPageTable;
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

pub fn set_apic_base(apic: usize) {
    const IA32_APIC_BASE_MSR_ENABLE: u32 = 0x800;
    let mut edx = 0u32;
    let mut eax = (apic & 0xfffff0000) as u32 | IA32_APIC_BASE_MSR_ENABLE;
    unsafe {
        asm!("wrmsr",
            in("ecx") 0x1b,
            in("eax") eax,
            in("edx") edx,
        );
    }
}

pub fn get_apic_base() -> usize {
    let mut edx = 0u32;
    let mut eax = 0u32;
    unsafe {
        asm!("rdmsr",
            in("ecx") 0x1b,
            lateout("eax") eax,
            lateout("edx") edx,
        );
    }
    ((edx as usize) << 32) | (eax as usize)
}

pub fn enable_apic(mem_mapper: &mut OffsetPageTable, frame_alloc: &mut BootInfoFrameAllocator) {
    // PIC should be disabled by now
    // now enable local apic
    // 1. set bit 8 of spurious interrupt vector register
    let sivr_addr = 0xfee000f0;
    let sivr = read_phys_memory32(mem_mapper, frame_alloc, sivr_addr);
    write_phys_memory32(mem_mapper, frame_alloc, sivr_addr, sivr | (1 << 8));
}

pub fn apic_read_io(mem_mapper: &mut OffsetPageTable, frame_alloc: &mut BootInfoFrameAllocator, ioapicaddr: usize, reg: u32) -> u32 {
    write_phys_memory32(mem_mapper, frame_alloc, ioapicaddr as u32, reg);
    read_phys_memory32(mem_mapper, frame_alloc, ioapicaddr as u32 + 0x10)
}

pub fn apic_write_io(mem_mapper: &mut OffsetPageTable, frame_alloc: &mut BootInfoFrameAllocator, ioapicaddr: usize, reg: u32, val: u32) {
    write_phys_memory32(mem_mapper, frame_alloc, ioapicaddr as u32, reg);
    write_phys_memory32(mem_mapper, frame_alloc, ioapicaddr as u32 + 0x10, val);
}

pub fn disable_pic() {
    command(0x20, 0x11);
    command(0xa0, 0x11);

    command(0x21, 0xe0);
    command(0xa1, 0xe8);

    command(0x21, 0x04);
    command(0xa1, 0x02);

    command(0x21, 0x01);
    command(0xa1, 0x01);

    command(0x21, 0xff);
    command(0xa1, 0xff);
}

pub fn ioapic_set_irq(mem_mapper: &mut OffsetPageTable, frame_alloc: &mut BootInfoFrameAllocator, ioapicaddr: usize, irq: u8, apic_id: u64, vector:u8) {
    let lo_index: u32 = (0x10 + irq*2    ) as u32;
    let hi_index: u32 = (0x10 + irq*2 + 1) as u32;

    let mut high = apic_read_io(mem_mapper, frame_alloc, ioapicaddr, hi_index);
    // set apic id
    high &= !(0xff000000);
    high |= (apic_id as u32) << 24;
    apic_write_io(mem_mapper, frame_alloc, ioapicaddr, hi_index, high);

    let mut low = apic_read_io(mem_mapper, frame_alloc, ioapicaddr, lo_index);

    // unmask
    low &= !(1 << 16);
    // set to physical delivery
    low &= !(1 << 11);
    // set to fixed delivery
    low &= !(0x700);
    // set vector
    low &= !(0xff);
    low |= vector as u32;

    apic_write_io(mem_mapper, frame_alloc, ioapicaddr, lo_index, low);
}

pub fn apic_eoi() {
    unsafe {
        asm!("mov eax, 0",
            "mov ecx, 0xb0",
            "wrmsr",
            in("ecx") 0x80b,
            in("eax") 0,
        );
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
pub fn setup_apic_interrupts(mem_mapper: &mut OffsetPageTable, frame_alloc: &mut BootInfoFrameAllocator, ioapicaddr: usize) {
    // set keyboard irq to interrupt 40
    ioapic_set_irq(mem_mapper, frame_alloc, ioapicaddr, 1, 0, 40);
}