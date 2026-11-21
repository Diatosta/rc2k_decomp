//! The system textures' lock and unlock (`d3d::lock_texture`, `d3d::unlock_texture`,
//! `d3d::texture_surface_info`): what they share, and their test. A system texture's record
//! (`Textures.system.records`, `TextureRecord`): `width`, `height`, `format` (its pixel kind's
//! description: bytes a pixel and the format), `surfaces[0]` its first level's surface
//! (`d3d::device::lock_surface` / `unlock_surface`), `locked`.

use crate::errors::FatalError;
use crate::gfx::texture_upload::RecordCell;

/// Record `slot`, if it is in the pool.
pub fn record(gfx: &crate::gfx::owner::Gfx, slot: u32) -> Option<&RecordCell> {
    gfx.card.texture_tables.system.records.get(slot as usize)
}

pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

// ---- the test: a surface of the test's ----
