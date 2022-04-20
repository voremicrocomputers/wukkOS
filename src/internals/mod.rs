pub mod errors;

pub mod WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood {
    use core::arch::asm;
    use core::marker::PhantomData;

    #[derive(Clone, Copy)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[derive(Clone, Copy)]
    pub struct Colour {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    // colours
    pub const MICROSOFT_BLUE: Colour = Colour{r:30,g:129,b:176};
    pub const COMMUNIST_RED: Colour = Colour{r:245,g:77,b:30};
    pub const CUM_WHITE: Colour = Colour{r:255,g:255,b:255};

    #[derive(Clone, Copy)]
    pub struct IDTEntry<F> {
        based_offset: u16,
        code_selector: u16,
        ist_offset_wastes_6_bits: u8,
        attributes: u8,
        offset_popping_off: u16,
        what: PhantomData<F>
    }

    impl<F> IDTEntry<F> {
        pub fn default() -> Self {
            IDTEntry {
                based_offset: 0,
                code_selector: 0,
                ist_offset_wastes_6_bits: 0,
                attributes: 0,
                offset_popping_off: 0,
                what: PhantomData
            }
        }
    }

    macro_rules! impl_idtentry {
        ($f:ty) => {
            impl IDTEntry<$f> {
                pub fn set_handler(&mut self, handler: $f) {
                    unsafe { // no shit this is unsafe, i wrote it (:
                        let handler_addr = handler as u32;
                        self.based_offset = handler_addr as u16;
                        self.offset_popping_off = (handler_addr >> 16) as u16;
                        let code_selector : u16;
                        asm!("mov {0:x}, cs", out(reg) code_selector, options(nomem, nostack, preserves_flags));
                        self.code_selector = code_selector;
                        self.attributes = 0b10001110;
                    }
                }
            }
        };
    }


    pub struct InterruptDescriptorTable {
        pub divide_error: IDTEntry<InterruptHandler>,
        pub debug: IDTEntry<InterruptHandler>,
        pub dream_mask_sus_version: IDTEntry<InterruptHandler>, // non-maskable interrupt
        pub breakpoint: IDTEntry<InterruptHandler>,
        pub into_detected_overflow: IDTEntry<InterruptHandler>,
        pub in_the_fortnite_storm: IDTEntry<InterruptHandler>, // bound range exceeded
        pub owo_whats_this: IDTEntry<InterruptHandler>, // invalid opcode
        pub device_not_available: IDTEntry<InterruptHandler>,
        pub fucky_wucky_twice: IDTEntry<SeriousFuckUpWithErrorCodeHandler>, // double fault
        we_are_all_about_backwards_compatibility: IDTEntry<InterruptHandler>, // coprocessor segment overrun
        pub invalid_tss: IDTEntry<ErrorWithErrorCodeHandler>,
        pub segment_not_present: IDTEntry<ErrorWithErrorCodeHandler>,
        pub stack_segment_fault: IDTEntry<ErrorWithErrorCodeHandler>,
        pub uh_oh_we_gotta_hacker_here: IDTEntry<ErrorWithErrorCodeHandler>, // general protection fault
        pub page_fault: IDTEntry<ErrorWithErrorCodeHandler>,
        reserved_1: IDTEntry<InterruptHandler>, // what the fuck is this?? the only comment is "vector nr.15"
        pub x87_floating_point_exception: IDTEntry<InterruptHandler>, // compatibility B)
        pub alignment_check: IDTEntry<ErrorWithErrorCodeHandler>,
        pub machine_check: IDTEntry<SeriousFuckUpHandler>,
        pub the_point_of_the_mask_float_exception: IDTEntry<InterruptHandler>, // simd floating point exception
        pub virtualization_exception: IDTEntry<InterruptHandler>, // qemu WILL be added to windows 12 B)
        reserved_2: IDTEntry<InterruptHandler>, // another reserved
        pub vmm_communication_exception: IDTEntry<ErrorWithErrorCodeHandler>, // honestly too tired to check what this is
        pub security_exception: IDTEntry<ErrorWithErrorCodeHandler>,
        reserved_3: IDTEntry<InterruptHandler>,

