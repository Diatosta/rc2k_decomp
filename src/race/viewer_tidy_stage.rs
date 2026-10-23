//! `race::viewer_tidy_stage` (0x41CEB4): the viewer's stage, if loaded (`dword_5BECC0` 1,
//! then 0), tidied away and the front end back: `race::tidy_everything`, the menus'
//! screen mode (`gfx::set_menu_screen_mode`), the loading picture
//! (`menu::show_loading_picture`), `race::clear_race_sound_objects`,
//! `sound::apply_settings`; the fonts freed (`gfx::free_fonts`) when the viewer built
//! them (`dword_5BECC4` 1); `scene::set_up_engine`, `scene::set_up_front_end_scene`,
//! `scene::set_up_engine`. Then (always) the pending key taken (`input::take_key`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! `dword_5BECC4` is never cleared: where `mov` was meant the original has
//! `cmp dword_5BECC4, 0` (0x41CEEA). Kept (section 16).
//! Callers: `race::replay_viewer`, `race::viewer_action` (0x14), `race::choose_save_slot`.
//! Its calls through `part` (places in `CALLEES`); in the game their Rust versions, the
//! screen mode (the platform), the loading picture and the sound settings through their
//! hooks. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version: not loaded, loaded with and without the fonts
//! built.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::race::viewer::{Part, get, put, regs_entry};
use crate::race::viewer_load_stage::{FONTS_BUILT, take_key};

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 10] = [
    0, 0x444AC7, // 1 race::tidy_everything
    0x404823, // 2 gfx::set_menu_screen_mode
    0x47CEA8, // 3 menu::show_loading_picture
    0x415C6D, // 4 race::clear_race_sound_objects
    0x40EFF3, // 5 sound::apply_settings
    0x415C55, // 6 gfx::free_fonts
    0x442AF6, // 7 scene::set_up_engine
    0x474E04, // 8 scene::set_up_front_end_scene
    0x407F95, // 9 input::take_key
];
const LOADED: usize = 0x5BECC0; // dword_5BECC0

/// The Rust version: the registers it leaves (the entry's).
pub fn viewer_tidy_stage(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{tidies, TIDY_FIRST, frees_fonts,
    // TIDY_AFTER}` (the fonts' switch read after the first steps, as the original).
    use rc2k_core::race::viewer_flow::{TIDY_AFTER, TIDY_FIRST, TidyStep, frees_fonts, tidies};
    let place = |step| match step {
        TidyStep::TidyEverything => 1,
        TidyStep::MenuScreenMode => 2,
        TidyStep::LoadingPicture => 3,
        TidyStep::ClearRaceSounds => 4,
        TidyStep::SoundSettings => 5,
        TidyStep::FreeFonts => 6,
        TidyStep::SetUpEngine => 7,
        TidyStep::SetUpFrontEnd => 8,
        TidyStep::TakeKey => 9,
    };
    let mut r = *entry;
    if tidies(get(LOADED)) {
        put(LOADED, 0);
        for step in TIDY_FIRST {
            part(place(step), &mut r)?;
        }
        if frees_fonts(get(FONTS_BUILT)) {
            part(place(TidyStep::FreeFonts), &mut r)?;
        }
        for step in TIDY_AFTER {
            part(place(step), &mut r)?;
        }
    }
    part(place(TidyStep::TakeKey), &mut r)?;
    Ok(*entry)
}

/// In the game: the Rust versions; the screen mode, the loading picture and the sound
/// settings through the game (the first the platform's, the others their hooks).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::{loadlevel::LoadVars, race_end, sound_tables};
    unsafe {
        match place {
            1 => *r = race_end::tidy_everything(r, &LoadVars::game(), &mut race_end::tidy_part)?,
            2 | 3 | 5 => crate::calls::call(rva(CALLEES[place as usize]), r),
            4 => {
                sound_tables::clear_race_sound_objects(r, &mut sound_tables::object_part)?;
            }
            6 => crate::gfx::free_fonts::free_fonts(
                game_addr(crate::gfx::glyph_tables::TABLES_LIST_RVA) as *mut _,
                game_addr(crate::gfx::build_fonts::PALETTES_RVA) as *mut u32,
            )?,
            7 => crate::scene::set_up_engine::set_up_engine_in_game(r),
            8 => crate::scene::set_up_front_end_scene::set_up_front_end_scene_in_game(r)?,
            _ => take_key(r),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| viewer_tidy_stage(r, &mut game_part))
}
