//! `menu::set_up_front_end` (0x47C63F): what the main menu's loop does first.
//!
//! In:  nothing (ebp and, after the loading picture, ecx are passed on as they are)
//! Out: every register preserved
//! In this order:
//! - `menu::add_clio_car` if `g_hasClio` is 1; `menu::link_player_names`; the loading
//!   picture (`sub_47CEA8`, not ported); `gfx::build_menu_sprites`;
//!   `menu::fit_player_names` (with the ecx the loading picture left);
//! - `dword_5180FC` = `dword_755688` and `dword_518100` = `dword_75568C`;
//! - `saves::allocate_best_times` (the original then copies the nine tables' addresses to
//!   the variables the screens read; the tables are typed now, `saves::best_times`, and
//!   only the summary's variable is set: 1 while it is there, for the race's rivals);
//! - unless the best times were loaded (`g_bestTimesLoaded`): the six fillers of made-up
//!   times, `dword_755A0C` (the checkpoints' switch) set before each (1 only for
//!   `default_checkpoint_times`), then `g_bestTimesLoaded` = 1;
//! - unless the options were loaded (`g_optionsLoaded`): `records::default_stage_records`
//!   on each of the thirteen record tables.
//!
//! One caller, `menu::main_menu_loop`. Refuses what its parts refuse; such a refusal comes with the
//! work half done. Rust callers run its `entry` (`routes.rs`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::saves::default_times::to_the_log;

// ---- game data ----

/// The record tables that get made-up records: the championship's and the twelve events'
/// (IDA 0x739642, 0x7391C2, 0x738D42, 0x7388C2, 0x738442, 0x737FC2, then 0x739AC2 ..
/// 0x73B142), in the original's order.
fn record_tables() -> [crate::records::stages_store::RecordTable; 13] {
    let t = &crate::records::stages_store::STAGE_TABLES;
    let mut all = [t.championship(); 13];
    for (n, k) in [5, 4, 3, 2, 1, 0, 6, 7, 8, 9, 10, 11]
        .into_iter()
        .enumerate()
    {
        all[n + 1] = t.event_table(k);
    }
    all
}

// ---- Rust version ----

/// The original's ebp and ecx at entry reach the made-up times as their stage and
/// `fit_player_names` as its x. Neither is looked at: the stage only while `dword_755A0C` is
/// 1, which it is only for `default_checkpoint_times`, which does not read it; the x is
/// where a measuring starts, which a width does not depend on. The stage passed is 0.
pub fn set_up_front_end(menus: &Menus) -> Result<(), Refused> {
    let stage = 0;
    use crate::saves::{
        allocate_best_times, default_best_scores, default_championship_times,
        default_checkpoint_times, default_class_times, default_stage_times,
        default_stage_times_alt,
    };
    if crate::install::owner::install().vars.has_clio.get() == 1 {
        crate::menu::add_clio_car::add_clio_car()?;
    }
    crate::menu::link_player_names::link_player_names()?;
    crate::menu::show_loading_picture::show_loading_picture()?;
    crate::gfx::build_menu_sprites::build_menu_sprites_in_game()?;
    crate::menu::fit_player_names::fit_player_names()?;
    // The two dwords: the music volumes, then the tables' copies.
    crate::sound::owner::sound()
        .menu
        .options
        .music_volume
        .set(menus.screens.option_work.music_level.get());
    crate::sound::owner::sound()
        .menu
        .options
        .race_music_volume
        .set(menus.screens.option_work.other_volume.get());
    allocate_best_times::allocate_best_times(
        crate::saves::owner::saves().best_times.list(),
        crate::records::stages_store::STAGE_TABLES.championship(),
    )?;
    // Every table is typed now (`saves::best_times`): none is copied; the summary's
    // variable says it is there, for the race's rivals.
    menus
        .play
        .champ_view
        .summary
        .set(u32::from(crate::saves::best_times::SUMMARY.is_taken()));
    if crate::saves::owner::saves().best_times.loaded.get() != 1 {
        menus.screens.draw_work.work_0c.set(0u32);
        default_best_scores::default_best_scores()?;
        menus.screens.draw_work.work_0c.set(0u32);
        default_championship_times::default_championship_times(stage, &mut to_the_log)?;
        menus.screens.draw_work.work_0c.set(1u32);
        default_checkpoint_times::default_checkpoint_times(stage, &mut to_the_log)?;
        menus.screens.draw_work.work_0c.set(0u32);
        default_class_times::default_class_times()?;
        menus.screens.draw_work.work_0c.set(0u32);
        default_stage_times::default_stage_times(stage, &mut to_the_log)?;
        menus.screens.draw_work.work_0c.set(0u32);
        default_stage_times_alt::default_stage_times_alt(stage, &mut to_the_log)?;
        crate::saves::owner::saves().best_times.loaded.set(1);
    }
    if menus.screens.loop_flags.options_loaded.get() != 1 {
        for table in record_tables() {
            crate::records::default_stage_records::default_stage_records(table)?;
        }
    }
    Ok(())
}
