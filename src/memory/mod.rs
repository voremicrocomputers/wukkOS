pub mod allocator;

use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub unsafe fn init(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(phys_mem_offset);
    OffsetPageTable::new(level_4_table, phys_mem_offset)
}

unsafe fn active_level_4_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut  *page_table_ptr } // unsafe
}

#[cfg(feature = "f_multiboot2")]
use multiboot2::{MemoryMapTag, BootInformation};
use spin::Mutex;
use crate::boot::KernelInfo;

pub struct BootInfoFrameAllocator {
    kern_info: Mutex<KernelInfo>,
    next: usize,
}

impl BootInfoFrameAllocator {
    #[cfg(feature = "f_multiboot2")]
    pub unsafe fn init(kern_info: Mutex<crate::boot::KernelInfo>) -> Self {
        Self {
            kern_info,
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        #[cfg(feature = "f_multiboot2")] {
            let mut kern_lock = self.kern_info.lock();
            let mut usable_frames = kern_lock
                .memory_areas();
            let mut usable_frames = usable_frames
                    .map(|area| {
                        let frame_addr = area.start_address();
                        let frame_end = area.end_address();
                        let frame_size = frame_end - frame_addr;
                        let num_frames = frame_size / 4096;
                        let start_frame = PhysFrame::containing_address(PhysAddr::new(frame_addr));
                        (0..num_frames).map(move |i| start_frame + i)
                    })
                    .flatten();
            let frame = usable_frames.nth(self.next).clone();
            self.next += 1;

            // ensure unlock
            unsafe { self.kern_info.force_unlock() };

            frame
        }
    }
}