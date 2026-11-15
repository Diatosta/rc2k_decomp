//! Copies player 1's record into the network game's settings (car, driving aids, HUD
//! switches, controls, co-driver, view) and sets the record's `entry` to 1.
//!
//! The record's `damage[12]` has no field to go to: the settings' `cars[0].wear[9]` is written
//! from it and then from `parts_wear[9]`, and only the second stays (kept as is).

use crate::menu::players_store::PlayerSlots;
use crate::race::owner::Race;
use crate::race::state_store::RaceState;
use rc2k_core::menu::player::{Player, TAKES_PART};

/// Player 1's record into the network game's settings, its entry marked as taking part.
pub fn take_player_settings(race: &Race, slots: &PlayerSlots) {
    let player = &slots.records[0];
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
