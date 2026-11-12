//! `race::viewer_tidy_stage` (0x41CEB4): the viewer's stage, if loaded (`viewer_stage_loaded` 1, then 0),
//! tidied away and the front end back: `race::tidy_everything`, the menus' screen mode
//! (`gfx::set_menu_screen_mode`), the loading picture (`menu::show_loading_picture`),
//! `race::clear_race_sound_objects`, `sound::apply_settings`; the fonts freed (`gfx::free_fonts`)
//! when the viewer built them (`viewer_fonts_built` 1); `scene::set_up_engine`,
//! `scene::set_up_front_end_scene`, `scene::set_up_engine`. Then (always) the pending key taken
//! (`input::take_key`).
//!
//! `viewer_fonts_built` is never cleared: where `mov` was meant the original has `cmp `viewer_fonts_built`, 0`
//! (0x41CEEA). Kept (section 16). Callers: `race::replay_viewer`, `race::viewer_action` (0x14),
//! `race::choose_save_slot`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;

pub fn viewer_tidy_stage(race: &Race, o: Owners<'static>) -> Result<(), Refused> {
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer_flow::{tidies, TIDY_FIRST, frees_fonts,
    // TIDY_AFTER}` (the fonts' switch read after the first steps, as the original).
    use rc2k_core::race::viewer_flow::{TIDY_AFTER, TIDY_FIRST, TidyStep, frees_fonts, tidies};
    if tidies(shared.replay_file.viewer_stage_loaded.get()) {
        shared.replay_file.viewer_stage_loaded.set(0);
        for step in TIDY_FIRST {
            run(race, o, step)?;
        }
        if frees_fonts(shared.replay_file.viewer_fonts_built.get()) {
            run(race, o, TidyStep::FreeFonts)?;
        }
        for step in TIDY_AFTER {
            run(race, o, step)?;
        }
    }
    run(race, o, TidyStep::TakeKey)
}

fn run(
    race: &Race,
    o: Owners<'static>,
    step: rc2k_core::race::viewer_flow::TidyStep,
) -> Result<(), Refused> {
    let fonts = &o.gfx.fonts;
    use crate::race::{race_end, sound_tables};
    use rc2k_core::race::viewer_flow::TidyStep;
    match step {
        TidyStep::TidyEverything => race_end::tidy_everything(race, o)?,
        TidyStep::MenuScreenMode => {
            crate::platform::set_menu_mode_now();
        }
        TidyStep::LoadingPicture => {
            crate::menu::show_loading_picture::show_loading_picture_in(o.menus, o.gfx, o.system)?;
        }
        TidyStep::ClearRaceSounds => {
            sound_tables::clear_race_sound_objects(o.sound, o.audio, o.system)?;
        }
        TidyStep::SoundSettings => crate::sound::apply_settings::apply_settings(o)?,
        TidyStep::FreeFonts => crate::gfx::free_fonts::free_fonts(
            &fonts.glyph_tables.memory,
            &fonts.font_block.palettes,
        )?,
        TidyStep::SetUpEngine => crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?,
        TidyStep::SetUpFrontEnd => {
            crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
        }
        TidyStep::TakeKey => {
            crate::input::take_key::take_key();
        }
    }
    Ok(())
}
