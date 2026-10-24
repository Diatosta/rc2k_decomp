//! More item actions the standalone could reach unported (2026-10-19, the survey of the
//! menu and draw-list tables):
//! - the current player's (`g_players`[`g_currentPlayer`]) settings, every register kept
//!   unless said: `menu::keep_player_keys` (0x46AEFC): the 0x2C bytes at `unk_7555B1` to
//!   +0x50; `menu::lower_player_40` (0x46B386, no `pusha`: edi the player, eax the value):
//!   +0x40 one less while above 0 (signed); `menu::lower_player_44` (0x46B3CB): +0x44 one less
//!   unless 0; `menu::raise_player_48` (0x46B528): +0x48 ten more unless 100;
//!   `menu::lower_player_48` (0x46B54C): +0x48 ten less unless 0; `menu::toggle_player_3c`
//!   (0x4704EC): +0x3C's bit 0 turned over;
//! - `menu::next_scores_page` (0x46D0FB) and `menu::previous_scores_page` (0x46D115): the
//!   best scores' page (`dword_755240`) one more (11 back to 0) or less (below 0 to 10);
//! - `menu::previous_replay_range` (0x46D423) and `menu::next_replay_range` (0x46D43B): the
//!   replays' range (`dword_755248`) one less (below 0 to 4) or more (past 4 to 0), then its
//!   caption (`screens::replay_range_caption`);
//! - `menu::player_N_car_specs` (0x46FCC1, 0x46FCDD, 0x46FCF9, 0x46FD15): `menu::show_car_specs`
//!   for player N (0 .. 3, `g_currentPlayer` set and put back);
//! - `menu::toggle_caption_choice` (0x47056B): `dword_75AB30`'s bit 0 turned over and string
//!   0x4FC + it with a NUL into the caption (`g_captionText`; eax as
//!   `strings::append_string_by_id` leaves it, ebx past the NUL);
//! - `menu::leave_with_carry` (0x471CCE): `dword_755561` and `dword_755565` 1, the carry set;
//!
//! 0x48558A, `pusha; popa; retn` in a draw list, does nothing and is too short for a hook's
//! jump: the standalone counts it among the empty functions it skips.
//!
//! Refuses (before anything) no current player (or one past the four).
//! Hooks removed 2026-10-22 (typed tables): the menus run their `entry` (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::Carried;
use rc2k_core::menu::player::Player;
use rc2k_core::menu::steps;

const CURRENT_PLAYER_RVA: usize = rva(0x75598C); // g_currentPlayer
const KEYS_RVA: usize = rva(0x7555B1); // unk_7555B1: 0x2C bytes
const SCORES_PAGE_RVA: usize = rva(0x755240); // dword_755240
const REPLAY_RANGE_RVA: usize = rva(0x755248); // dword_755248
const CHOICE_RVA: usize = rva(0x75AB30); // dword_75AB30
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}
/// The current player's record.
fn player() -> Result<u32, Refused> {
    match crate::menu::player::player_at(g(CURRENT_PLAYER_RVA)) {
        0 => Err(BadInput("no current player (or one past the four)").into()),
        p => Ok(p),
    }
}

// ---- the Rust versions ----

// Converted: `rc2k_core::menu::player` (the player's fields) and `rc2k_core::menu::steps`
// (`scores_page`, `replay_range`, `flip`).

/// The current player's record changed by `f` (read, changed, written back whole).
fn with_player<R>(f: impl FnOnce(&mut Player) -> R) -> Result<(u32, R), Refused> {
    let at = player()?;
    unsafe {
        let mut p = (at as *const Player).read_unaligned();
        let r = f(&mut p);
        (at as *mut Player).write_unaligned(p);
        Ok((at, r))
    }
}

pub fn keep_player_keys() -> Result<(), Refused> {
    let controls = unsafe { game_read(KEYS_RVA) };
    with_player(|p| p.controls_50 = controls)?;
    Ok(())
}

/// edi the player, eax the value left.
pub fn lower_player_40() -> Result<(u32, u32), Refused> {
    with_player(Player::lower_40)
}

pub fn lower_player_44() -> Result<(), Refused> {
    with_player(Player::lower_44)?;
    Ok(())
}

pub fn raise_player_48() -> Result<(), Refused> {
    with_player(|p| p.step_48(true))?;
    Ok(())
}

pub fn lower_player_48() -> Result<(), Refused> {
    with_player(|p| p.step_48(false))?;
    Ok(())
}

pub fn toggle_player_3c() -> Result<(), Refused> {
    with_player(Player::toggle_3c)?;
    Ok(())
}

pub fn next_scores_page() {
    p(
        SCORES_PAGE_RVA,
        steps::scores_page(g(SCORES_PAGE_RVA), true),
    );
}

pub fn previous_scores_page() {
    p(
        SCORES_PAGE_RVA,
        steps::scores_page(g(SCORES_PAGE_RVA), false),
    );
}

pub fn previous_replay_range() {
    p(
        REPLAY_RANGE_RVA,
        steps::replay_range(g(REPLAY_RANGE_RVA), false),
    );
    unsafe { crate::screens::best_scores::replay_range_caption() };
}

