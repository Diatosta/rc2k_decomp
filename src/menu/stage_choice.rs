//! The stage choice's own steps (IDA, 2026-10-15): the UK map with the chosen stage
//! (`menu::draw_stage_map`, 14 callers), and the choice's three settings stepped, each then
//! drawing the map again.

use crate::errors::{FatalError, Refused};
use crate::gfx::picture_store::PICTURE;
use crate::gfx::screen_state::SCREEN_MODE;
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::screens::draw_stage_records::selection;
use crate::session::Dword;

const RECORD_SIZE: u32 = 0xA4;

pub const GROUP_RVA: &Dword = &EVENT_STATE.stage_group; // dword_7558C8: 0 or 1
pub const SUB_RVA: &Dword = &EVENT_STATE.stage_sub; // dword_7558D0: 0 .. 5
pub const NUMBER_RVA: &Dword = &EVENT_STATE.stage_number; // dword_7558CC: 1 .. 6
const MAP_SPRITE: u32 = 0xA0;
/// The map that cannot be loaded: no message.
const NO_MAP: FatalError = FatalError {
    code: 0x8071_2345,
    message: "",
};

fn dword(at: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(at).read::<u32>("a game value")
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
        let mode: u32 = CHAMP_VIEW.stage_mode.get();
        let record = match mode {
            2 => {
                let table: u32 = ARCADE_GAME.level.get();
                let at = ARCADE_GAME.stage.get().wrapping_mul(RECORD_SIZE);
                crate::records::stages_store::STAGE_TABLES
                    .level_table(table)
                    .wrapping_add(at)
            }
            3 => crate::records::find_record::find_record_in(
                crate::records::stages_store::STAGE_TABLES.championship(),
                EVENT_STATE.index.get() as i32,
                EVENT_STATE.section.get(),
            )?,
            _ => {
                let s = selection()?.ok_or(crate::errors::BadInput(
                    "no stage tables for the choice: the original reads through null",
                ))?;
                s.table
                    .wrapping_add(EVENT_STATE.stage_number.get().wrapping_mul(RECORD_SIZE))
            }
        };
        let overlay =
            dword(MENU_LISTS.map_overlays[0].addr() + dword(record + 0x3C)?.wrapping_mul(4))?;
        if load_picture_with_overlay(
            crate::pointers::host(crate::menu::texts_store::MENU_TEXTS.ukmap_png.addr()),
            crate::pointers::host(overlay),
        )? {
            return Err(NO_MAP.into());
        }
        convert_picture_now()?;
        PICTURE.flags.set(SCREEN_MODE.mode_flags.get() & 0xFFF);
        picture_as_surface()?;
        // The overlay's pointer is the blit's palette too (ebx all the way).
        blit_glyph_on_surface(MAP_SPRITE, LOOP_FLAGS.sprite_table.get(), 0, 0, overlay)?;
    }
    Ok(())
}

/// How a setting draws the map: the Rust version in the game, a counter in the tests.
pub type Draw<'a> = &'a mut dyn FnMut() -> Result<(), Refused>;
/// The choice's settings (converted: `rc2k_core::menu::steps::stage_sub` and
/// `stage_number`): the variable stepped, then the map drawn again (`draw`: the Rust
/// version in the game). When the map refuses, the variable is put back first, so that the
/// original, which then runs, steps it once.
fn step(cell: &Dword, next: impl Fn(u32) -> u32, draw: Draw) -> Result<(), Refused> {
    let old = cell.get();
    cell.set(next(old));
    draw().inspect_err(|_| cell.set(old))
}

/// `menu::switch_stage_group`: the group 0 <-> 1.
pub fn switch_stage_group(draw: Draw) -> Result<(), Refused> {
    step(GROUP_RVA, |g| g ^ 1, draw)
}

/// `menu::next_stage_sub`: 5 or more (unsigned) to 0, else on.
pub fn next_stage_sub(draw: Draw) -> Result<(), Refused> {
    step(
        SUB_RVA,
        |s| rc2k_core::menu::steps::stage_sub(s, true),
        draw,
    )
}

/// `menu::previous_stage_sub`: below 1 (signed) to 5, else back.
pub fn previous_stage_sub(draw: Draw) -> Result<(), Refused> {
    step(
        SUB_RVA,
        |s| rc2k_core::menu::steps::stage_sub(s, false),
        draw,
    )
}

/// `menu::next_stage_number`: 6 or more (unsigned) to 1, else on.
pub fn next_stage_number(draw: Draw) -> Result<(), Refused> {
    step(
        NUMBER_RVA,
        |n| rc2k_core::menu::steps::stage_number(n, true),
        draw,
    )
}

/// `menu::previous_stage_number`: below 2 (signed) to 6, else back one.
pub fn previous_stage_number(draw: Draw) -> Result<(), Refused> {
    step(
        NUMBER_RVA,
        |n| rc2k_core::menu::steps::stage_number(n, false),
        draw,
    )
}
