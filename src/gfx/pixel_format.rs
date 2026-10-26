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

/// Layouts for the self-tests: three with rotations, six bit by bit (16 and 32 bits, one
/// with components wider than a byte, one with overlapping masks, one of a single bit).
pub const TEST_FORMATS: [PixelFormat; 9] = [
    PixelFormat {
        layout: 0x8000_0000,
        masks: [0xFF, 0xFF00, 0xFF_0000, 0],
    },
    PixelFormat {
        layout: 0x9810_081D,
        masks: [0xF800, 0x07E0, 0x001F, 0x8000_0000],
    },
    PixelFormat {
        layout: 0x8305_0B13,
        masks: [0x7C00, 0x03E0, 0x001F, 0xFF00_0000],
    },
    PixelFormat {
        layout: 15,
        masks: [0xF800, 0x07E0, 0x001F, 0],
    },
    PixelFormat {
        layout: 31,
        masks: [0xFF_0000, 0xFF00, 0xFF, 0xFF00_0000],
    },
    PixelFormat {
        layout: 14,
        masks: [0x7C00, 0x03E0, 0x001F, 0],
    },
    PixelFormat {
        layout: 31,
        masks: [0x3FF0_0000, 0x000F_FC00, 0x3FF, 0],
    },
    PixelFormat {
        layout: 23,
        masks: [0xFF_00FF, 0xFF00, 0xF0, 0xFF_FFFF],
    },
    PixelFormat {
        layout: 0,
        masks: [1, 0, 0, 0],
    },
];

pub const TEST_VALUES: [u32; 12] = [
    0,
    1,
    0xFF,
    0xFF00,
    0x12_3456,
    0xFFFF_FFFF,
    0x8080_8080,
    0xA5C3_F017,
    0x7FFF,
    0xF81F,
    0x0100_0000,
    0x5A5A_A5A5,
];
