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

use crate::framework::{game_read, rva};
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::pixel_to_components::pixel_to_components;

pub const THIRDS_RVA: usize = rva(0x4F4B30); // g_thirds
pub const READING_RVA: usize = rva(0x4F4A24); // dword_4F4A24
pub const SCALE_RVA: usize = rva(0x4F4A28); // dword_4F4A28
pub const SATURATION_RVA: usize = rva(0x4F4A2C); // dword_4F4A2C
pub const GRADES_RVA: usize = rva(0x4F4A30); // byte_4F4A30
/// The second picture's place, in bytes from the first's start.
pub const KEPT: usize = 0x40000;

fn thirds(sum: u32) -> u32 {
    unsafe { game_read::<u8>(THIRDS_RVA + sum as usize) as u32 }
}

pub unsafe fn alpha_from_kept(pixels: *mut u8, count: u32) {
    for n in 0..count as usize {
        unsafe { *pixels.add(n * 4 + 3) = !*pixels.add(KEPT + n * 4) };
    }
}

pub unsafe fn key_texture_colours(pixels: *mut u32, count: u32, key: u32) {
    let key = key | 0xFF00_0000;
    for n in 0..count as usize {
        unsafe {
            let p = pixels.add(n).read_unaligned();
            let v = match p & 0xF8_F8F8 {
                0 => key,
                0xF8_00F8 => 0,
                _ => p | 0xFF00_0000,
            };
            pixels.add(n).write_unaligned(v);
        }
    }
}

pub fn least_colour(format: &PixelFormat) -> u32 {
    let pixel = format
        .masks
        .iter()
        .fold(0, |acc, &m| acc | (m & !m.wrapping_mul(2)));
    pixel_to_components(format, pixel)
}

/// The table of 0x100 bytes the scale makes.
pub fn grades(scale: u32) -> [u8; 0x100] {
    let mut table = [0u8; 0x100];
    let mut sum = 0u32;
    for n in 0..0x100 {
        table[n] = (sum >> 8) as u8;
        sum = sum.wrapping_add(scale);
        if sum > 0xFF00 {
            table[n..].fill(0xFF);
            break;
        }
    }
    table
}

/// Whether the grading is done: the reading switch and the two settings as they are.
pub fn grading(reading: u32, scale: u32, saturation: u32) -> bool {
    reading == 1 && !(scale == 0x100 && saturation == 0x100)
}

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
    let key = key & 0xFF_FFFF;
    let saturation = saturation as i32;
    for n in 0..count as usize {
        unsafe {
            let p = pixels.add(n * 4);
            if *p.add(3) == 0 {
                continue;
            }
            let parts = [0, 1, 2].map(|i| table[*p.add(i) as usize] as u32);
            let average = thirds(parts[2] + parts[0] + parts[1]) as i32;
            for (i, &part) in parts.iter().enumerate() {
                let v =
                    ((part as i32 - average).wrapping_mul(saturation) >> 8).wrapping_add(average);
                *p.add(i) = if v < 0 {
                    0
                } else if v > 0xFF {
                    0xFF
                } else {
                    v as u8
                };
            }
            let whole = (p as *mut u32).read_unaligned();
            if whole & 0xF8_F8F8 == 0 {
                (p as *mut u32).write_unaligned(whole & 0xFF00_0000 | key);
            }
        }
    }
}

/// Whether `halve_texture` can do it: every pass at least two pixels wide and high.
pub fn halvable(width: u32, height: u32, times: u32) -> bool {
    times <= 32 && (width >> times) > 0 && (height >> times) > 0
}

/// Returns the size after the last pass (as it was for no pass).
pub unsafe fn halve_texture(pixels: *mut u32, width: u32, height: u32, times: u32) -> (u32, u32) {
    let (mut width, mut height) = (width, height);
    for _ in 0..times {
        let row = width as usize;
        width >>= 1;
        height >>= 1;
        let (mut from, mut to) = (pixels, pixels);
        for _ in 0..height {
            for _ in 0..width {
                unsafe {
                    let four = [
                        from.read_unaligned(),
                        from.add(1).read_unaligned(),
                        from.add(row).read_unaligned(),
                        from.add(row + 1).read_unaligned(),
                    ];
                    let (mut alpha, mut sums, mut counted) = (0u32, [0u32; 3], 0u32);
                    for p in four {
                        if p >> 24 != 0 {
                            alpha += p >> 24;
                            for (i, sum) in sums.iter_mut().enumerate() {
                                *sum += (p >> (i * 8)) & 0xFF;
                            }
                            counted += 1;
                        }
                    }
                    let alpha = (alpha >> 2) & 0xFF;
                    let v = if alpha == 0 {
                        0
                    } else {
                        let sums = sums.map(|s| match counted {
                            1 => s,
                            2 => s >> 1,
                            3 => thirds(s),
                            _ => s >> 2,
                        });
                        alpha << 24 | sums[2] << 16 | sums[1] << 8 | sums[0]
                    };
                    to.write_unaligned(v);
                    from = from.add(2);
                    to = to.add(1);
                }
            }
            from = unsafe { from.add(row) };
        }
    }
    (width, height)
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
