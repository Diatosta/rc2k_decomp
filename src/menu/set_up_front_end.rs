//! `menu::set_up_front_end` (0x47C63F): what the main menu's loop does first.
//!
//! In:  nothing (ebp and, after the loading picture, ecx are passed on as they are)
//! Out: every register preserved
//! In this order:
//! - `menu::add_clio_car` if `g_hasClio` is 1; `menu::link_player_names`; the loading
//!   picture (`sub_47CEA8`, not ported); `gfx::build_menu_sprites`;
//!   `menu::fit_player_names` (with the ecx the loading picture left);
//! - `dword_5180FC` = `dword_755688` and `dword_518100` = `dword_75568C`;
//! - `saves::allocate_best_times`, and the nine tables' addresses copied to the variables
//!   the screens read them from (`COPIES`);
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
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::saves::best_times_store::BEST_TIMES;
use crate::saves::default_times::to_the_log;

// ---- game data ----

/// (from, to): two dwords, then the nine tables of `saves::allocate_best_times`.
const COPIES: [(usize, usize); 11] = [
    (0x755688, 0x5180FC),
    (0x75568C, 0x518100),
    (0x747618, 0x7556F4),
    (0x747620, 0x7556D4),
    (0x747628, 0x7556D8),
    (0x747630, 0x7556DC),
    (0x747638, 0x7556E0),
    (0x747640, 0x7556EC),
    (0x747648, 0x7556F0),
    (0x747650, 0x7556E4),
    (0x747658, 0x7556E8),
];
/// The record tables that get made-up records: the championship's and the twelve events'
/// (IDA 0x739642, 0x7391C2, 0x738D42, 0x7388C2, 0x738442, 0x737FC2, then 0x739AC2 ..
/// 0x73B142), in the original's order.
fn record_tables() -> [u32; 13] {
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

/// `stage` and `x` are ebp and ecx at entry: the stage handed to the made-up times (only
/// read while `dword_755A0C` is 1, which it is only for `default_checkpoint_times`, which
/// does not read it) and the x `fit_player_names` measures from (the loading picture
/// leaves ecx alone).
pub unsafe fn set_up_front_end(stage: u32, x: u32) -> Result<(), Refused> {
    use crate::saves::{
        allocate_best_times, default_best_scores, default_championship_times,
        default_checkpoint_times, default_class_times, default_stage_times,
        default_stage_times_alt,
    };
    unsafe {
        if crate::install::store::INSTALL.has_clio.get() == 1 {
            crate::menu::add_clio_car::add_clio_car()?;
        }
        crate::menu::link_player_names::link_player_names()?;
        crate::menu::show_loading_picture::show_loading_picture()?;
        crate::gfx::build_menu_sprites::build_menu_sprites_in_game()?;
        crate::menu::fit_player_names::fit_player_names(x)?;
        for (n, (from, to)) in COPIES.into_iter().enumerate() {
            if n == 2 {
                allocate_best_times::allocate_best_times(
                    BEST_TIMES.vars(),
                    BEST_TIMES.list(),
                    crate::records::stages_store::STAGE_TABLES.championship() as *const u8,
                )?;
            }
            // The pairs name their variables by IDA's address (`moved::place`).
            crate::store::ida_write(to, crate::store::ida_read::<u32>(from));
        }
        if BEST_TIMES.loaded.get() != 1 {
            DRAW_WORK.work_0c.set(0u32);
            default_best_scores::default_best_scores()?;
            DRAW_WORK.work_0c.set(0u32);
            default_championship_times::default_championship_times(stage, &mut to_the_log)?;
            DRAW_WORK.work_0c.set(1u32);
            default_checkpoint_times::default_checkpoint_times(stage, &mut to_the_log)?;
            DRAW_WORK.work_0c.set(0u32);
            default_class_times::default_class_times()?;
            DRAW_WORK.work_0c.set(0u32);
            default_stage_times::default_stage_times(stage, &mut to_the_log)?;
            DRAW_WORK.work_0c.set(0u32);
            default_stage_times_alt::default_stage_times_alt(stage, &mut to_the_log)?;
            BEST_TIMES.loaded.set(1);
        }
        if LOOP_FLAGS.options_loaded.get() != 1 {
            for table in record_tables() {
                crate::records::default_stage_records::default_stage_records(table)?;
            }
        }
        Ok(())
    }
}
