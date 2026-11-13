//! The screen mode's choice and the draw surface's description (what `gfx::try_screen_mode` leaves
//! of the mode chosen, `platform` presents, the race's drawing code writes pixels through).
//! `#[repr(C)]` structs of atomic cells.

use crate::session::{Dword, PlainDword};

/// The screen mode chosen.
#[derive(rc2k_derive::Cells)]
pub struct ScreenMode {
    /// The depth formats' bits (`gfx::mark_3d_modes`).
    pub depth_bits: PlainDword,
    /// The extra flags.
    pub depth_extra: PlainDword,
    /// The flags of the screen mode in use.
    pub mode_flags: PlainDword,
    /// The mode set (`platform`).
    pub mode_set: PlainDword,
    /// The menus' screen mode.
    pub menu_mode: PlainDword,
    /// The race's screen mode.
    pub race_mode: PlainDword,
}

impl ScreenMode {
    pub const fn new() -> Self {
        ScreenMode {
            depth_bits: PlainDword::new(0x0),
            depth_extra: PlainDword::new(0x0),
            mode_flags: PlainDword::new(0x0),
            mode_set: PlainDword::new(0x0),
            menu_mode: PlainDword::new(0x0),
            race_mode: PlainDword::new(0x0),
        }
    }
}

/// The surface the screen is drawn on.
#[derive(rc2k_derive::Cells)]
pub struct DrawSurface {
    /// The presented picture's row pitch (`platform`).
    pub present_pitch: Dword,
    /// The presented picture's pixels.
    pub present_pixels: Dword,
    /// The bytes after a row.
    pub present_gap: Dword,
    /// The picture is upside down.
    pub flipped: Dword,
    /// The frames presented.
    pub frames: Dword,
    /// The surface's left (`Surface`, 0x30 bytes).
    pub x: Dword,
    /// Its top.
    pub y: Dword,
    /// Its width.
    pub width: Dword,
    /// Its height (rows).
    pub height: Dword,
    /// Bits a pixel.
    pub bits_per_pixel: Dword,
    /// Bytes a pixel.
    pub bytes_per_pixel: Dword,
    /// Bytes a row.
    pub row_bytes: Dword,
    /// Dwords a row.
    pub row_dwords: Dword,
    /// The pixel format record, by handle (`gfx::pixel_format::layout`).
    pub format: Dword,
    /// The row pitch.
    pub pitch: Dword,
    /// The pixels (a pointer).
    pub pixels: Dword,
    /// The bytes after a row's pixels.
    pub row_gap: Dword,
}

impl DrawSurface {
    /// The surface.
    pub const fn new() -> Self {
        DrawSurface {
            present_pitch: Dword::new(0x0),
            present_pixels: Dword::new(0x0),
            present_gap: Dword::new(0x0),
            flipped: Dword::new(0x0),
            frames: Dword::new(0x0),
            x: Dword::new(0x0),
            y: Dword::new(0x0),
            width: Dword::new(0x0),
            height: Dword::new(0x0),
            bits_per_pixel: Dword::new(0x0),
            bytes_per_pixel: Dword::new(0x0),
            row_bytes: Dword::new(0x0),
            row_dwords: Dword::new(0x0),
            format: Dword::new(0x4f496c),
            pitch: Dword::new(0x0),
            pixels: Dword::new(0x0),
            row_gap: Dword::new(0x0),
        }
    }
}
