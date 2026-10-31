//! The system textures' lock and unlock (`d3d::lock_texture`, `d3d::unlock_texture`,
//! `d3d::texture_surface_info`): what they share, and their test. A system texture's
//! record (`unk_5B521C`, 0x78 each, `platform_3d`'s `TextureRecord`): +4 width, +8
//! height, +0xC its pixel kind's description (+4 bytes a pixel, +8 the format), +0x14 its
//! first level's surface (a DirectDraw surface: `Lock` at +0x64 and `Unlock` at +0x80 of
//! its method table; `platform_3d::lock_surface` / `unlock_surface`), +0x60 locked.

use crate::errors::FatalError;

pub const RECORD: u32 = 0x78;

pub fn record(slot: u32) -> u32 {
    crate::pointers::game_address(
        crate::gfx::texture_upload::TEXTURE_TABLES
            .system
            .records
            .as_ptr(),
    )
    .wrapping_add(slot.wrapping_mul(RECORD))
}

pub fn get(a: u32) -> u32 {
    crate::pointers::read_or_end(a, "a texture's record or surface")
}

pub fn put(a: u32, v: u32) {
    crate::pointers::write_or_end(a, v, "a texture's record or surface");
}

pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

pub use rc2k_core::d3d::textures::compared_with_one;

// ---- the test: a surface of the test's ----
