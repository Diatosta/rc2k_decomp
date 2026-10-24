//! `gfx::build_fonts` (0x415563): builds the fonts' glyph tables and palettes and hangs
//! them on the font objects.
//!
//! In:  nothing
//! Out: every register preserved
//! Fatal error 0x80A48020 if it has been done (`dword_51DEB0`, the palettes, is not 0).
//! Takes `dword_51B980` * 0x400 * `dword_4F4958` bytes on the list `stru_51B984`
//! (0x80E48023 if there are none), `begin_glyph_tables`, the builder's flags = 1, 0x400
//! bytes off the list for the palettes; then five builds with `gfx::build_glyphs`, a new
//! table for each: `files\d\fnt\pal_00.pcx` (a sheet without glyphs: its palette becomes
//! the 256 pixels at the palettes' place, and is kept as components at `unk_51DEC4`),
//! `fnt13_00.pcx`, `fnt21_00.pcx`, `fnt23_00.pcx`, `fnt27_00.pcx`; `end_glyph_tables`. Then
//! 125 stores: each font object's palette (+0x14: the palettes' place plus the offset of
//! its sixteen colours) and glyph table (+0x18), as listed in `HUNG`. A font that measures
//! (mode 4) gets a palette too, which `text::draw_text` then uses as its line counter.
//! Callers: `sub_47486C` (the game's start, before the main menu), `sub_4154CD`,
//! `sub_4703C2`, `sub_41CDC2`, `sub_412DE0`, `sub_443104`.
//! A refusal from one of the steps leaves the work half done; none can come with the
//! game's own files.
//! The registers above are the original's, for reference; a call by address goes through
//! `routes.rs`.

use crate::errors::{FatalError, Refused};
use crate::framework::rva;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::build_glyphs::{BUILDER_FLAGS_RVA, TABLE_RVA, build_glyphs};
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{
    TABLES_LIST_RVA, TABLES_UNITS_RVA, begin_glyph_table, begin_glyph_tables, end_glyph_tables,
    palette_to_pixels,
};
use crate::gfx::pcx::PALETTE_RVA;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::take_from_list::{rewind_list, take_from_list};
use crate::session::SCREEN;

// ---- game data ----

/// dword_51DEB0: 256 pixels, the fonts' palettes; 0 = the fonts are not built.
pub const PALETTES_RVA: usize = rva(0x51DEB0);
const FONT27_RVA: usize = rva(0x51DEB4); // dword_51DEB4 (first the palette sheet's table)
const FONT23_RVA: usize = rva(0x51DEB8); // dword_51DEB8
const FONT21_RVA: usize = rva(0x51DEBC); // dword_51DEBC
const FONT13_RVA: usize = rva(0x51DEC0); // dword_51DEC0
const PALETTE_COPY_RVA: usize = rva(0x51DEC4); // unk_51DEC4: the palette sheet's 256 dwords
const PALETTE_BYTES: usize = 0x400;
/// The four font sheets and where each one's table is kept.
fn font_sheets() -> [(u32, usize); 4] {
    use crate::gfx::texts::{FNT13, FNT21, FNT23, FNT27, address};
    [
        (address(&FNT13), FONT13_RVA), // fnt13_00.pcx
        (address(&FNT21), FONT21_RVA), // fnt21_00.pcx
        (address(&FNT23), FONT23_RVA), // fnt23_00.pcx
        (address(&FNT27), FONT27_RVA), // fnt27_00.pcx
    ]
}

