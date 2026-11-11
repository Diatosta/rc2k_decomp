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
    car.controller.set(p.controller);
    car.gearing.set(p.gearing);
    car.sensitivity_level.set(p.sensitivity_level);
    car.axis_codes.set(u32::from_le_bytes(p.axis_codes));
    car.controller_mode.set(p.controller_mode);
    car.end_zone_pct.set(p.end_zone_pct);
    car.centre_dead_zone_pct.set(p.centre_dead_zone_pct);
    car.strength.set(p.strength);
    for (cell, word) in car.recorded_bindings().zip(p.bindings.words()) {
        cell.set(word);
    }
    for (n, b) in p.keys.iter().enumerate() {
        car.keys.set(n, *b);
    }
    let r = &state.results;
    r.service_spent_current.set(p.service_spent);
    r.hud_bits_current.set(p.hud_bits as u32);
    r.flags_current.set(p.state_flags);
    for (n, w) in p.impact_marks.iter().enumerate() {
        r.results[0].impact_marks[n].set(*w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointers::GamePtr;
    use crate::race::state_store::RaceSettings;
    use std::mem::{offset_of, size_of};

    /// The original's dword copies by offset: (the settings' offset from
    /// `car_kind_current`, the record's), in its order. The settings' 0xA4 is written twice.
    #[rustfmt::skip]
    const DWORDS: [(u32, u32); 91] = [
        (0x0, 0x8),
        (0x34, 0x10C),
        (0x38, 0x110),
        (0x3C, 0x114),
        (0x40, 0x118),
        (0x44, 0x11C),
        (0x48, 0x120),
        (0x31C, 0x288),
        (0x32C, 0x278),
        (0x4C, 0xD4),
        (0x50, 0xD8),
        (0x54, 0xDC),
        (0x58, 0xE0),
        (0x5C, 0xE4),
        (0x60, 0xE8),
        (0x64, 0xEC),
        (0x68, 0xF0),
        (0x6C, 0xF4),
        (0x70, 0xF8),
        (0x74, 0xFC),
        (0x78, 0x100),
        (0xA4, 0x104),
        (0x7C, 0x108),
        (0x110, 0x158),
        (0x114, 0x15C),
        (0x118, 0x160),
        (0x120, 0x168),
        (0x124, 0x164),
        (0x12C, 0x170),
        (0x128, 0x16C),
        (0x130, 0x174),
        (0x11C, 0x178),
        (0x324, 0x284),
        (0x80, 0x17C),
        (0x84, 0x180),
        (0x88, 0x184),
        (0x8C, 0x188),
        (0x90, 0x18C),
        (0x94, 0x190),
        (0x98, 0x194),
        (0x9C, 0x198),
        (0xA0, 0x19C),
        (0xA4, 0x1A0),
        (0xA8, 0x1A4),
        (0xAC, 0x1A8),
        (0xB0, 0x1AC),
        (0xB4, 0x1B0),
        (0xB8, 0x1B4),
        (0xBC, 0x1B8),
        (0xC0, 0x1BC),
        (0xC4, 0x1C0),
        (0xC8, 0x1C4),
        (0xCC, 0x1C8),
        (0xD0, 0x1CC),
        (0xD4, 0x1D0),
        (0xD8, 0x1D4),
        (0xDC, 0x1D8),
        (0xE0, 0x1DC),
        (0xE4, 0x1E0),
        (0xE8, 0x1E4),
        (0xEC, 0x1E8),
        (0xF0, 0x1EC),
        (0xF4, 0x1F0),
        (0xF8, 0x1F4),
        (0xFC, 0x1F8),
        (0x100, 0x1FC),
        (0x104, 0x200),
        (0x108, 0x204),
        (0x10C, 0x208),
        (0x134, 0x28),
        (0x138, 0x154),
        (0x13C, 0xC),
        (0x140, 0x2C),
        (0x144, 0x30),
        (0x148, 0x34),
        (0x14C, 0x38),
        (0x150, 0x3C),
        (0x154, 0x40),
        (0x158, 0x44),
        (0x15C, 0x48),
        (0x164, 0x50),
        (0x168, 0x54),
        (0x16C, 0x58),
        (0x170, 0x5C),
        (0x174, 0x60),
        (0x178, 0x64),
        (0x17C, 0x68),
        (0x180, 0x6C),
        (0x184, 0x70),
        (0x18C, 0x74),
        (0x190, 0x78),
    ];

    /// The 0x20 bytes from the record's 0x24C go to the settings' 0x384, the twelve bytes
    /// from the record's 0x1C to 0x194.
    const BYTES: [(usize, usize, usize); 2] = [(0x384, 0x24C, 0x20), (0x194, 0x1C, 12)];

    #[test]
    fn field_copy_matches_the_offset_copy() {
        let shared = &crate::race::owner::race().shared;
        let _g = crate::store::test_lock();
        // A record whose every dword is different.
        let mut bytes = [0u8; 0x28C];
        for (n, chunk) in bytes.chunks_mut(4).enumerate() {
            chunk.copy_from_slice(&(0x0101_0101 * (n as u32 + 1)).to_le_bytes());
        }
        let record = Player::from_le_bytes(bytes);
        let whole = GamePtr::to(&shared.state);
        let before = whole
            .read_bytes(size_of::<RaceState>(), "the race state")
            .unwrap();
        let first = offset_of!(RaceState, settings) + offset_of!(RaceSettings, car_kind_current);
        // The offset copy, on the bytes as they were.
        let mut expected = before.clone();
        for &(to, from) in &DWORDS {
            let at = first + to as usize;
            expected[at..at + 4].copy_from_slice(&bytes[from as usize..from as usize + 4]);
        }
        for &(to, from, len) in &BYTES {
            let at = first + to;
            expected[at..at + len].copy_from_slice(&bytes[from..from + len]);
        }
        copy_player_settings(&record, &shared.state);
        let after = whole
            .read_bytes(size_of::<RaceState>(), "the race state")
            .unwrap();
        whole.write_bytes(&before, "the race state").unwrap();
        assert_eq!(after, expected);
    }
}
