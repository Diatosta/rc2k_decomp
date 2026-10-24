//! How a pixel is laid out, as the two colour conversions (`pixel_to_components`,
//! `components_to_pixel`) are told it, and the surface's own layout.

use crate::gfx::formats::Slot as FmtSlot;

/// The surface's format record (`g_surfaceFormat`, the session's): its place.
pub fn surface_format_addr() -> usize {
    crate::session::SCREEN.format_addr()
}
/// dword_4F3FA8: four bytes `components_to_pixel` shifts the components out of.
pub const SCRATCH_RVA: FmtSlot = FmtSlot::at(0x4F3FA8);

/// Converted: `rc2k_core::gfx::pixels::PixelFormat`.
pub use rc2k_core::gfx::pixels::PixelFormat;

pub fn surface_format() -> PixelFormat {
    unsafe { (surface_format_addr() as *const PixelFormat).read_unaligned() }
}
