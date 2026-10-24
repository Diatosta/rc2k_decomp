//! What the callers of `gfx::build_glyphs` do around it, as Rust functions, and the calls
//! they make as a trait, so that a self-test can stand in for them.
//!
//! `begin_glyph_tables` is `sub_412014` (0x412014), `begin_glyph_table` `sub_411FEC`
//! (0x411FEC), `end_glyph_tables` `sub_412077` (0x412077), `palette_to_pixels`
//! `sub_411FC8` (0x411FC8) with `sub_405E02` under it, `alpha_palette` `sub_412528`
//! (0x412528).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::gfx::build_glyphs::{
    BUILDER_FLAGS_RVA, FORMAT_RVA, TABLE_RVA, TABLE_SIZE_RVA, WORK_MEMORY_RVA,
};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::glyph_builder::DIRECT_COLOUR;
use crate::gfx::pcx::PALETTE_RVA;
use crate::gfx::picture::{HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA};
use crate::gfx::pixel_format::{PixelFormat, SURFACE_FORMAT_RVA};
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::memory::take_from_list::take_from_list;

/// stru_51B984: the list the fonts' and the sprites' tables lie on, and dword_51B980: its
/// size in units of 0x400 bytes times `dword_4F4958`.
pub const TABLES_LIST_RVA: usize = rva(0x51B984);
pub const TABLES_UNITS_RVA: usize = rva(0x51B980);
pub const UNIT_FACTOR_RVA: usize = rva(0x4F4958); // dword_4F4958
const WORK_UNITS_RVA: usize = rva(0x519554); // dword_519554

/// The work memory taken, the picture's size set for sheets, the pixel layout the
/// surface's, and no table yet.
pub unsafe fn begin_glyph_tables() -> Result<(), Refused> {
    unsafe {
        game_write(WIDTH_RVA, 800u32);
        game_write(HEIGHT_RVA, 600u32);
        game_write(ROW_BYTES_RVA, 0xFFFF_FFFDu32);
        game_write(FORMAT_RVA, game_addr(SURFACE_FORMAT_RVA) as u32);
        let bytes = game_read::<u32>(WORK_UNITS_RVA)
            .wrapping_mul(0x400)
            .wrapping_mul(game_read(UNIT_FACTOR_RVA));
        // (The original's fatal error 0x80E48021 for no memory cannot come: the allocator
        // reports its own.)
        allocate(game_addr(WORK_MEMORY_RVA) as *mut MemoryList, bytes)?;
        game_write(TABLE_RVA, 0u32);
        game_write(TABLE_SIZE_RVA, 0u32);
        Ok(())
    }
}

/// A new table starts where the list stands, after the padding that brings the table
/// before it to whole dwords.
pub unsafe fn begin_glyph_table(list: *mut MemoryList) -> Result<(), BadInput> {
    unsafe {
        let padding = game_read::<u32>(TABLE_SIZE_RVA).wrapping_neg() & 3;
        take_from_list(list, padding)?;
        game_write(TABLE_RVA, take_from_list(list, 0)?.unwrap_or(0));
        game_write(TABLE_SIZE_RVA, 0u32);
        Ok(())
    }
}

pub unsafe fn end_glyph_tables() -> Result<(), BadInput> {
    unsafe { free_list(game_addr(WORK_MEMORY_RVA) as *mut MemoryList) }
}

/// The last sheet's palette as 256 pixels of the builder's layout at `to`; nothing if `to`
/// is 0 or the last sheet was one of components.
pub unsafe fn palette_to_pixels(to: *mut u32) -> Result<(), BadInput> {
    unsafe {
        if to.is_null() || game_read::<u32>(BUILDER_FLAGS_RVA) & DIRECT_COLOUR != 0 {
            return Ok(());
        }
        let format = game_read::<u32>(FORMAT_RVA) as *const PixelFormat;
        if format.is_null() {
            return Err(BadInput("no pixel layout for the palette"));
        }
        let format = format.read_unaligned();
        for entry in 0..256 {
            let components: u32 = game_read(PALETTE_RVA + entry * 4);
            to.add(entry)
                .write_unaligned(components_to_pixel(&format, components));
        }
        Ok(())
    }
}

/// The last sheet's palette for pixels that are a colour in the low four bits and an alpha
/// in the top four: entry 0 becomes 0, every entry the colour of its low four bits'
/// entry, and its fourth byte the alpha, the four bits twice.
pub unsafe fn alpha_palette() {
    unsafe {
        let palette = game_addr(PALETTE_RVA) as *mut u32;
        palette.write_unaligned(0);
        for entry in 0..256usize {
            let colour = palette.add(entry & 0x0F).read_unaligned();
            let alpha = (entry & 0xF0 | entry >> 4) as u32;
            palette
                .add(entry)
                .write_unaligned(colour & 0x00FF_FFFF | alpha << 24);
        }
    }
}