const P: usize = PALETTES_RVA;
const F13: usize = FONT13_RVA;
const F21: usize = FONT21_RVA;
const F23: usize = FONT23_RVA;
const F27: usize = FONT27_RVA;
/// The stores, in the original's order: (where, the variable stored, plus this much).
#[rustfmt::skip]
const HUNG: [(u32, usize, u32); 125] = [
    (0x51E3B8, P, 0x0),
    (0x51E3BC, F13, 0x0),
    (0x51E3F8, P, 0x300),
    (0x51E3FC, F13, 0x0),
    (0x51E3D8, P, 0x300),
    (0x51E3DC, F13, 0x0),
    (0x51E418, P, 0x300),
    (0x51E41C, F13, 0x0),
    (0x51E438, P, 0x0),
    (0x51E43C, F21, 0x0),
    (0x51E478, P, 0x180),
    (0x51E47C, F21, 0x0),
    (0x51E498, P, 0x180),
    (0x51E49C, F21, 0x0),
    (0x51E458, P, 0x180),
    (0x51E45C, F21, 0x0),
    (0x51E4D8, P, 0x200),
    (0x51E4DC, F21, 0x0),
    (0x51E4F8, P, 0x200),
    (0x51E4FC, F21, 0x0),
    (0x51E4B8, P, 0x200),
    (0x51E4BC, F21, 0x0),
    (0x51E538, P, 0x80),
    (0x51E53C, F21, 0x0),
    (0x51E558, P, 0x80),
    (0x51E55C, F21, 0x0),
    (0x51E518, P, 0x80),
    (0x51E51C, F21, 0x0),
    (0x51E598, P, 0x0),
    (0x51E59C, F21, 0x0),
    (0x51E578, P, 0x0),
    (0x51E57C, F21, 0x0),
    (0x51E5B8, P, 0x0),
    (0x51E5BC, F21, 0x0),
    (0x51E5F8, P, 0x380),
    (0x51E5FC, F21, 0x0),
    (0x51E5D8, P, 0x380),
    (0x51E5DC, F21, 0x0),
    (0x51E618, P, 0x380),
    (0x51E61C, F21, 0x0),
    (0x51E638, P, 0x0),
    (0x51E63C, F23, 0x0),
    (0x51E678, P, 0x80),
    (0x51E67C, F23, 0x0),
    (0x51E658, P, 0x80),
    (0x51E65C, F23, 0x0),
    (0x51E698, P, 0x80),
    (0x51E69C, F23, 0x0),
    (0x51E6D8, P, 0x0),
    (0x51E6DC, F23, 0x0),
    (0x51E6B8, P, 0x0),
    (0x51E6BC, F23, 0x0),
    (0x51E6F8, P, 0x0),
    (0x51E6FC, F23, 0x0),
    (0x51E738, P, 0x380),
    (0x51E73C, F23, 0x0),
    (0x51E718, P, 0x380),
    (0x51E71C, F23, 0x0),
    (0x51E758, P, 0x380),
    (0x51E75C, F23, 0x0),
    (0x51E798, P, 0x340),
    (0x51E79C, F23, 0x0),
    (0x51E778, P, 0x340),
    (0x51E77C, F23, 0x0),
    (0x51E7B8, P, 0x340),
    (0x51E7BC, F23, 0x0),
    (0x51E7F8, P, 0x180),
    (0x51E7FC, F23, 0x0),
    (0x51E7D8, P, 0x180),
    (0x51E7DC, F23, 0x0),
    (0x51E818, P, 0x180),
    (0x51E81C, F23, 0x0),
    (0x51E838, P, 0x0),
    (0x51E83C, F27, 0x0),
    (0x51E878, P, 0x80),
    (0x51E87C, F27, 0x0),
    (0x51E858, P, 0x80),
    (0x51E85C, F27, 0x0),
    (0x51E898, P, 0x80),
    (0x51E89C, F27, 0x0),
    (0x51E8D8, P, 0x0),
    (0x51E8DC, F27, 0x0),
    (0x51E8B8, P, 0x0),
    (0x51E8BC, F27, 0x0),
    (0x51E8F8, P, 0x0),
    (0x51E8FC, F27, 0x0),
    (0x51E938, P, 0x100),
    (0x51E93C, F27, 0x0),
    (0x51E918, P, 0x100),
    (0x51E91C, F27, 0x0),
    (0x51E958, P, 0x100),
    (0x51E95C, F27, 0x0),
    (0x51E998, P, 0x380),
    (0x51E99C, F27, 0x0),
    (0x51E978, P, 0x380),
    (0x51E97C, F27, 0x0),
    (0x51E9B8, P, 0x380),
    (0x51E9BC, F27, 0x0),
    (0x51EA58, P, 0x340),
    (0x51EA5C, F27, 0x0),
    (0x51EA38, P, 0x340),
    (0x51EA3C, F27, 0x0),
    (0x51EA78, P, 0x340),
    (0x51EA7C, F27, 0x0),
    (0x51E9F8, P, 0x180),
    (0x51E9FC, F27, 0x0),
    (0x51E9D8, P, 0x180),
    (0x51E9DC, F27, 0x0),
    (0x51EA18, P, 0x180),
    (0x51EA1C, F27, 0x0),
    (0x51EA98, P, 0xC0),
    (0x51EA9C, F27, 0x0),
    (0x51EABC, F27, 0x0),
    (0x51EAF8, P, 0x300),
    (0x51EAFC, F13, 0x0),
    (0x51EAD8, P, 0x300),
    (0x51EADC, F13, 0x0),
    (0x51EB18, P, 0x300),
    (0x51EB1C, F13, 0x0),
    (0x51EB58, P, 0x0),
    (0x51EB5C, F21, 0x0),
    (0x51EB38, P, 0x0),
    (0x51EB3C, F21, 0x0),
    (0x51EB78, P, 0x0),
    (0x51EB7C, F21, 0x0),
];

// ---- Rust version ----

/// The part with the calls; `palettes` is where the palettes' place is kept. (The
/// original's fatal error 0x80E48023 for a failed allocation cannot come: the allocator
/// reports its own.)
unsafe fn tables(palettes: *mut u32) -> Result<(), Refused> {
    unsafe {
        let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
        let bytes = game_read::<u32>(TABLES_UNITS_RVA)
            .wrapping_mul(0x400)
            .wrapping_mul(SCREEN.bytes.get());
        allocate(list, bytes)?;
        rewind_list(list);
        begin_glyph_tables()?;
        game_write(BUILDER_FLAGS_RVA, RUN_LENGTHS);
        palettes.write_unaligned(take_from_list(list, 0x400)?.unwrap_or(0));
        begin_glyph_table(list)?;
        build_glyphs(
            crate::gfx::texts::address(&crate::gfx::texts::PAL_00) as *const u8,
            list,
        )?;
        game_write(FONT27_RVA, game_read::<u32>(TABLE_RVA));
        palette_to_pixels(palettes.read_unaligned() as *mut u32)?;
        std::ptr::copy(
            game_addr(PALETTE_RVA) as *const u8,
            game_addr(PALETTE_COPY_RVA) as *mut u8,
            PALETTE_BYTES,
        );
        for (sheet, kept) in font_sheets() {
            begin_glyph_table(list)?;
            build_glyphs(sheet as *const u8, list)?;
            game_write(kept, game_read::<u32>(TABLE_RVA));
        }
        end_glyph_tables()?;
        Ok(())
    }
}

/// What a store of `HUNG` stores.
unsafe fn hung_value(variable: usize, plus: u32) -> u32 {
    unsafe { game_read::<u32>(variable).wrapping_add(plus) }
}

pub unsafe fn build_fonts() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(PALETTES_RVA) != 0 {
            return Err(FatalError {
                code: 0x80A4_8020,
                message: "",
            }
            .into());
        }
        tables(game_addr(PALETTES_RVA) as *mut u32)?;
        for (place, variable, plus) in HUNG {
            game_write(rva(place as usize), hung_value(variable, plus));
        }
        Ok(())
    }
}
