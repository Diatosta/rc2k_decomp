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
//! One caller, `menu::main_menu_loop`.
//! Refuses what its parts refuse; such a refusal comes with the work half done.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::saves::default_times::{WITH_CHECKPOINTS_RVA, to_the_log};

// ---- game data ----

const HAS_CLIO_RVA: usize = rva(0x529DB4); // g_hasClio
const BEST_TIMES_LOADED_RVA: usize = rva(0x747614); // g_bestTimesLoaded
const OPTIONS_LOADED_RVA: usize = rva(0x75557D); // g_optionsLoaded
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
/// The record tables that get made-up records: the championship's and twelve more.
const RECORD_TABLES: [usize; 13] = [
    0x735576, 0x739642, 0x7391C2, 0x738D42, 0x7388C2, 0x738442, 0x737FC2, 0x739AC2, 0x739F42,
    0x73A3C2, 0x73A842, 0x73ACC2, 0x73B142,
];

// ---- Rust version ----

/// What the function calls, in the order of the test's stand-ins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    AddClioCar,
    LinkPlayerNames,
    LoadingPicture,
    BuildMenuSprites,
    FitPlayerNames,
    AllocateBestTimes,
    DefaultBestScores,
    DefaultChampionshipTimes,
    DefaultCheckpointTimes,
    DefaultClassTimes,
    DefaultStageTimes,
    DefaultStageTimesAlt,
    DefaultStageRecords,
}

/// Does a call with the registers as they are at it (edi is the table for
/// `DefaultStageRecords`); the loading picture may change them.
pub type Outside<'a> = &'a mut dyn FnMut(Call, &mut Regs) -> Result<(), Refused>;

pub unsafe fn set_up_front_end(r: &mut Regs, outside: Outside) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(HAS_CLIO_RVA) == 1 {
            outside(Call::AddClioCar, r)?;
        }
        outside(Call::LinkPlayerNames, r)?;
        outside(Call::LoadingPicture, r)?;
        outside(Call::BuildMenuSprites, r)?;
        outside(Call::FitPlayerNames, r)?;
        for (n, (from, to)) in COPIES.into_iter().enumerate() {
            if n == 2 {
                outside(Call::AllocateBestTimes, r)?;
            }
            game_write(rva(to), game_read::<u32>(rva(from)));
        }
        if game_read::<u32>(BEST_TIMES_LOADED_RVA) != 1 {
            for (call, with_checkpoints) in [
                (Call::DefaultBestScores, 0u32),
                (Call::DefaultChampionshipTimes, 0),
                (Call::DefaultCheckpointTimes, 1),
                (Call::DefaultClassTimes, 0),
                (Call::DefaultStageTimes, 0),
                (Call::DefaultStageTimesAlt, 0),
            ] {
                game_write(WITH_CHECKPOINTS_RVA, with_checkpoints);
                outside(call, r)?;
            }
            game_write(BEST_TIMES_LOADED_RVA, 1u32);
        }
        if game_read::<u32>(OPTIONS_LOADED_RVA) != 1 {
            for table in RECORD_TABLES {
                r.edi = game_addr(rva(table)) as u32;
                outside(Call::DefaultStageRecords, r)?;
            }
        }
        Ok(())
    }
}

/// The calls done by the Rust versions; the loading picture by the game, and the
/// sprites through their hook.
fn in_game(call: Call, r: &mut Regs) -> Result<(), Refused> {
    use crate::saves::{
        allocate_best_times, default_best_scores, default_championship_times,
        default_checkpoint_times, default_class_times, default_stage_times,
        default_stage_times_alt,
    };
    unsafe {
        match call {
            Call::AddClioCar => crate::menu::add_clio_car::add_clio_car()?,
            Call::LinkPlayerNames => crate::menu::link_player_names::link_player_names()?,
            Call::LoadingPicture => crate::calls::call(rva(0x47CEA8), r),
            // Through its own hook, which decides which version of it runs.
            Call::BuildMenuSprites => crate::calls::call(rva(0x47C4A1), &mut { *r }),
            Call::FitPlayerNames => crate::menu::fit_player_names::fit_player_names(r.ecx)?,
            Call::AllocateBestTimes => allocate_best_times::allocate_best_times(
                game_addr(allocate_best_times::VARS_RVA) as *mut u32,
                game_addr(rva(0x7475D0)) as *mut crate::memory::MemoryList,
                game_addr(rva(0x735576)) as *const u8,
            )?,
            Call::DefaultBestScores => default_best_scores::default_best_scores()?,
            Call::DefaultChampionshipTimes => {
                default_championship_times::default_championship_times(r.ebp, &mut to_the_log)?
            }
            Call::DefaultCheckpointTimes => {
                default_checkpoint_times::default_checkpoint_times(r.ebp, &mut to_the_log)?
            }
            Call::DefaultClassTimes => default_class_times::default_class_times()?,
            Call::DefaultStageTimes => {
                default_stage_times::default_stage_times(r.ebp, &mut to_the_log)?
            }
            Call::DefaultStageTimesAlt => {
                default_stage_times_alt::default_stage_times_alt(r.ebp, &mut to_the_log)?
            }
            Call::DefaultStageRecords => {
                crate::records::default_stage_records::default_stage_records(r.edi)?
            }
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut running = *r;
    unsafe { set_up_front_end(&mut running, &mut in_game) }?;
    Ok(())
}
