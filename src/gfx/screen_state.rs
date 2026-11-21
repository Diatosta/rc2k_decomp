//! The screen mode's choice and the draw surface's description (what `gfx::try_screen_mode` leaves
//! of the mode chosen, `platform` presents, the race's drawing code writes pixels through).
//! The draw surface is atomic cells; the mode choice plain ones.

use crate::memory::session::Dword;
use std::cell::Cell;

/// The screen mode chosen.
pub struct ScreenMode {
    /// The depth formats' bits (`gfx::mark_3d_modes`).
    pub depth_bits: Cell<u32>,
    /// The extra flags.
    pub depth_extra: Cell<u32>,
    /// The flags of the screen mode in use.
    pub mode_flags: Cell<u32>,
    /// The menus' screen mode.
    pub menu_mode: Cell<u32>,
    /// The race's screen mode.
    pub race_mode: Cell<u32>,
}

impl ScreenMode {
    pub const fn new() -> Self {
        ScreenMode {
            depth_bits: Cell::new(0),
            depth_extra: Cell::new(0),
            mode_flags: Cell::new(0),
            menu_mode: Cell::new(0),
            race_mode: Cell::new(0),
        }
    }
}

/// The surface the screen is drawn on (atomic cells: the drawing threads read it).
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
            format: Dword::new(crate::gfx::pixel_format::SCREEN_FORMAT),
            pitch: Dword::new(0x0),
            pixels: Dword::new(0x0),
            row_gap: Dword::new(0x0),
        }
    }
}
