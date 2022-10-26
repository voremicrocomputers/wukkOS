use core::marker::PhantomData;
use crate::KernelArgs;

#[cfg(feature = "f_multiboot2")]
use multiboot2::{load, MemoryMapTag, BootInformation};

pub struct KernelInfo {
    #[cfg(feature = "f_multiboot2")]
    boot_info: BootInformation,
}

impl KernelInfo {
    pub fn init_from_kernel_args(args: KernelArgs) -> Self {
        let mut kernel_info = KernelInfo {
            #[cfg(feature = "f_multiboot2")]
            boot_info: unsafe { load(args.multiboot_information_address).expect("ERR ARGS BAD!") },
        };
        kernel_info
    }

    #[cfg(feature = "f_multiboot2")]
    pub fn memory_areas(&self) -> impl Iterator<Item = &multiboot2::MemoryArea> {
        let mm_tag = self.boot_info.memory_map_tag().expect("ERR NO MEM MAP TAG!");
        mm_tag.all_memory_areas()
    }
}