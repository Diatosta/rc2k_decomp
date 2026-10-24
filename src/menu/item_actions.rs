//! Item actions the standalone met unported (2026-10-19, the owner's run without RAL.EXE):
//! - `menu::show_intro` (0x47058E): the options' "intro" item: the films played
//!   (`menu::play_films`, eax 1), then back (`menu::go_back`);
//! - `menu::previous_save_page` (0x46D3F2): the championship load page's left arrow: the
//!   page (`dword_75523C`) one less, from 0 round to 9;
//! - `service::open_steering_box`, `service::open_brake_box` (0x46B706, 0x46B71A): the
//!   service area's steering sensitivity and brake bias items: the box switch
//!   (`dword_75AA9C`) turned over and its page (`dword_75AAA0`) 3, or 4;
//! - `menu::view_stage_details` (0x47037E): "view stage info": for a time trial
//!   (`g_eventType` 3) `menu::time_trial_stage_details_loop`, else `menu::view_stage_loop`;
//! - `menu::new_simulation_if_unlocked` (0x4A96BF): with `dword_75ACCC` 1
//!   `menu::new_simulation_loop`, else the refusal's sound (`sound::play_menu_sound` 8) and a
//!   quarter of a second's sleep;
//! - `menu::start_arcade` (0x487CAA): the arcade's level and stage (`g_arcadeLevel`,
//!   `g_arcadeStage`) 0, the first player's units taken (`menu::take_player1_units`) and the
//!   arcade event run (`menu::arcade_event`).
//!
//! In:  nothing (eax, the item's dword, is not read but by `show_intro`, which sets it)
//! Out: the pure ones and those in `pusha` / `popa` every register kept; the others the
//!      registers their last call leaves
//!
//! The calls through `part` (by place in each one's `CALLEES`): in the game the screen loops,
//! the films and the menu sound through their hooks (`call_game`), `go_back`, `sleep`,
//! `take_player1_units` and `arcade_event` by their Rust versions.
//! Hooks removed 2026-10-22 (typed tables): the menus run their `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Carried, run_screen};

const SAVE_PAGE_RVA: usize = rva(0x75523C); // dword_75523C: the save list's page
const BOX_SWITCH_RVA: usize = rva(0x75AA9C); // dword_75AA9C: a service box shown
const BOX_PAGE_RVA: usize = rva(0x75AAA0); // dword_75AAA0: its page
const EVENT_TYPE_RVA: usize = rva(0x755954); // g_eventType
const UNLOCKED_RVA: usize = rva(0x75ACCC); // dword_75ACCC: 1 lets a new simulation start
const ARCADE_LEVEL_RVA: usize = rva(0x7587EC); // g_arcadeLevel
const ARCADE_STAGE_RVA: usize = rva(0x7587F0); // g_arcadeStage

fn g(at: usize) -> u32 {
    unsafe { game_read(at) }
}

fn p(at: usize, v: u32) {
    unsafe { game_write(at, v) }
}

// ---- the Rust versions ----

/// `menu::show_intro`: the films (idle mode), then back.
pub fn show_intro() -> Result<(), Refused> {
    crate::menu::play_films::play_films(true)?;
    crate::menu::go_back::go_back();
    Ok(())
}

pub fn previous_save_page() {
    // Converted: `rc2k_core::menu::steps::savegame_page`.
    p(
        SAVE_PAGE_RVA,
        rc2k_core::menu::steps::savegame_page(g(SAVE_PAGE_RVA), false),
    );
}

/// The steering (page 3) or brake (page 4) box.
pub fn open_service_box(page: u32) {
    p(BOX_SWITCH_RVA, g(BOX_SWITCH_RVA) ^ 1);
    p(BOX_PAGE_RVA, page);
}

/// `menu::view_stage_details`: the stage information screen. `c` the carried values the
/// item gets.
pub unsafe fn view_stage_details(c: &Carried) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::stage_screens::time_trial_details`.
    let time_trial = rc2k_core::menu::stage_screens::time_trial_details(g(EVENT_TYPE_RVA));
    unsafe {
        if time_trial {
            run_screen(&crate::menu::time_trial_stage_details_loop::SCREEN, c)?;
        } else {
            run_screen(&crate::menu::view_stage_loop::SCREEN, c)?;
        }
    }
    Ok(())
}

/// `menu::new_simulation_if_unlocked` (it reads no register).
pub fn new_simulation_if_unlocked() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::stage_screens::simulation_unlocked`.
    use rc2k_core::menu::stage_screens::{REFUSED_SLEEP, REFUSED_SOUND, simulation_unlocked};
    if simulation_unlocked(g(UNLOCKED_RVA)) {
        // (An item of the typed tables: it runs with no carried values, as the menus'
        // items do.)
        crate::menu::new_simulation_loop::run()?;
    } else {
        crate::sound::play_menu_sound::play_menu_sound_in_game(REFUSED_SOUND);
        crate::system::sleep::sleep(REFUSED_SLEEP);
    }
    Ok(())
}

/// `menu::start_arcade`: the arcade's level and stage 0, the first player's units taken
/// and the arcade event run. `c` the carried values the item gets.
pub unsafe fn start_arcade(c: &Carried) -> Result<(), Refused> {
    p(ARCADE_LEVEL_RVA, 0);
    p(ARCADE_STAGE_RVA, 0);
    crate::menu::take_player1_units::take_player1_units()?;
    unsafe { crate::menu::arcade_event::arcade_event(c) }
}
