//! How a pixel is laid out, as the two colour conversions (`pixel_to_components`,
//! `components_to_pixel`) are told it, and the surface's own layout.

use crate::errors::BadInput;
use crate::gfx::owner::gfx;
use crate::system::owner::system;

/// A pixel layout the game keeps a pointer to (`g_surface.format`, `g_glyphPixelFormat`),
/// by handle (phase 13): `SCREEN_FORMAT` the screen's record (`g_surfaceFormat`, the
/// session's), `TEXTURE_FORMAT | k` texture description `k`'s layout (its +8); 0 none.
pub const SCREEN_FORMAT: u32 = 0xCD00_0000;
pub const TEXTURE_FORMAT: u32 = 0xCD00_0100;
/// What `texture_kind_format` keeps for a kind without its description: the original's
/// `0 + 8`, a layout no read finds.
const NO_TEXTURE_FORMAT: u32 = TEXTURE_FORMAT | 0xFF;

/// The handle for texture kind `n`'s layout, as the original takes its address
/// (`kinds[n] + 8`) when it keeps it: the description it names now.
pub fn texture_kind_format(n: usize, gfx: &crate::gfx::owner::Gfx) -> u32 {
    match gfx.card.texture_tables.kind_index(n) {
        Some(k) => TEXTURE_FORMAT | k as u32,
        None => NO_TEXTURE_FORMAT,
    }
}

/// The handle for the layout of the texture description `description` names (a texture
/// record's kind field, a `FORMAT_TAG` handle, whose +8 the original keeps): a value that
/// is none gives a handle no read finds, as the original's read there would go wrong.
pub fn texture_description_format(description: u32, gfx: &crate::gfx::owner::Gfx) -> u32 {
    use crate::gfx::texture_upload::FORMAT_TAG;
    match description.checked_sub(FORMAT_TAG) {
        Some(k) if (k as usize) < gfx.card.texture_tables.descriptions.len() => TEXTURE_FORMAT | k,
        _ => NO_TEXTURE_FORMAT,
    }
}

/// The layout `handle` names (not 0: the callers test that first, with their own
/// messages); one that names none is refused as the original's read of it would fail.
pub fn layout(handle: u32) -> Result<PixelFormat, BadInput> {
    if handle == SCREEN_FORMAT {
        return surface_format();
    }
    let descriptions = &gfx().card.texture_tables.descriptions;
    match handle.checked_sub(TEXTURE_FORMAT) {
        Some(k) if (k as usize) < descriptions.len() => Ok(descriptions[k as usize].get().format),
        _ => Err(BadInput("a pixel layout's handle that names none")),
    }
}

/// Converted: `rc2k_core::gfx::pixels::PixelFormat`.
pub use rc2k_core::gfx::pixels::PixelFormat;

pub fn surface_format() -> Result<PixelFormat, BadInput> {
    let format = &system().session.screen.format;
    Ok(PixelFormat {
        layout: format[0].get(),
        masks: std::array::from_fn(|n| format[1 + n].get()),
    })
}
