pub mod WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood {
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