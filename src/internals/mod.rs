pub mod cpu;

pub mod WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood {

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