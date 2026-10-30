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
//! The only register it reads at entry is esi, which goes to `tidy_everything` and
//! `set_menu_screen_mode` (attrib), whose Rust versions read none.

use crate::errors::Refused;
use crate::gfx::fonts_store::FONTS;

use crate::race::replay_file_store::REPLAY_FILE;

/// The Rust version. `set_up_engine` and `set_up_front_end_scene` run on a scratch
/// register set (they read none of the caller's: attrib finds only esi read at entry).
pub fn viewer_tidy_stage() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{tidies, TIDY_FIRST, frees_fonts,
    // TIDY_AFTER}` (the fonts' switch read after the first steps, as the original).
    use rc2k_core::race::viewer_flow::{TIDY_AFTER, TIDY_FIRST, TidyStep, frees_fonts, tidies};
    if tidies(REPLAY_FILE.viewer_stage_loaded.get()) {
        REPLAY_FILE.viewer_stage_loaded.set(0);
        for step in TIDY_FIRST {
            run(step)?;
        }
        if frees_fonts(REPLAY_FILE.viewer_fonts_built.get()) {
            run(TidyStep::FreeFonts)?;
        }
        for step in TIDY_AFTER {
            run(step)?;
        }
    }
    run(TidyStep::TakeKey)
}

fn run(step: rc2k_core::race::viewer_flow::TidyStep) -> Result<(), Refused> {
    use crate::race::{race_end, sound_tables};
    use rc2k_core::race::viewer_flow::TidyStep;
    match step {
        TidyStep::TidyEverything => race_end::tidy_everything()?,
        TidyStep::MenuScreenMode => {
            crate::platform::set_menu_mode_now();
        }
        TidyStep::LoadingPicture => {
            crate::menu::show_loading_picture::show_loading_picture()?;
        }
        TidyStep::ClearRaceSounds => sound_tables::clear_race_sound_objects()?,
        TidyStep::SoundSettings => crate::sound::apply_settings::apply_settings()?,
        TidyStep::FreeFonts => crate::gfx::free_fonts::free_fonts(
            crate::pointers::GamePtr::to(&crate::gfx::glyph_tables::TABLES.memory),
            &FONTS.palettes,
        )?,
        TidyStep::SetUpEngine => crate::scene::set_up_engine::set_up_engine()?,
        TidyStep::SetUpFrontEnd => crate::scene::set_up_front_end_scene::set_up_front_end_scene()?,
        TidyStep::TakeKey => {
            crate::input::take_key::take_key();
        }
    }
    Ok(())
}
