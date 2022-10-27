pub mod allocator;

use alloc::boxed::Box;
use alloc::sync::Arc;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate};
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
use crate::{debug, print, println};

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

pub fn read_phys_memory32(mem_mapper: &mut OffsetPageTable, frame_allocator: &mut BootInfoFrameAllocator, addr: u32) -> u32 {
    let initaladdr = VirtAddr::new(addr as u64);
    let addr = unsafe { mem_mapper.translate_addr(initaladdr) };
    if let Some(addr) = addr {
        let addr = addr.as_u64() as *const u32;
        unsafe { *addr }
    } else {
        debug!("read_phys_memory32: addr {:x} not mapped", initaladdr.as_u64());
        // map the page
        let frame = frame_allocator.allocate_frame().unwrap();
        debug!("allocated frame: {:?}", frame);
        let flags = x86_64::structures::paging::PageTableFlags::PRESENT | x86_64::structures::paging::PageTableFlags::WRITABLE;
        let page = x86_64::structures::paging::Page::containing_address(initaladdr);
        debug!("mapped page: {:?}", page);
        let map_to_result = unsafe { mem_mapper.map_to(page, frame, flags, frame_allocator) };
        debug!("map_to_result: {:?}", map_to_result);
        if map_to_result.is_err() {
            panic!("Failed to map page");
        }
        let addr = unsafe { mem_mapper.translate_addr(initaladdr) };
        if let Some(addr) = addr {
            let addr = addr.as_u64() as *const u32;
            unsafe { *addr }
        } else {
            panic!("Failed to map page");
        }
    }
}

pub fn write_phys_memory32(mem_mapper: &mut OffsetPageTable, frame_allocator: &mut BootInfoFrameAllocator, addr: u32, value: u32) {
    let initaladdr = VirtAddr::new(addr as u64);
    let addr = unsafe { mem_mapper.translate_addr(initaladdr) };
    if let Some(addr) = addr {
        let addr = addr.as_u64() as *mut u32;
        unsafe { *addr = value };
    } else {
        debug!("write_phys_memory32: addr {:x} not mapped", initaladdr.as_u64());
        // map the page
        let frame = frame_allocator.allocate_frame().unwrap();
        debug!("allocated frame: {:?}", frame);
        let flags = x86_64::structures::paging::PageTableFlags::PRESENT | x86_64::structures::paging::PageTableFlags::WRITABLE;
        let page = x86_64::structures::paging::Page::containing_address(initaladdr);
        debug!("mapped page: {:?}", page);
        let map_to_result = unsafe { mem_mapper.map_to(page, frame, flags, frame_allocator) };
        debug!("map_to_result: {:?}", map_to_result);
        if map_to_result.is_err() {
            panic!("Failed to map page");
        }
        let addr = unsafe { mem_mapper.translate_addr(initaladdr) };
        if let Some(addr) = addr {
            let addr = addr.as_u64() as *mut u32;
            unsafe { *addr = value };
        } else {
            panic!("Failed to map page");
        }
    }
}