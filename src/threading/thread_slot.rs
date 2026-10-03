#[repr(C)]
pub struct ThreadSlot {
    pub unknown0: u32,
    pub thread_id: u32,
    _rest: [u8; 0x108],
}