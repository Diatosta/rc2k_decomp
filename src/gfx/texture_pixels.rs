//! What a picture goes through on its way into a texture (read 2026-10-14 from
//! `gfx::load_picture_to_texture`'s upload `sub_41A68B` and `scene::upload_plate`).
//! Pixels are 32-bit components: byte 0, 1, 2 the colour, byte 3 the alpha.
//!
//! - `gfx::alpha_from_kept` (0x405443): each pixel's alpha = the first byte of the
//!   pixel at the same place 0x40000 bytes on (a second picture kept there), inverted.
//! - `gfx::key_texture_colours` (0x40540C): alpha made 0xFF; a colour whose top five
//!   bits are all 0 becomes the key (the format's least colour), 0xF800F8 by those
//!   bits (magenta) becomes 0, all of it.
//! - `gfx::least_colour` (0x4053BC): the lowest step of each of a format's four masks
//!   put together, as components.
//! - `gfx::grade_texture` (0x4055A2): with the reading switch `dword_4F4A24` at 1 and
//!   the scale `dword_4F4A28` and saturation `dword_4F4A2C` not both 0x100: each
//!   colour's parts through a table of 0x100 bytes (`byte_4F4A30`, made here from the
//!   scale), drawn towards or away from their average (`g_thirds`) by the saturation,
//!   and a colour left black by the top five bits becomes the key; pixels of alpha 0
//!   are left alone.
//! - `gfx::halve_texture` (0x40545F): the picture made half as wide and high in
//!   place, so many times: each pixel the average of four, counting only those of an
//!   alpha above 0 for the colour.

use crate::framework::rva;

pub const THIRDS_RVA: usize = rva(0x4F4B30); // g_thirds
pub const READING_RVA: usize = rva(0x4F4A24); // dword_4F4A24
pub const SCALE_RVA: usize = rva(0x4F4A28); // dword_4F4A28
pub const SATURATION_RVA: usize = rva(0x4F4A2C); // dword_4F4A2C
pub const GRADES_RVA: usize = rva(0x4F4A30); // byte_4F4A30
/// The second picture's place, in bytes from the first's start.
pub const KEPT: usize = rc2k_core::gfx::pixels::KEPT;

// Converted: `rc2k_core::gfx::pixels` (the pixels as slices of the counts given).

pub unsafe fn alpha_from_kept(pixels: *mut u8, count: u32) {
    let buffer = unsafe { std::slice::from_raw_parts_mut(pixels, KEPT + count as usize * 4) };
    rc2k_core::gfx::pixels::alpha_from_kept(buffer, count as usize).expect("the slice holds both");
}

pub unsafe fn key_texture_colours(pixels: *mut u32, count: u32, key: u32) {
    let pixels = unsafe { std::slice::from_raw_parts_mut(pixels, count as usize) };
    rc2k_core::gfx::pixels::key_colours(pixels, key);
}

pub use rc2k_core::gfx::pixels::{grades, grading, halvable, least_colour};

/// `table` gets the grades (the game's `byte_4F4A30`).
pub unsafe fn grade_texture(
    pixels: *mut u8,
    count: u32,
    key: u32,
    scale: u32,
    saturation: u32,
    table: &mut [u8; 0x100],
) {
    *table = grades(scale);
    let pixels = unsafe { std::slice::from_raw_parts_mut(pixels as *mut u32, count as usize) };
    rc2k_core::gfx::pixels::grade(pixels, key, saturation, table);
}

/// Returns the size after the last pass (as it was for no pass).
pub unsafe fn halve_texture(pixels: *mut u32, width: u32, height: u32, times: u32) -> (u32, u32) {
    let pixels =
        unsafe { std::slice::from_raw_parts_mut(pixels, width as usize * height as usize) };
    rc2k_core::gfx::pixels::halve(pixels, width, height, times).expect("the slice is the picture")
}

/// Pixels for the self-tests, by lot: some of alpha 0, some near black, some magenta,
/// the rest anything.
pub fn test_pixels(seed: &mut u32, count: usize) -> Vec<u32> {
    (0..count)
        .map(|_| {
            *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let lot = *seed;
            match lot >> 28 {
                0 => lot & 0xFF_FFFF,
                1 => lot & 0xFF07_0707,
                2 => 0xF8_00F8 | lot & 0xFF07_0007,
                3 => lot | 0xFF00_0000,
                _ => lot.rotate_left(7),
            }
        })
        .collect()
}
