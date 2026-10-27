//! What a picture goes through on its way into a texture (read 2026-10-14 from
//! `gfx::load_picture_to_texture`'s upload `sub_41A68B` and `scene::upload_car_icon`).
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

use crate::errors::BadInput;
use crate::gfx::picture::{read_bytes, write_bytes};
use crate::session::{Bytes, Dword};

/// IDA's address of the grading variables.
pub const START: usize = 0x4F4A24;
/// Their length.
pub const LEN: usize = 0x4F4B30 - START;

/// The texture pictures' grading (`gfx::texture_upload` grades a texture's colours when the
/// settings ask for it).
#[repr(C)]
pub struct Grading {
    /// `dword_4F4A24`: 1 while the textures are being read (`reload_system_textures`).
    pub reading: Dword,
    /// `dword_4F4A28`: the scale pictures are read at (0x100: none).
    pub scale: Dword,
    /// `dword_4F4A2C`: the saturation pictures are read at (0x100: none).
    pub saturation: Dword,
    /// `byte_4F4A30`: the grades table the last graded texture used.
    pub grades: Bytes<0x100>,
}

/// The grading variables; the first values are the image's.
pub static GRADING: Grading = Grading {
    reading: Dword::new(1),
    scale: Dword::new(0x100),
    saturation: Dword::new(0x100),
    grades: Bytes::new([0; 0x100]),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x4F4A24).
pub fn home() -> usize {
    crate::pointers::game_address(&GRADING as *const Grading) as usize
}

/// The second picture's place, in bytes from the first's start.
pub const KEPT: usize = rc2k_core::gfx::pixels::KEPT;

// Converted: `rc2k_core::gfx::pixels` (the pixels as slices of the counts given).

// `pixels` names a picture buffer (`gfx::picture`; the textures are read in it).

pub fn alpha_from_kept(pixels: u32, count: u32) -> Result<(), BadInput> {
    let mut buffer = read_bytes(pixels, 0, KEPT + count as usize * 4)?;
    rc2k_core::gfx::pixels::alpha_from_kept(&mut buffer, count as usize)
        .ok_or(BadInput("a texture's pixels shorter than both pictures"))?;
    write_bytes(pixels, 0, &buffer)
}

pub fn key_texture_colours(pixels: u32, count: u32, key: u32) -> Result<(), BadInput> {
    let mut dwords = dwords_of(&read_bytes(pixels, 0, count as usize * 4)?);
    rc2k_core::gfx::pixels::key_colours(&mut dwords, key);
    write_bytes(pixels, 0, &bytes_of(&dwords))
}

fn dwords_of(bytes: &[u8]) -> Vec<u32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|d| u32::from_le_bytes(*d))
        .collect()
}

fn bytes_of(dwords: &[u32]) -> Vec<u8> {
    dwords.iter().flat_map(|d| d.to_le_bytes()).collect()
}

pub use rc2k_core::gfx::pixels::{grades, grading, halvable, least_colour};

/// `table` gets the grades (the game's `byte_4F4A30`).
pub fn grade_texture(
    pixels: u32,
    count: u32,
    key: u32,
    scale: u32,
    saturation: u32,
    table: &mut [u8; 0x100],
) -> Result<(), BadInput> {
    *table = grades(scale);
    let mut dwords = dwords_of(&read_bytes(pixels, 0, count as usize * 4)?);
    rc2k_core::gfx::pixels::grade(&mut dwords, key, saturation, table);
    write_bytes(pixels, 0, &bytes_of(&dwords))
}

/// Returns the size after the last pass (as it was for no pass).
pub fn halve_texture(
    pixels: u32,
    width: u32,
    height: u32,
    times: u32,
) -> Result<(u32, u32), BadInput> {
    let mut dwords = dwords_of(&read_bytes(
        pixels,
        0,
        width as usize * height as usize * 4,
    )?);
    let size = rc2k_core::gfx::pixels::halve(&mut dwords, width, height, times)
        .ok_or(BadInput("a texture that is not its picture"))?;
    write_bytes(pixels, 0, &bytes_of(&dwords))?;
    Ok(size)
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&GRADING, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), GRADING.reading.addr() as usize);
        assert_eq!(
            crate::moved::place(0x4F4A30),
            GRADING.grades.addr() as usize
        );
    }
}
