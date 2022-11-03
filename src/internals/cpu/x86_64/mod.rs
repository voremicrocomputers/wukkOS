
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PhysAddr, set_general_handler, VirtAddr};
use x86_64::registers::segmentation::{CS, Segment, SS};
use x86_64::structures::paging::Translate;
use spin::Mutex;
use crate::{internals, println};

use lazy_static::lazy_static;

pub mod apic;


lazy_static! {
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
            idt[internals::cpu::x86_64::apic::TIMER_IRQ].set_handler_fn(internals::cpu::x86_64::apic::timer).set_stack_index(1);
            idt[internals::cpu::x86_64::apic::ERROR_IRQ].set_handler_fn(internals::cpu::x86_64::apic::error).set_stack_index(1);
            idt[internals::cpu::x86_64::apic::SPURIOUS_IRQ].set_handler_fn(internals::cpu::x86_64::apic::spurious).set_stack_index(1);
            idt[internals::cpu::x86_64::apic::FALLBACK_KEYBOARD_IRQ].set_handler_fn(internals::cpu::x86_64::apic::keyboard_irq).set_stack_index(1);
        }
        idt
    };
}


pub fn init() {
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
}