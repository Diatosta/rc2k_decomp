//! The system textures' lock and unlock (`d3d::lock_texture`, `d3d::unlock_texture`,
//! `d3d::texture_surface_info`): what they share, and their test. A system texture's record
//! (`Textures.system.records`, `TextureRecord`): +4 width, +8 height, +0xC its pixel kind's
//! description (+4 bytes a pixel, +8 the format), +0x14 its first level's surface
//! (`platform_3d::lock_surface` / `unlock_surface`), +0x60 locked.

use crate::errors::FatalError;
use crate::gfx::texture_upload::RecordCell;

/// Record `slot`, if it is in the pool.
pub fn record(slot: u32) -> Option<&'static RecordCell> {
    crate::gfx::owner::textures()
        .system
        .records
        .get(slot as usize)
}

pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

pub use rc2k_core::d3d::textures::compared_with_one;

// ---- the test: a surface of the test's ----
