//! How a pixel is laid out, as the two colour conversions (`pixel_to_components`,
//! `components_to_pixel`) are told it, and the surface's own layout.

use crate::framework::{game_ptr, rva};

pub const SURFACE_FORMAT_RVA: usize = rva(0x4F496C); // dword_4F496C
/// dword_4F3FA8: four bytes `components_to_pixel` shifts the components out of.
pub const SCRATCH_RVA: usize = rva(0x4F3FA8);

/// Converted: `rc2k_core::gfx::pixels::PixelFormat`.
pub use rc2k_core::gfx::pixels::PixelFormat;

pub fn surface_format() -> PixelFormat {
    unsafe { game_ptr::<PixelFormat>(SURFACE_FORMAT_RVA).read_unaligned() }
}