        pub interrupts: [IDTEntry<InterruptHandler>; 256 - 32], // the original one didn't make this public, but who care (:
    }

    impl InterruptDescriptorTable {
        pub fn new() -> InterruptDescriptorTable {
            InterruptDescriptorTable {
                divide_error: IDTEntry::default(),
                debug: IDTEntry::default(),
                dream_mask_sus_version: IDTEntry::default(),
                breakpoint: IDTEntry::default(),
                into_detected_overflow: IDTEntry::default(),
                in_the_fortnite_storm: IDTEntry::default(),
                owo_whats_this: IDTEntry::default(),
                device_not_available: IDTEntry::default(),
                fucky_wucky_twice: IDTEntry::default(),
                we_are_all_about_backwards_compatibility: IDTEntry::default(),
                invalid_tss: IDTEntry::default(),
                segment_not_present: IDTEntry::default(),
                stack_segment_fault: IDTEntry::default(),
                uh_oh_we_gotta_hacker_here: IDTEntry::default(),
                page_fault: IDTEntry::default(),
                reserved_1: IDTEntry::default(),
                x87_floating_point_exception: IDTEntry::default(),
                alignment_check: IDTEntry::default(),
                machine_check: IDTEntry::default(),
                the_point_of_the_mask_float_exception: IDTEntry::default(),
                virtualization_exception: IDTEntry::default(),
                reserved_2: IDTEntry::default(),
                vmm_communication_exception: IDTEntry::default(),
                security_exception: IDTEntry::default(),
                reserved_3: IDTEntry::default(),
                interrupts: [IDTEntry::default(); 256 - 32]
            }
        }

        pub fn load(&'static self) {
            unsafe {
                // load the IDT
                let idt_ptr = &self as *const _ as *const _;
                asm!("lidt [{}]", in(reg) idt_ptr, options(readonly, nostack, preserves_flags));
            }
        }
    }

    pub type InterruptHandler = extern "x86-interrupt" fn(_: InterruptStackFrame);
    pub type ErrorWithErrorCodeHandler = extern "x86-interrupt" fn(_: InterruptStackFrame, error_code: u32);
    pub type SeriousFuckUpHandler = extern "x86-interrupt" fn(_: InterruptStackFrame) -> !;
    pub type SeriousFuckUpWithErrorCodeHandler = extern "x86-interrupt" fn(_: InterruptStackFrame, error_code: u32) -> !;

    impl_idtentry!(InterruptHandler);
    impl_idtentry!(ErrorWithErrorCodeHandler);
    impl_idtentry!(SeriousFuckUpHandler);
    impl_idtentry!(SeriousFuckUpWithErrorCodeHandler);

    // y'know the x86 crate has a point with doing it this way, so i'm gonna do the same (:

    pub struct InterruptStackFrame {
        pub value: InterruptStackValues,
    }

    pub struct InterruptStackValues {
        pub instruction_pointer: u32,
        pub code_segment: u32,
        pub cpu_flags: u32,
        pub stack_pointer: u32,
    }

    pub enum ErrorKind {
        HardwareFuckUp,
    }

    pub enum ErrorLevel {
        HugeFuckUp,
        MinorFuckUp,
        Warning,
    }

    pub struct KernelError {
        pub kind: ErrorKind,
        pub level: ErrorLevel,
        pub desc: &'static str,
        pub nerdinfo: &'static str,
    }

    impl KernelError {
        pub fn new(kind: ErrorKind, level: ErrorLevel, desc: &'static str, nerdinfo: &'static str) -> KernelError {
            KernelError {
                kind: kind,
                level: level,
                desc: desc,
                nerdinfo: nerdinfo,
            }
        }
    }
}