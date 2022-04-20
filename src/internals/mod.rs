pub mod WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood {
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

    pub struct IDTEntry<F> {
        based_offset: u16,
        code_selector: u16,
        ist_offset_wastes_6_bits: u8,
        attributes: u8,
        mid_offset: u16,
        offset_popping_off: u32,
        what: PhantomData<F>
    }

    pub struct InterruptDescriptorTable {
        pub divide_error: IDTEntry<ErrorHandler>,
        pub debug: IDTEntry<ErrorHandler>,
        pub dream_mask_sus_version: IDTEntry<ErrorHandler>, // non-maskable interrupt
        pub breakpoint: IDTEntry<ErrorHandler>,
        pub into_detected_overflow: IDTEntry<ErrorHandler>,
        pub in_the_fortnite_storm: IDTEntry<ErrorHandler>, // bound range exceeded
        pub owo_whats_this: IDTEntry<ErrorHandler>, // invalid opcode
        pub device_not_available: IDTEntry<ErrorHandler>,
        pub fucky_wucky_twice: IDTEntry<ErrorHandler>, // double fault
        we_are_all_about_backwards_compatibility: IDTEntry<ErrorHandler>, // coprocessor segment overrun
        pub invalid_tss: IDTEntry<ErrorHandler>,
        pub segment_not_present: IDTEntry<ErrorHandler>,
        pub stack_segment_fault: IDTEntry<ErrorHandler>,
        pub uh_oh_we_gotta_hacker_here: IDTEntry<ErrorHandler>, // general protection fault

    }

    pub struct ErrorHandler(());

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