//! The system textures' lock and unlock (`d3d::lock_texture`, `d3d::unlock_texture`,
//! `d3d::texture_surface_info`): what they share, and their test. A system texture's
//! record (`unk_5B521C`, 0x78 each, `platform_3d`'s `TextureRecord`): +4 width, +8
//! height, +0xC its pixel kind's description (+4 bytes a pixel, +8 the format), +0x14 its
//! first level's surface (a DirectDraw surface: `Lock` at +0x64 and `Unlock` at +0x80 of
//! its method table; `platform_3d::lock_surface` / `unlock_surface`), +0x60 locked.

use crate::errors::FatalError;

pub const RECORD: u32 = 0x78;

/// Record `slot`; `slot` is unchecked (as the original's), the pointer bounded to the
/// texture tables as a whole.
pub fn record(slot: u32) -> crate::pointers::GamePtr {
    use crate::gfx::texture_upload::{Pool, TEXTURE_TABLES, Textures};
    crate::pointers::GamePtr::to(&TEXTURE_TABLES).offset(
        ((core::mem::offset_of!(Textures, system) + core::mem::offset_of!(Pool, records)) as u32)
            .wrapping_add(slot.wrapping_mul(RECORD)),
    )
}

/// The dword `offset` bytes into the record `r`.
pub fn field(r: crate::pointers::GamePtr, offset: u32) -> u32 {
    r.offset(offset)
        .read::<u32>("a texture's record or surface")
        .unwrap_or_else(|e| crate::fatal::raise::end("d3d::texture_lock::field", e))
}

/// Sets the dword `offset` bytes into the record `r`.
pub fn put_field(r: crate::pointers::GamePtr, offset: u32, v: u32) {
    r.offset(offset)
        .write::<u32>(v, "a texture's record or surface")
        .unwrap_or_else(|e| crate::fatal::raise::end("d3d::texture_lock::put_field", e));
}

pub fn get(a: u32) -> u32 {
    crate::pointers::read_or_end(a, "a texture's record or surface")
}

pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

pub use rc2k_core::d3d::textures::compared_with_one;

// ---- the test: a surface of the test's ----