pub fn next_replay_range() {
    p(
        REPLAY_RANGE_RVA,
        steps::replay_range(g(REPLAY_RANGE_RVA), true),
    );
    unsafe { crate::screens::best_scores::replay_range_caption() };
}

/// eax as `append_string_by_id` leaves it, ebx past the NUL.
pub fn toggle_caption_choice() -> (u32, u32) {
    let choice = steps::flip(g(CHOICE_RVA));
    p(CHOICE_RVA, choice);
    let mut cursor = game_addr(CAPTION_RVA) as u32;
    let caption = rc2k_core::screens::page_values::stage_options_caption(choice);
    let eax = unsafe {
        crate::strings::append_string_by_id::append_string_by_id(caption, &mut |at, n| {
            std::ptr::copy(at, cursor as *mut u8, n);
            cursor += n as u32;
        })
    };
    unsafe { (cursor as *mut u8).write(0) };
    (eax, cursor + 1)
}

pub fn leave_with_carry() {
    // Converted: `rc2k_core::menu::leaving::Leaving::leave_with_carry`.
    crate::menu::leaving::with_leaving(|l| l.leave_with_carry());
}

/// `menu::player_N_car_specs`: the car specifications screen with player `n` the current
/// one. `c` the carried values the item gets.
pub unsafe fn player_car_specs(n: u32, c: &Carried) -> Result<(), Refused> {
    let kept = g(CURRENT_PLAYER_RVA);
    p(CURRENT_PLAYER_RVA, n);
    let done = unsafe { crate::menu::show_car_specs::show_car_specs(c) };
    p(CURRENT_PLAYER_RVA, kept);
    done
}

// ---- by number: what the typed tables run ----

/// Each state action's Rust version by its number. What they leave in registers is read
/// by none of their callers (the item call sites have nothing live after the call).
fn state_run(which: u32) -> Result<(), Refused> {
    match which {
        0 => keep_player_keys()?,
        1 => {
            lower_player_40()?;
        }
        2 => lower_player_44()?,
        3 => raise_player_48()?,
        4 => lower_player_48()?,
        5 => toggle_player_3c()?,
        6 => next_scores_page(),
        7 => previous_scores_page(),
        8 => previous_replay_range(),
        9 => next_replay_range(),
        10 => {
            toggle_caption_choice();
        }
        11 => leave_with_carry(),
        _ => {}
    }
    Ok(())
}

macro_rules! state_action {
    ($module:ident, $name:literal, $address:expr, $which:expr) => {
        pub mod $module {
            use super::*;
            /// The Rust version where the typed tables run it (`menu::table_calls`; hook removed
            /// 2026-10-22).
            pub fn run() -> Result<(), crate::errors::Refused> {
                state_run($which)
            }
        }
    };
}

state_action!(keep_player_keys_hook, "menu::keep_player_keys", 0x46AEFC, 0);
state_action!(lower_player_40_hook, "menu::lower_player_40", 0x46B386, 1);
state_action!(lower_player_44_hook, "menu::lower_player_44", 0x46B3CB, 2);
state_action!(raise_player_48_hook, "menu::raise_player_48", 0x46B528, 3);
state_action!(lower_player_48_hook, "menu::lower_player_48", 0x46B54C, 4);
state_action!(toggle_player_3c_hook, "menu::toggle_player_3c", 0x4704EC, 5);
state_action!(next_scores_page_hook, "menu::next_scores_page", 0x46D0FB, 6);
state_action!(
    previous_scores_page_hook,
    "menu::previous_scores_page",
    0x46D115,
    7
);
state_action!(
    previous_replay_range_hook,
    "menu::previous_replay_range",
    0x46D423,
    8
);
state_action!(
    next_replay_range_hook,
    "menu::next_replay_range",
    0x46D43B,
    9
);
state_action!(
    toggle_caption_choice_hook,
    "menu::toggle_caption_choice",
    0x47056B,
    10
);
state_action!(
    leave_with_carry_hook,
    "menu::leave_with_carry",
    0x471CCE,
    11
);

macro_rules! car_specs {
    ($module:ident, $name:literal, $address:expr, $n:expr, $entry:expr) => {
        pub mod $module {
            use super::*;
            /// The Rust version where the typed tables run it (`menu::table_calls`; hook removed
            /// 2026-10-22).
            pub fn run(c: &Carried) -> Result<(), crate::errors::Refused> {
                unsafe { player_car_specs($n, c) }
            }
        }
    };
}

// push g_currentPlayer (FF 35 8C 59 75 00)
car_specs!(
    player_0_car_specs,
    "menu::player_0_car_specs",
    0x46FCC1,
    0,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);
car_specs!(
    player_1_car_specs,
    "menu::player_1_car_specs",
    0x46FCDD,
    1,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);
car_specs!(
    player_2_car_specs,
    "menu::player_2_car_specs",
    0x46FCF9,
    2,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);
car_specs!(
    player_3_car_specs,
    "menu::player_3_car_specs",
    0x46FD15,
    3,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);
