//! `net::take_player_settings` (0x48755D): player 1's record (`g_players[0]`) copied into
//! the network game's settings (the player's car to its last setting), field to field: the car,
//! the driving aids and the HUD's switches, the controls, the co-driver, the view; and the
//! record's +0xD0 (`entry`) made 1.
//!
//! The original copies 91 dwords, 12 bytes from the record's +0x1C and 0x20 bytes from
//! +0x24C by offset. It writes the settings' 0xA4 (`cars[0].wear[9]`) twice, first from the
//! record's +0x104 and then from its +0x1A0 (`parts_wear[9]`); only the second stays, so the
//! record's +0x104 (`damage[12]`) has no field to go to.
//!
//! Called by `net::start_link`.

use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::race::state_store::RaceState;
use rc2k_core::menu::player::{Player, TAKES_PART};

/// Player 1's record into the network game's settings, its entry marked as taking part.
pub fn take_player_settings(race: &Race, menus: &Menus) {
    let player = &menus.play.players.players[0];
    player.update(|p| p.entry = TAKES_PART);
    copy_player_settings(&player.get(), &race.shared.state);
}

/// The record's fields into the state's settings and results.
fn copy_player_settings(p: &Player, state: &RaceState) {
    let s = &state.settings;
    s.car_kind_current.set(p.car_number);
    let car = &s.cars[0];
    car.tyre.set(p.tyre);
    car.brake_balance.set(p.brake_balance);
    car.ride_height.set(p.ride_height);
    car.suspension_stiffness.set(p.suspension_stiffness);
    car.steering_sensitivity.set(p.steering_sensitivity);
    car.gear_ratio.set(p.gear_ratio);
    // The record's fourteen damage words: the last but one goes to nothing (see above).
    for n in 0..12 {
        car.damage[n].set(p.damage[n]);
    }
    car.damage[12].set(p.damage[13]);
    for (n, w) in p.parts_wear.iter().enumerate() {
        car.wear[n].set(*w);
    }
    for (n, w) in p.tyre_wear.iter().enumerate() {
        car.wear[p.parts_wear.len() + n].set(*w);
    }
    car.map_area.set(p.map_area);
    car.rear_mirror.set(p.rear_mirror);
    car.rev_counter.set(p.rev_counter);
    car.metric_units.set(p.metric_units);
    car.message_display.set(p.message_display);
    car.car_info.set(p.car_info);
    car.progress_bar.set(p.progress_bar);
    car.driver_info.set(p.driver_info);
    car.pace_notes.set(p.pace_notes);
    car.race_number.set(p.race_number);
    car.codriver_sex.set(p.codriver_sex);
    car.view.set(p.view);
    car.controls.controller.set(p.controller);
    car.controls.gearing.set(p.gearing);
    car.controls.sensitivity_level.set(p.sensitivity_level);
    car.controls
        .axis_codes
        .set(u32::from_le_bytes(p.axis_codes));
    car.controls.controller_mode.set(p.controller_mode);
    car.controls.end_zone_pct.set(p.end_zone_pct);
    car.controls
        .centre_dead_zone_pct
        .set(p.centre_dead_zone_pct);
    car.controls.strength.set(p.strength);
    for (cell, word) in car.controls.recorded_bindings().zip(p.bindings.words()) {
        cell.set(word);
    }
    for (n, b) in p.keys.iter().enumerate() {
        car.controls.keys.set(n, *b);
    }
    let r = &state.results;
    r.service_spent_current.set(p.service_spent);
    r.hud_bits_current.set(p.hud_bits as u32);
    r.flags_current.set(p.state_flags);
    for (n, w) in p.impact_marks.iter().enumerate() {
        r.results[0].impact_marks[n].set(*w);
    }
}
