//! How a pixel is laid out, as the two colour conversions (`pixel_to_components`,
//! `components_to_pixel`) are told it, and the surface's own layout.

/// The surface's format record (`g_surfaceFormat`, the session's): its place.
pub fn surface_format_addr() -> u32 {
    crate::session::SCREEN.format_addr()
}

/// Converted: `rc2k_core::gfx::pixels::PixelFormat`.
pub use rc2k_core::gfx::pixels::PixelFormat;

pub fn surface_format() -> Result<PixelFormat, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(surface_format_addr()).read("the surface's format")
}
