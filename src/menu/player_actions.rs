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
//! Refuses (before anything) no current player (or one past the four).

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::keys_store::KEY_TABLES;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_loop::Carried;
use crate::menu::state_store::MENU_STATE;
use crate::pointers::GamePtr;
use rc2k_core::menu::player::Player;
use rc2k_core::menu::steps;

/// The current player's record.
fn player() -> Result<crate::menu::standings::PlayerCell, Refused> {
    crate::menu::player::player(EVENT_STATE.current_player.get())
        .ok_or_else(|| BadInput("no current player (or one past the four)").into())
}

// ---- the Rust versions ----

// Converted: `rc2k_core::menu::player` (the player's fields) and `rc2k_core::menu::steps`
// (`scores_page`, `replay_range`, `flip`).

/// The current player's record changed by `f` (read, changed, written back whole).
fn with_player<R>(f: impl FnOnce(&mut Player) -> R) -> Result<(u32, R), Refused> {
    let at = player()?;
    let mut p = at.get();
    let r = f(&mut p);
    at.set(p);
    Ok((at.addr(), r))
}

pub fn keep_player_keys() -> Result<(), Refused> {
    let controls = KEY_TABLES.controls.read();
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
    MENU_PAGES
        .scores_page
        .set(steps::scores_page(MENU_PAGES.scores_page.get(), true));
}

pub fn previous_scores_page() {
    MENU_PAGES
        .scores_page
        .set(steps::scores_page(MENU_PAGES.scores_page.get(), false));
}

pub fn previous_replay_range() -> Result<(), BadInput> {
    MENU_PAGES
        .replay_page
        .set(steps::replay_range(MENU_PAGES.replay_page.get(), false));
    crate::screens::best_scores::replay_range_caption()
}

pub fn next_replay_range() -> Result<(), BadInput> {
    MENU_PAGES
        .replay_page
        .set(steps::replay_range(MENU_PAGES.replay_page.get(), true));
    crate::screens::best_scores::replay_range_caption()
}

/// eax as `append_string_by_id` leaves it, ebx past the NUL.
pub fn toggle_caption_choice() -> Result<(u32, u32), crate::errors::BadInput> {
    let choice = steps::flip(MENU_STATE.menus.caption_choice.get());
    MENU_STATE.menus.caption_choice.set(choice);
    let mut cursor = crate::screens::text_buffers_store::TEXT_BUFFERS
        .caption
        .addr();
    let caption = rc2k_core::screens::page_values::stage_options_caption(choice);
    let fault = crate::menu::event_flow::Fault::default();
    let eax = crate::strings::append_string_by_id::append_string_by_id(caption, &mut |text| {
        fault.ok(GamePtr::new(cursor).write_bytes(text, "the caption"));
        cursor += text.len() as u32;
    })?;
    fault.check()?;
    GamePtr::new(cursor).write::<u8>(0, "the caption")?;
    Ok((eax, cursor + 1))
}

pub fn leave_with_carry() {
    // Converted: `rc2k_core::menu::leaving::Leaving::leave_with_carry`.
    crate::menu::leaving::with_leaving(rc2k_core::menu::leaving::Leaving::leave_with_carry);
}

/// `menu::player_N_car_specs`: the car specifications screen with player `n` the current
/// one. `c` the carried values the item gets.
pub fn player_car_specs(n: u32, c: &Carried) -> Result<(), Refused> {
    let kept = EVENT_STATE.current_player.get();
    EVENT_STATE.current_player.set(n);
    let done = { crate::menu::show_car_specs::show_car_specs(c) };
    EVENT_STATE.current_player.set(kept);
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
        8 => previous_replay_range()?,
        9 => next_replay_range()?,
        10 => {
            toggle_caption_choice()?;
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
            /// The Rust version where the typed tables run it (`menu::table_calls`).
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
            /// The Rust version where the typed tables run it (`menu::table_calls`).
            pub fn run(c: &Carried) -> Result<(), crate::errors::Refused> {
                { player_car_specs($n, c) }
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
