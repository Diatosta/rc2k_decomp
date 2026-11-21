//! `race::viewer_tidy_stage`: tidies away the viewer's stage, if loaded (`viewer_stage_loaded` 1,
//! then set to 0), and brings the front end back: `race::tidy_everything`, the menus' screen mode
//! (`gfx::set_menu_screen_mode`), the loading picture (`menu::show_loading_picture`),
//! `race::clear_race_sound_objects` and `sound::apply_settings`; the fonts are freed
//! (`gfx::free_fonts`) when the viewer built them (`viewer_fonts_built` 1); then
//! `scene::set_up_engine`, `scene::set_up_front_end_scene` and `scene::set_up_engine`. In every
//! case the pending key is taken last (`input::take_key`).
//!
//! `viewer_fonts_built` is never cleared: a 1999 slip kept as is.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;

pub fn viewer_tidy_stage(race: &Race, o: Owners<'static>) -> Result<(), Refused> {
    let shared = &race.shared;
    // The fonts' switch is read after the first steps.
    use rc2k_core::race::viewer_flow::{TIDY_AFTER, TIDY_FIRST, TidyStep, frees_fonts, tidies};
    if tidies(shared.replay_file().viewer_stage_loaded.get()) {
        shared.replay_file().viewer_stage_loaded.set(0);
        for step in TIDY_FIRST {
            run(race, o, step)?;
        }
        if frees_fonts(shared.replay_file().viewer_fonts_built.get()) {
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
            crate::platform::set_menu_mode_now(o.gfx, o.system);
        }
        TidyStep::LoadingPicture => {
            crate::menu::show_loading_picture::show_loading_picture_in(o)?;
        }
        TidyStep::ClearRaceSounds => {
            sound_tables::clear_race_sound_objects(o)?;
        }
        TidyStep::SoundSettings => crate::sound::apply_settings::apply_settings(o)?,
        TidyStep::FreeFonts => crate::gfx::free_fonts::free_fonts(fonts)?,
        TidyStep::SetUpEngine => crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?,
        TidyStep::SetUpFrontEnd => {
            crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
        }
        TidyStep::TakeKey => {
            crate::input::take_key::take_key(o.input);
        }
    }
    Ok(())
}
