/// The info block's place (a game address). The shared scratch (`files::scratch::SCRATCH`,
/// 0x93E8B0) holds first the file's first 0x80 bytes, then what the decoder says about the
/// picture, a `PngInfo`.
pub fn info_ptr(system: &crate::system::owner::System) -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&system.work.files.buffer)
}

/// The info's dword at `offset` (`core::mem::offset_of!(PngInfo, ..)`).
pub fn info_dword(offset: usize, system: &crate::system::owner::System) -> u32 {
    system.work.files.buffer.get_u32(offset)
}

/// The info's byte at `offset`.
pub fn info_byte(offset: usize) -> u8 {
    let system = crate::system::owner::system();
    system.work.files.buffer.get(offset)
}

/// What `decode_png` leaves behind. Only the first byte of `has_palette` is written, and the
/// palette only when the file has one: the rest is whatever was there (in the original, the
/// work area of zlib, which it lays over this block).
#[repr(C)]
pub struct PngInfo {
    /// 1 if the file has a palette (a PLTE chunk), else 0.
    pub has_palette: u8,
    pub unset: [u8; 3],
    pub width: u32,
    pub height: u32,
    /// Three bytes an entry.
    pub palette: [u8; 0x300],
}
