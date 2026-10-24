//! `gfx::read_png` (0x41CB50): reads a PNG file into the picture buffer as 32-bit pixels.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
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
//! One caller, the picture loader `sub_4119F4`, which falls back on the PCX reader.
//! The file is read and decoded twice, and its memory counted twice in the game's memory
//! figures. Refuses what `read_file`, `file_size` and `decode_png` refuse. The name is a
//! reading.

use crate::errors::Refused;
use crate::files::file_size::file_size;
use crate::files::read_file::read_file;
use crate::gfx::block::{game_addr, game_ptr, game_read, game_write};
use crate::gfx::decode_png::{FILE_LEN_RVA, FILE_RVA, decode_png};
use crate::gfx::lay_out_png::lay_out_png;
use crate::gfx::palette_pixels_to_dwords::palette_pixels_to_dwords;
use crate::gfx::pcx::{DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, STRIDE_RVA};
use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA, picture};
use crate::gfx::png::INFO_RVA;
use crate::gfx::png_file::Slot as PngSlot;
use crate::gfx::rgb_pixels_to_dwords::rgb_pixels_to_dwords;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;

// ---- game data ----

const FILE_MEMORY_RVA: PngSlot = PngSlot::at(0x5BEC38); // dword_5BEC38: the list the file's memory is on
const HEAD_LEN: u32 = 0x80;

// ---- Rust version ----

/// What comes before the memory is taken: whether the file's head is a PNG the game takes.
unsafe fn head_is_taken(path: *const u8) -> Result<bool, Refused> {
    unsafe {
        let head = game_addr(INFO_RVA) as *mut u8;
        (head as *mut u32).write_unaligned(0);
        let read = read_file(path, head, HEAD_LEN, 0)?;
        if !read.ok || read.bytes_read.is_none_or(|n| n < HEAD_LEN) {
            return Ok(false);
        }
        // Converted: `rc2k_core::gfx::png::head_taken`.
        Ok(rc2k_core::gfx::png::head_taken(
            &(head as *const [u8; 0x80]).read_unaligned(),
        ))
    }
}

/// The part with the file's memory held: whether it worked.
unsafe fn read_held(path: *const u8, size: u32) -> Result<bool, Refused> {
    unsafe {
        let file = allocate(game_addr(FILE_MEMORY_RVA) as *mut MemoryList, size)?;
        game_write(FILE_RVA, file as u32);
        let read = read_file(path, file, size, 0)?;
        if !read.ok || read.bytes_read != Some(size) {
            return Ok(false);
        }
        decode_png(
            std::slice::from_raw_parts(file, size as usize),
            picture(),
            game_ptr(INFO_RVA),
        )?;
        game_write(FLAGS_RVA, 0x802u32);
        lay_out_png();
        game_write(WIDTH_RVA, game_read::<u32>(DECODED_WIDTH_RVA));
        game_write(HEIGHT_RVA, game_read::<u32>(DECODED_HEIGHT_RVA));
        game_write(ROW_BYTES_RVA, game_read::<u32>(STRIDE_RVA));
        if game_read::<u8>(INFO_RVA) == 1 {
            palette_pixels_to_dwords()?;
        } else {
            rgb_pixels_to_dwords()?;
        }
        Ok(true)
    }
}

/// Returns whether the picture was read.
pub unsafe fn read_png(path: *const u8) -> Result<bool, Refused> {
    unsafe {
        let free = || {
            // Refused only with the game's own heap, which `allocate` refuses first.
            let _ = free_list(game_addr(FILE_MEMORY_RVA) as *mut MemoryList);
        };
        // The original gives the list back on every way out, also with nothing on it.
        let taken = head_is_taken(path).inspect_err(|_| free())?;
        if !taken {
            free();
            return Ok(false);
        }
        let (size, sized) = file_size(path).inspect_err(|_| free())?;
        if !sized {
            free();
            return Ok(false);
        }
        game_write(FILE_LEN_RVA, size);
        let worked = read_held(path, size);
        free();
        worked
    }
}
