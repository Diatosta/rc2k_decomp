//! The screen mode's choice and the draw surface's description (what `gfx::try_screen_mode` leaves
//! of the mode chosen, `platform` presents, the race's drawing code writes pixels through), out
//! of RAL.EXE's image. Placed in a file of its own beside the graphics module's stores.
//!
//! `#[repr(C)]` structs of atomic cells, first values the image's
//! (`tests`).

use crate::session::{Dword, PlainDword};

/// The screen mode chosen (IDA 0x4F492C .. 0x4F494C).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ScreenMode {
    /// 0x4F492C: `g_depthFormatBits`: the depth formats' bits (`gfx::mark_3d_modes`).
    pub depth_bits: PlainDword,
    /// 0x4F4930: `g_depthExtraFlags`: the extra flags.
    pub depth_extra: PlainDword,
    /// 0x4F4934: `g_screenModeFlags`: the flags of the screen mode in use.
    pub mode_flags: PlainDword,
    /// 0x4F4938: `g_screenModeSet`: the mode set (`platform`).
    pub mode_set: PlainDword,
    /// 0x4F4944: `g_menuScreenMode`: the menus' screen mode.
    pub menu_mode: PlainDword,
    /// 0x4F4948: `g_raceScreenMode`: the race's screen mode.
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

/// The surface the screen is drawn on (IDA 0x4F4984 .. 0x4F49C8).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawSurface {
    /// 0x4F4984: `dword_4F4984`: the presented picture's row pitch (`platform`).
    pub present_pitch: Dword,
    /// 0x4F4988: `dword_4F4988`: the presented picture's pixels.
    pub present_pixels: Dword,
    /// 0x4F498C: `dword_4F498C`: the bytes after a row.
    pub present_gap: Dword,
    /// 0x4F4990: `dword_4F4990`: the picture is upside down.
    pub flipped: Dword,
    /// 0x4F4994: `dword_4F4994`: the frames presented.
    pub frames: Dword,
    /// 0x4F4998: `g_surface.x`: the surface's left (`Surface`, 0x30 bytes).
    pub x: Dword,
    /// 0x4F499C: `g_surface.y`: its top.
    pub y: Dword,
    /// 0x4F49A0: `g_surface.width`: its width.
    pub width: Dword,
    /// 0x4F49A4: `g_surface.height`: its height (rows).
    pub height: Dword,
    /// 0x4F49A8: `g_surface.bits_per_pixel`: bits a pixel.
    pub bits_per_pixel: Dword,
    /// 0x4F49AC: `g_surface.bytes_per_pixel`: bytes a pixel.
    pub bytes_per_pixel: Dword,
    /// 0x4F49B0: `g_surface.row_bytes`: bytes a row.
    pub row_bytes: Dword,
    /// 0x4F49B4: `g_surface.row_dwords`: dwords a row.
    pub row_dwords: Dword,
    /// 0x4F49B8: `g_surface.format`: the pixel format record, by handle
    /// (`gfx::pixel_format::layout`; the original keeps its address).
    pub format: Dword,
    /// 0x4F49BC: `g_surface.pitch`: the row pitch.
    pub pitch: Dword,
    /// 0x4F49C0: `g_surface.pixels`: the pixels (a pointer).
    pub pixels: Dword,
    /// 0x4F49C4: `g_surface.row_gap`: the bytes after a row's pixels.
    pub row_gap: Dword,
}

impl DrawSurface {
    /// The first values are the image's.
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
