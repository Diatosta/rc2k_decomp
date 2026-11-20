//! More menu item actions (found by surveying the menu and draw-list tables):
//! - the current player's settings: keep the key copy (the bindings), lower the dead
//!   zones (`end_zone_pct`, `centre_dead_zone_pct`), step the `strength` (by ten, 0 .. 100), turn
//!   the `controller_mode`'s bit 0;
//! - the best scores' page (`pages.scores_page`, 0 .. 10) and the replays' range
//!   (`pages.replay_page`, 0 .. 4) one more or less, wrapping, then the replays' caption;
//! - `menu::player_N_car_specs`: `menu::show_car_specs` for player N (0 .. 3);
//! - `menu::toggle_caption_choice`: the caption choice's bit 0 turned over and its string
//!   (0x4FC + it) into the caption;
//! - `menu::leave_with_carry`: `loop_flags.leave_screen` and `left_with_carry` set.
//!
//! Refuses (before anything) no current player (or one past the four).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use crate::menu::screen_loop::Entry;
use rc2k_core::menu::player::Player;
use rc2k_core::menu::steps;

/// The current player's record.
fn player(menus: &Menus) -> Result<crate::menu::standings::PlayerCell<'_>, Refused> {
    crate::menu::player::player(menus, menus.screens.event.current_player.get())
        .ok_or_else(|| BadInput("no current player (or one past the four)").into())
}

// The logic is in `rc2k_core::menu::player` (the player's fields) and `rc2k_core::menu::steps`
// (`scores_page`, `replay_range`, `flip`).

/// The current player's record changed by `f`.
fn with_player(menus: &Menus, f: impl FnOnce(&mut Player)) -> Result<(), Refused> {
    player(menus)?.update(f);
    Ok(())
}

/// `menu::keep_player_keys`: the keys' table's bindings into the player's.
pub fn keep_player_keys(menus: &Menus) -> Result<(), Refused> {
    let controls = menus.keys.tables.controls.read();
    with_player(menus, |p| {
        p.bindings = rc2k_core::menu::player::ControlsBlock::from_bytes(controls);
    })
}

/// The device's strength ten more.
pub fn raise_strength(menus: &Menus) -> Result<(), Refused> {
    with_player(menus, |p| p.step_strength(true))
}

/// The device's strength ten less.
pub fn lower_strength(menus: &Menus) -> Result<(), Refused> {
    with_player(menus, |p| p.step_strength(false))
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

pub(super) fn previous_replay_range(
    menus: &Menus,
    tables: &Tables,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), BadInput> {
    tables
        .pages
        .replay_page
        .set(steps::replay_range(tables.pages.replay_page.get(), false));
    crate::screens::best_scores::replay_range_caption(&menus.tables.pages, text)
}

pub(super) fn next_replay_range(
    menus: &Menus,
    tables: &Tables,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), BadInput> {
    tables
        .pages
        .replay_page
        .set(steps::replay_range(tables.pages.replay_page.get(), true));
    crate::screens::best_scores::replay_range_caption(&menus.tables.pages, text)
}

/// `menu::toggle_caption_choice`: the choice flipped and its caption written.
pub(super) fn toggle_caption_choice(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    screens: &Screens,
    text_buffers: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), BadInput> {
    let choice = steps::flip(screens.state.menus.caption_choice.get());
    screens.state.menus.caption_choice.set(choice);
    let caption = rc2k_core::screens::page_values::stage_options_caption(choice);
    let mut text = Vec::new();
    crate::strings::append_string_by_id::push_string(so, &mut text, caption)?;
    text.push(0);
    text_buffers.caption.write_at(0, &text, "the caption")
}

pub fn leave_with_carry(menus: &Menus) {
    crate::menu::leaving::with_leaving(menus, rc2k_core::menu::leaving::Leaving::leave_with_carry);
}

/// `menu::player_N_car_specs`: the car specifications screen with player `n` the current
/// one.
pub(super) fn player_car_specs(menus: &Menus, n: u32, entry: &Entry) -> Result<(), Refused> {
    let screens = &menus.screens;
    let kept = screens.event.current_player.get();
    screens.event.current_player.set(n);
    let done = { crate::menu::show_car_specs::show_car_specs(menus, entry) };
    screens.event.current_player.set(kept);
    done
}
