//! `gfx::read_png` (0x41CB50): reads a PNG file into the picture buffer as 32-bit pixels.
//!
//! Takes: path
//! Reads the file's first 0x80 bytes into `buf` (0x93E8B0) and gives up unless they are
//! all there and are a PNG signature and an IHDR chunk of 13 bytes, with a width of at most
//! 800, a height of at most 600 and no interlacing. Then the file's size (`file_size`),
//! memory for the whole file from the game's allocator (`memory::allocate`, on the list
//! `dword_5BEC38`), the whole file read into it, `decode_png` into the picture buffer with
//! `buf` as the info block, `g_pictureFlags` = 0x802, `lay_out_png`, the picture's width,
//! height and row length set from the layout, and the pixels spread out to dwords
//! (`palette_pixels_to_dwords` if the file had a palette, else `rgb_pixels_to_dwords`).
//! The memory is given back (`memory::free_list`) on every way out. Once the decoder has been
//! called the carry flag is clear, whatever it returned.
//! One caller, the picture loader `gfx::read_picture_file`, which falls back on the PCX reader.
//! The file is read and decoded twice, and its memory counted twice in the game's memory
//! figures. Refuses what `read_file`, `file_size` and `decode_png` refuse. The name is a
//! reading.

use crate::errors::Refused;
use crate::gfx::decode_png::decode_png;
use crate::gfx::lay_out_png::lay_out_png;
use crate::gfx::owner::Png;
use crate::gfx::palette_pixels_to_dwords::palette_pixels_to_dwords;
use crate::gfx::rgb_pixels_to_dwords::rgb_pixels_to_dwords;
use crate::memory::free_list::free_list;

// ---- game data ----

const HEAD_LEN: u32 = 0x80;

/// What comes before the memory is taken: whether the file's head is a PNG the game takes.
fn head_is_taken(path: &[u8], system: &crate::system::owner::System) -> Result<bool, Refused> {
    let scratch = &system.work.files.buffer;
    scratch.write_at(0, &[0; 4], "the PNG head")?;
    let mut head = [0u8; HEAD_LEN as usize];
    head.copy_from_slice(&scratch.read_at(0, HEAD_LEN as usize, "the PNG head")?);
    let read = crate::files::read_file::read_file_into(path, &mut head, 0)?;
    scratch.write_at(0, &head, "the PNG head")?;
    if !read.ok || read.bytes_read.is_none_or(|n| n < HEAD_LEN) {
        return Ok(false);
    }
    // Converted: `rc2k_core::gfx::png::head_taken`.
    Ok(rc2k_core::gfx::png::head_taken(&head))
}

/// The part with the file's memory held: whether it worked.
fn read_held(
    png: &Png,
    path: &[u8],
    size: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    let pictures = &gfx.pictures;
    // The file's memory Rust's, counted on the list as the game's block was.
    let mut file = crate::memory::owned::counted(&png.file.memory, size)?;
    let file = &mut file[..size as usize];
    let read = crate::files::read_file::read_file_into(path, file, 0)?;
    if !read.ok || read.bytes_read != Some(size) {
        return Ok(false);
    }
    decode_png(
        png,
        file,
        pictures.picture.buffer.get(),
        crate::gfx::png::info_ptr(system),
    )?;
    pictures.picture.flags.set(0x802u32);
    lay_out_png(gfx, system)?;
    pictures.picture.width.set(pictures.decoder.width.get());
    pictures.picture.height.set(pictures.decoder.height.get());
    pictures
        .picture
        .row_bytes
        .set(pictures.decoder.stride.get());
    if crate::gfx::png::info_byte(core::mem::offset_of!(crate::gfx::png::PngInfo, has_palette)) == 1
    {
        palette_pixels_to_dwords(gfx)?;
    } else {
        rgb_pixels_to_dwords(gfx)?;
    }
    Ok(true)
}

/// Returns whether the picture was read.
pub fn read_png(
    png: &Png,
    path: &[u8],
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    let free = || {
        // Refused only with the game's own heap, which `allocate` refuses first.
        let _ = free_list(&png.file.memory);
    };
    // The original gives the list back on every way out, also with nothing on it.
    let taken = head_is_taken(path, system).inspect_err(|_| free())?;
    if !taken {
        free();
        return Ok(false);
    }
    let (size, sized) = crate::files::file_size::file_size(path).inspect_err(|_| free())?;
    if !sized {
        free();
        return Ok(false);
    }
    png.file.length.set(size);
    let worked = read_held(png, path, size, gfx, system);
    free();
    worked
}
