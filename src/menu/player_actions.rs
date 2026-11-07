//! More item actions the standalone could reach unported (2026-10-19, the survey of the
//! menu and draw-list tables):
//! - the current player's (`g_players`[`g_currentPlayer`]) settings, every register kept
//!   unless said: `menu::keep_player_keys` (0x46AEFC): the 0x2C bytes at `unk_7555B1` to
//!   +0x50; `menu::lower_player_40` (0x46B386, `lower_dead_zone_1`; no `pusha`, but no
//!   caller reads edi or eax after it): the first dead zone (+0x40) one less while above 0
//!   (signed); `menu::lower_player_44` (0x46B3CB, `lower_dead_zone_2`): the second one less
//!   unless 0; `menu::raise_player_48` (0x46B528) / `lower_player_48` (0x46B54C): the
//!   strength (+0x48) ten more unless 100 / ten less unless 0; `menu::toggle_player_3c`
//!   (0x4704EC, `toggle_controller_mode`): the mode's (+0x3C) bit 0 turned over;
//! - `menu::next_scores_page` (0x46D0FB) and `menu::previous_scores_page` (0x46D115): the
//!   best scores' page (`dword_755240`) one more (11 back to 0) or less (below 0 to 10);
//! - `menu::previous_replay_range` (0x46D423) and `menu::next_replay_range` (0x46D43B): the
//!   replays' range (`dword_755248`) one less (below 0 to 4) or more (past 4 to 0), then its
//!   caption (`screens::replay_range_caption`);
//! - `menu::player_N_car_specs` (0x46FCC1, 0x46FCDD, 0x46FCF9, 0x46FD15): `menu::show_car_specs`
//!   for player N (0 .. 3, `g_currentPlayer` set and put back);
//! - `menu::toggle_caption_choice` (0x47056B): `dword_75AB30`'s bit 0 turned over and string
//!   0x4FC + it with a NUL into the caption (`g_captionText`; the eax and ebx it leaves are
//!   read by none of its callers);
//! - `menu::leave_with_carry` (0x471CCE): `dword_755561` and `dword_755565` 1, the carry set;
//!
//! Refuses (before anything) no current player (or one past the four).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use rc2k_core::menu::player::Player;
use rc2k_core::menu::steps;

/// The current player's record.
fn player(screens: &Screens) -> Result<crate::menu::standings::PlayerCell, Refused> {
    crate::menu::player::player(screens.event.current_player.get())
        .ok_or_else(|| BadInput("no current player (or one past the four)").into())
}

// ---- the Rust versions ----

// Converted: `rc2k_core::menu::player` (the player's fields) and `rc2k_core::menu::steps`
// (`scores_page`, `replay_range`, `flip`).

/// The current player's record changed by `f`.
fn with_player(screens: &Screens, f: impl FnOnce(&mut Player)) -> Result<(), Refused> {
    player(screens)?.update(f);
    Ok(())
}

/// `menu::keep_player_keys`: the keys' table's bindings into the player's.
pub fn keep_player_keys(menus: &Menus) -> Result<(), Refused> {
    let controls = menus.keys.tables.controls.read();
    with_player(&menus.screens, |p| p.bindings = controls)
}

/// `menu::lower_player_40`.
pub fn lower_dead_zone_1(screens: &Screens) -> Result<(), Refused> {
    with_player(screens, Player::lower_dead_zone_1)
}

/// `menu::lower_player_44`.
pub fn lower_dead_zone_2(screens: &Screens) -> Result<(), Refused> {
    with_player(screens, Player::lower_dead_zone_2)
}

/// `menu::raise_player_48`.
pub fn raise_strength(screens: &Screens) -> Result<(), Refused> {
    with_player(screens, |p| p.step_strength(true))
}

/// `menu::lower_player_48`.
pub fn lower_strength(screens: &Screens) -> Result<(), Refused> {
    with_player(screens, |p| p.step_strength(false))
}

/// `menu::toggle_player_3c`.
pub fn toggle_controller_mode(screens: &Screens) -> Result<(), Refused> {
    with_player(screens, Player::toggle_controller_mode)
}

pub fn next_scores_page(tables: &Tables) {
    tables
        .pages
        .scores_page
        .set(steps::scores_page(tables.pages.scores_page.get(), true));
}

pub fn previous_scores_page(tables: &Tables) {
    tables
        .pages
        .scores_page
        .set(steps::scores_page(tables.pages.scores_page.get(), false));
}

pub fn previous_replay_range(menus: &Menus, tables: &Tables) -> Result<(), BadInput> {
    tables
        .pages
        .replay_page
        .set(steps::replay_range(tables.pages.replay_page.get(), false));
    crate::screens::best_scores::replay_range_caption(menus)
}

pub fn next_replay_range(menus: &Menus, tables: &Tables) -> Result<(), BadInput> {
    tables
        .pages
        .replay_page
        .set(steps::replay_range(tables.pages.replay_page.get(), true));
    crate::screens::best_scores::replay_range_caption(menus)
}

/// `menu::toggle_caption_choice`: the choice flipped and its caption written.
pub fn toggle_caption_choice(screens: &Screens) -> Result<(), BadInput> {
    let choice = steps::flip(screens.state.menus.caption_choice.get());
    screens.state.menus.caption_choice.set(choice);
    let caption = rc2k_core::screens::page_values::stage_options_caption(choice);
    let mut text = Vec::new();
    crate::strings::append_string_by_id::push_string(&mut text, caption)?;
    text.push(0);
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .caption
        .write_at(0, &text, "the caption")
}

pub fn leave_with_carry() {
    // Converted: `rc2k_core::menu::leaving::Leaving::leave_with_carry`.
    crate::menu::leaving::with_leaving(rc2k_core::menu::leaving::Leaving::leave_with_carry);
}

/// `menu::player_N_car_specs`: the car specifications screen with player `n` the current
/// one.
pub fn player_car_specs(menus: &'static Menus, n: u32) -> Result<(), Refused> {
    let screens = &menus.screens;
    let kept = screens.event.current_player.get();
    screens.event.current_player.set(n);
    let done = { crate::menu::show_car_specs::show_car_specs(menus) };
    screens.event.current_player.set(kept);
    done
}
