//! `menu::set_up_front_end` (0x47C63F): what the main menu's loop does first.
//!
//! In this order:
//! - `menu::add_clio_car` if `g_hasClio` is 1; `menu::link_player_names`; the loading
//!   picture (`menu::show_loading_picture`, not ported); `gfx::build_menu_sprites`;
//!   `menu::fit_player_names` (with the ecx the loading picture left);
//! - the first volume setting = `option_work.music_level` and the game music volume = `option_work.other_volume`;
//! - `saves::allocate_best_times` (the original then copies the nine tables' addresses to
//!   the variables the screens read; the tables are typed now, `saves::best_times`, and
//!   only the summary's variable is set: 1 while it is there, for the race's rivals);
//! - unless the best times were loaded (`g_bestTimesLoaded`): the six fillers of made-up
//!   times, `draw_work.scratch_0c` (the checkpoints' switch) set before each (1 only for
//!   `default_checkpoint_times`), then `g_bestTimesLoaded` = 1;
//! - unless the options were loaded (`g_optionsLoaded`): `records::default_stage_records`
//!   on each of the thirteen record tables.
//!
//! One caller, `menu::main_menu_loop`. Refuses what its parts refuse; such a refusal comes with the
//! work half done. Rust callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::saves::default_times::to_the_log;
use crate::system::owner::System;

// ---- game data ----

/// The record tables that get made-up records: the championship's and the twelve events'
/// (IDA 0x739642, 0x7391C2, 0x738D42, 0x7388C2, 0x738442, 0x737FC2, then 0x739AC2 ..
/// 0x73B142), in the original's order.
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

/// The original's ebp and ecx at entry reach the made-up times as their stage and
/// `fit_player_names` as its x. Neither is looked at: the stage only while `DrawWork.scratch_0c` is
/// 1, which it is only for `default_checkpoint_times`, which does not read it; the x is
/// where a measuring starts, which a width does not depend on. The stage passed is 0.
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
        crate::menu::add_clio_car::add_clio_car(menus)?;
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
        .set(menus.screens.option_work.music_level.get());
    sound
        .menu
        .options
        .race_music_volume
        .set(menus.screens.option_work.other_volume.get());
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
        menus.screens.draw_work.scratch_0c.set(0u32);
        default_best_scores::default_best_scores(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
        )?;
        menus.screens.draw_work.scratch_0c.set(0u32);
        default_championship_times::default_championship_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
            system,
            saves,
            stage,
            &mut to_the_log,
        )?;
        menus.screens.draw_work.scratch_0c.set(1u32);
        default_checkpoint_times::default_checkpoint_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
            system,
            stage,
            &mut to_the_log,
        )?;
        menus.screens.draw_work.scratch_0c.set(0u32);
        default_class_times::default_class_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
        )?;
        menus.screens.draw_work.scratch_0c.set(0u32);
        default_stage_times::default_stage_times(
            crate::strings::append_string_by_id::StringOwners::of(o),
            menus,
            race,
            system,
            stage,
            &mut to_the_log,
        )?;
        menus.screens.draw_work.scratch_0c.set(0u32);
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
