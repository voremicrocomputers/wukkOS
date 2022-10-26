#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}