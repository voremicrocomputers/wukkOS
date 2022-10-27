use core::marker::PhantomData;
use crate::KernelArgs;

#[cfg(feature = "f_multiboot2")]
use multiboot2::{load, MemoryMapTag, BootInformation};

pub struct KernelInfo {
    kernel_start: u64,
    kernel_end: u64,
    safe_mem_start: u64,
    #[cfg(feature = "f_multiboot2")]
    boot_info: BootInformation,
}

impl KernelInfo {
    pub fn init_from_kernel_args(args: KernelArgs) -> Self {
        #[cfg(feature = "f_multiboot2")]
        {
            let boot_info = unsafe { load(args.multiboot_information_address) }.expect("failed to load multiboot2 information");
            let elf_sections = boot_info.elf_sections_tag().expect("no elf sections tag");
            let kernel_start = elf_sections.sections().map(|s| s.start_address()).min().unwrap();
            let kernel_end = elf_sections.sections().map(|s| s.end_address()).max().unwrap();
            // get end of multiboot for safe memory
            let safe_mem_start = boot_info.start_address() + boot_info.total_size();
            let kernel_info = KernelInfo {
                kernel_start,
                kernel_end,
                safe_mem_start: safe_mem_start as u64,
                boot_info,
            };
            kernel_info
        }
    }

    #[cfg(feature = "f_multiboot2")]
    pub fn get_memory_tag(&self) -> &MemoryMapTag {
        let mm_tag = self.boot_info.memory_map_tag().expect("no memory map tag").clone();
        mm_tag
    }

    #[cfg(feature = "f_multiboot2")]
    pub fn memory_areas(&self) -> impl Iterator<Item = &multiboot2::MemoryArea> {
        let mm_tag = self.boot_info.memory_map_tag().expect("ERR NO MEM MAP TAG!");
        mm_tag.all_memory_areas()
    }

    pub fn is_safe_memory(&self, addr: u64) -> bool {
        addr >= self.safe_mem_start && addr >= self.kernel_end
    }

    pub fn safe_memory_start(&self) -> u64 {
        self.safe_mem_start
    }
}