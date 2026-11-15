//! What a picture goes through on its way into a texture (from `gfx::upload_texture_picture` and
//! `scene::upload_car_icon`). Pixels are 32-bit components: bytes 0, 1, 2 the colour, byte 3 the
//! alpha.
//!
//! - `gfx::alpha_from_kept`: each pixel's alpha = the first byte of the pixel at the same place
//!   0x40000 bytes on (a second picture kept there), inverted.
//! - `gfx::key_texture_colours`: alpha made 0xFF; a colour whose top five bits are all 0 becomes
//!   the key (the format's least colour), 0xF800F8 by those bits (magenta) becomes 0.
//! - `gfx::least_colour`: the lowest step of each of a format's four masks put together.
//! - `gfx::grade_texture`: with the reading switch at 1 and the scale and saturation not both
//!   0x100: each colour's parts go through a table of 0x100 bytes (`grades`, made from the
//!   scale), drawn towards or away from their average by the saturation, and a colour left black
//!   by the top five bits becomes the key; pixels of alpha 0 are left alone.
//! - `gfx::halve_texture`: the picture made half as wide and high in place, so many times: each
//!   pixel the average of four, counting only those of an alpha above 0 for the colour.

use crate::errors::BadInput;
use crate::gfx::picture::{PictureBuffers, read_bytes, write_bytes};
use crate::memory::session::{PlainBytes, PlainDword};

/// The texture pictures' grading (`gfx::texture_upload` grades a texture's colours when the
/// settings ask for it).
#[derive(rc2k_derive::Cells)]
pub struct Grading {
    /// 1 while the textures are being read (`reload_system_textures`).
    pub reading: PlainDword,
    /// The scale pictures are read at (0x100: none).
    pub scale: PlainDword,
    /// The saturation pictures are read at (0x100: none).
    pub saturation: PlainDword,
    /// `byte_4F4A30`: the grades table the last graded texture used.
    pub grades: PlainBytes<0x100>,
}

impl Grading {
    pub const fn new() -> Self {
        Grading {
            reading: PlainDword::new(1),
            scale: PlainDword::new(0x100),
            saturation: PlainDword::new(0x100),
            grades: PlainBytes::new([0; 0x100]),
        }
    }
}

/// The second picture's place, in bytes from the first's start.
pub const KEPT: usize = rc2k_core::gfx::pixels::KEPT;

// `pixels` names a picture buffer (`gfx::picture`; the textures are read in it).

pub fn alpha_from_kept(buffers: &PictureBuffers, pixels: u32, count: u32) -> Result<(), BadInput> {
    // In the buffer itself: only the first pixels' alpha bytes change.
    crate::gfx::picture::with_buffer(buffers, pixels, |buffer| {
        let part = buffer.get_mut(..KEPT + count as usize * 4)?;
        rc2k_core::gfx::pixels::alpha_from_kept(part, count as usize)
    })?
    .ok_or(BadInput("a read past the picture buffer"))
}

pub fn key_texture_colours(
    buffers: &PictureBuffers,
    pixels: u32,
    count: u32,
    key: u32,
) -> Result<(), BadInput> {
    crate::gfx::picture::with_buffer(buffers, pixels, |buffer| {
        let part = buffer.get_mut(..count as usize * 4)?;
        rc2k_core::gfx::pixels::key_colours_in_bytes(part, key);
        Some(())
    })?
    .ok_or(BadInput("a read past the picture buffer"))
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

/// `table` gets the grades (the game's `Grading.grades`).
pub fn grade_texture(
    buffers: &PictureBuffers,
    pixels: u32,
    count: u32,
    key: u32,
    scale: u32,
    saturation: u32,
    table: &mut [u8; 0x100],
) -> Result<(), BadInput> {
    *table = grades(scale);
    let mut dwords = dwords_of(&read_bytes(buffers, pixels, 0, count as usize * 4)?);
    rc2k_core::gfx::pixels::grade(&mut dwords, key, saturation, table);
    write_bytes(buffers, pixels, 0, &bytes_of(&dwords))
}

/// Returns the size after the last pass (as it was for no pass).
pub fn halve_texture(
    buffers: &PictureBuffers,
    pixels: u32,
    width: u32,
    height: u32,
    times: u32,
) -> Result<(u32, u32), BadInput> {
    let mut dwords = dwords_of(&read_bytes(
        buffers,
        pixels,
        0,
        width as usize * height as usize * 4,
    )?);
    let size = rc2k_core::gfx::pixels::halve(&mut dwords, width, height, times)
        .ok_or(BadInput("a texture that is not its picture"))?;
    write_bytes(buffers, pixels, 0, &bytes_of(&dwords))?;
    Ok(size)
}
