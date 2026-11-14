//! `menu::set_up_front_end`: what the main menu's loop does first, in this order:
//! - `menu::add_clio_car` if the Clio update is there; `menu::link_player_names`; the loading
//!   picture; `gfx::build_menu_sprites`; `menu::fit_player_names`;
//! - the music and other volume settings from the options;
//! - `saves::allocate_best_times` (the nine tables are typed, `saves::best_times`; the summary's
//!   variable is set to 1, for the race's rivals);
//! - unless the best times were loaded: the six fillers of made-up times
//!   (the checkpoint times guessed only for `default_checkpoint_times`);
//! - unless the options were loaded: `records::default_stage_records` on each of the thirteen
//!   record tables.
//!
//! One caller, `menu::main_menu_loop`. Refuses what its parts refuse; such a refusal comes with
//! the work half done. Rust callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::saves::default_times::to_the_log;
use crate::system::owner::System;

/// The record tables that get made-up records: the championship's and the twelve events'.
fn record_tables(system: &System) -> [crate::records::stages_store::RecordTable; 13] {
    let t = &system.records().stage_tables;
    let mut all = [t.championship(); 13];
    for (n, k) in [5, 4, 3, 2, 1, 0, 6, 7, 8, 9, 10, 11]
        .into_iter()
        .enumerate()
    {
        all[n + 1] = t.event_table(k);
    }
    all
}

/// The made-up times get a stage and `fit_player_names` an x, neither of which is looked at:
/// the stage is read only when the checkpoint times are guessed, which
/// `default_checkpoint_times` does without reading it; the x is where a measuring starts, which
/// a width does not depend on. The stage passed is 0.
pub fn set_up_front_end(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (menus, sound, system, gfx) = (o.menus, o.sound, o.system, o.gfx);
    let (race, saves) = (o.race, o.saves);
    let best_times = &saves.best_times;
    let stage = 0;
    use crate::saves::{
        allocate_best_times, default_best_scores, default_championship_times,
        default_checkpoint_times, default_class_times, default_stage_times,
        default_stage_times_alt,
    };
    if o.install.vars.has_clio.get() == 1 {
        crate::menu::add_clio_car::add_clio_car()?;
    }
    crate::menu::link_player_names::link_player_names(menus)?;
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    crate::gfx::build_menu_sprites::build_menu_sprites_in_game(gfx, system, menus, o.strings)?;
    crate::menu::fit_player_names::fit_player_names(menus, o.gfx)?;
    // The two dwords: the music volumes, then the tables' copies.
    sound
        .menu
        .options
        .music_volume
        .set(menus.options.snapshot().music_level);
    sound
        .menu
        .options
        .race_music_volume
        .set(menus.options.snapshot().race_music_level);
    allocate_best_times::allocate_best_times(
        best_times,
        &system.records().stage_tables,
        system.records().stage_tables.championship(),
    )?;
    // Every table is typed now (`saves::best_times`): none is copied; the summary's
    // variable says it is there, for the race's rivals.
    menus
        .play
        .champ_view
        .summary
        .set(u32::from(crate::saves::best_times::SUMMARY.is_taken()));
    if best_times.loaded.get() != 1 {
        default_best_scores::default_best_scores(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
        )?;
        default_championship_times::default_championship_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
            system,
            saves,
            stage,
            &mut to_the_log,
        )?;
        default_checkpoint_times::default_checkpoint_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
            system,
            stage,
            &mut to_the_log,
        )?;
        default_class_times::default_class_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
        )?;
        default_stage_times::default_stage_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
            system,
            stage,
            &mut to_the_log,
        )?;
        default_stage_times_alt::default_stage_times_alt(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
            system,
            stage,
            &mut to_the_log,
        )?;
        best_times.loaded.set(1);
    }
    if !menus.screens.loop_flags.options_loaded.is_set() {
        for table in record_tables(system) {
            crate::records::default_stage_records::default_stage_records(
                crate::strings::append_string_by_id::StringOwners::of(o),
                menus,
                &system.records().stage_tables,
                table,
            )?;
        }
    }
    Ok(())
}
