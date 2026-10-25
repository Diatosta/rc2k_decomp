//! The stage choice's own steps (IDA, 2026-10-15): the UK map with the chosen stage
//! (`menu::draw_stage_map`, 14 callers), and the choice's three settings stepped, each then
//! drawing the map again.

use crate::errors::{FatalError, Refused};
use crate::framework::rva;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::picture_store::Slot as PicSlot;
use crate::menu::lists_store::MENU_LISTS;
use crate::screens::draw_stage_records::selection;

pub const MODE_RVA: usize = rva(0x755700); // dword_755700: 2 a championship's, 3 a record's
pub const GROUP_RVA: usize = rva(0x7558C8); // dword_7558C8: 0 or 1
pub const SUB_RVA: usize = rva(0x7558D0); // dword_7558D0: 0 .. 5
pub const NUMBER_RVA: usize = rva(0x7558CC); // dword_7558CC: 1 .. 6
const CHAMPIONSHIP_TABLES_RVA: usize = rva(0x73555A); // off_73555A
const CHAMPIONSHIP_TABLE_RVA: usize = rva(0x7587EC); // dword_7587EC
const CHAMPIONSHIP_STAGE_RVA: usize = rva(0x7587F0); // dword_7587F0
const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
const SECTION_RVA: usize = rva(0x75595C); // section
const INDEX_RVA: usize = rva(0x755958); // index
const MAP_PATH_RVA: usize = rva(0x73218B); // "files\g\graphics\screens\ukmap.png"
const SCREEN_MODE_FLAGS_RVA: usize = rva(0x4F4934); // g_screenModeFlags
const PICTURE_FLAGS_RVA: PicSlot = PicSlot::at(0x4F49CC); // g_pictureFlags
const SPRITE_TABLE_RVA: usize = rva(0x755581); // g_spriteTable
const RECORD_SIZE: u32 = 0xA4;
const MAP_SPRITE: u32 = 0xA0;
/// The map that cannot be loaded: no message.
const NO_MAP: FatalError = FatalError {
    code: 0x8071_2345,
    message: "",
};

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// `menu::draw_stage_map`: the stage by `dword_755700` (2: `off_73555A[dword_7587EC]`'s
/// `dword_7587F0`th; 3: `records::find_record` of `g_recordTable` with `section` and
/// `index`; else the choice's: `off_735512[group][sub]`'s `number`th), its overlay
/// `off_7598AE[+0x3C]`; `ukmap.png` loaded with it (the carry: fatal error 0x80712345),
/// converted, `g_pictureFlags` = `g_screenModeFlags` & 0xFFF, made the surface, sprite 0xA0
/// at (0, 0). Refuses a choice without its tables (the original reads through null).
pub unsafe fn draw_stage_map() -> Result<(), Refused> {
    use crate::gfx::blit_glyph::blit_glyph_on_surface;
    use crate::gfx::convert_picture::convert_picture_now;
    use crate::gfx::load_picture_with_overlay::load_picture_with_overlay;
    use crate::gfx::picture_as_surface::picture_as_surface;
    unsafe {
        let mode: u32 = game_read(MODE_RVA);
        let record = match mode {
            2 => {
                let table: u32 = game_read(CHAMPIONSHIP_TABLE_RVA);
                let at = game_read::<u32>(CHAMPIONSHIP_STAGE_RVA).wrapping_mul(RECORD_SIZE);
                dword(game_addr(CHAMPIONSHIP_TABLES_RVA) as u32 + table.wrapping_mul(4))
                    .wrapping_add(at)
            }
            3 => crate::records::find_record::find_record_in(
                game_addr(RECORD_TABLE_RVA) as u32,
                game_read::<u32>(INDEX_RVA) as i32,
                game_read(SECTION_RVA),
            ),
            _ => {
                let s = selection().ok_or(crate::errors::BadInput(
                    "no stage tables for the choice: the original reads through null",
                ))?;
                s.table
                    .wrapping_add(game_read::<u32>(NUMBER_RVA).wrapping_mul(RECORD_SIZE))
            }
        };
        let overlay =
            dword(MENU_LISTS.map_overlays[0].addr() as u32 + dword(record + 0x3C).wrapping_mul(4));
        if load_picture_with_overlay(game_addr(MAP_PATH_RVA) as *mut u8, overlay as *mut u8)? {
            return Err(NO_MAP.into());
        }
        convert_picture_now()?;
        game_write(
            PICTURE_FLAGS_RVA,
            game_read::<u32>(SCREEN_MODE_FLAGS_RVA) & 0xFFF,
        );
        picture_as_surface()?;
        // The overlay's pointer is the blit's palette too (ebx all the way).
        blit_glyph_on_surface(
            MAP_SPRITE,
            game_read::<u32>(SPRITE_TABLE_RVA) as *const u8,
            0,
            0,
            overlay as *const u32,
        )?;
    }
    Ok(())
}

/// How a setting draws the map: the Rust version in the game, a counter in the tests.
pub type Draw<'a> = &'a mut dyn FnMut() -> Result<(), Refused>;
/// The choice's settings (converted: `rc2k_core::menu::steps::stage_sub` and
/// `stage_number`): the variable stepped, then the map drawn again (`draw`: the Rust
/// version in the game). When the map refuses, the variable is put back first, so that the
/// original, which then runs, steps it once.
unsafe fn step(rva: usize, next: impl Fn(u32) -> u32, draw: Draw) -> Result<(), Refused> {
    unsafe {
        let old: u32 = game_read(rva);
        game_write(rva, next(old));
        draw().inspect_err(|_| game_write(rva, old))
    }
}

/// `menu::switch_stage_group`: the group 0 <-> 1.
pub unsafe fn switch_stage_group(draw: Draw) -> Result<(), Refused> {
    unsafe { step(GROUP_RVA, |g| g ^ 1, draw) }
}

/// `menu::next_stage_sub`: 5 or more (unsigned) to 0, else on.
pub unsafe fn next_stage_sub(draw: Draw) -> Result<(), Refused> {
    unsafe {
        step(
            SUB_RVA,
            |s| rc2k_core::menu::steps::stage_sub(s, true),
            draw,
        )
    }
}

/// `menu::previous_stage_sub`: below 1 (signed) to 5, else back.
pub unsafe fn previous_stage_sub(draw: Draw) -> Result<(), Refused> {
    unsafe {
        step(
            SUB_RVA,
            |s| rc2k_core::menu::steps::stage_sub(s, false),
            draw,
        )
    }
}

/// `menu::next_stage_number`: 6 or more (unsigned) to 1, else on.
pub unsafe fn next_stage_number(draw: Draw) -> Result<(), Refused> {
    unsafe {
        step(
            NUMBER_RVA,
            |n| rc2k_core::menu::steps::stage_number(n, true),
            draw,
        )
    }
}

/// `menu::previous_stage_number`: below 2 (signed) to 6, else back one.
pub unsafe fn previous_stage_number(draw: Draw) -> Result<(), Refused> {
    unsafe {
        step(
            NUMBER_RVA,
            |n| rc2k_core::menu::steps::stage_number(n, false),
            draw,
        )
    }
}
