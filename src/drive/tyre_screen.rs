//! `drive::tyre_screen` (0x41C5FA): the tyre-change screen for the mode eax (0: one table;
//! 2: two, for car esi, the wheel's place 0xAA further left; else two more) and car esi:
//! its layout's 0x20 bytes (`TyreChange::layouts`) into `panel` (16) and
//! `parts` (16), the wheel's place (`wheel`: the panel's middle
//! less the wheel's half), the car into `player`; the texture (`device_texture`,
//! place 0) and eleven render states (place 1); the panel and wheel quads (place 2), the
//! wheel turned by the car's input device (place 3), the fixed parts (place 4). All in
//! `pusha` / `popa`.
//!
//! In:  eax the mode, esi the car (0, 1)
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_change_screen`.
//! Refuses a car past 7 (`g_inputDevices`).

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::input::owner::input;
use crate::race::owner::race;
use crate::session::read_all;
use rc2k_core::drive::tyre_screen::{layout, wheel_place};

const STATES: [(u32, u32); 11] = [
    (2, 0x15),
    (0, 0x1C),
    (5, 0x13),
    (6, 0x14),
    (1, 0x1B),
    (3, 3),
    (2, 0x11),
    (2, 0x12),
    (0, 7),
    (0, 0xE),
    (0, 0xF),
];

/// The Rust version: the mode `mode` (eax), the car `car` (esi); every register kept.
// Converted: `rc2k_core::drive::tyre_screen::{layout, wheel_place}`.
pub fn tyre_screen(mode: u32, car: u32) -> Result<(), Refused> {
    let race = race();
    if car > 7 {
        return Err(crate::errors::BadInput("a car past 7 (g_inputDevices)").into());
    }
    let (number, shift) = layout(mode, car);
    let table = 8 * number as usize;
    for k in 0..4 {
        race.shared.tyre_change.panel[k].set(race.shared.tyre_change.layouts[table + k].get());
    }
    for k in 0..4 {
        race.shared.tyre_change.parts[k].set(race.shared.tyre_change.layouts[table + 4 + k].get());
    }
    let panel = std::array::from_fn(|k| race.shared.tyre_change.panel[k].get());
    let [x, y] = wheel_place(
        panel,
        [
            race.shared.tyre_change.wheel[2].get(),
            race.shared.tyre_change.wheel[3].get(),
        ],
        shift,
    );
    race.shared.tyre_change.wheel[0].set(x);
    race.shared.tyre_change.wheel[1].set(y);
    race.shared.tyre_change.player.set(car);
    let Some(device) = input().devices.input_device(car as usize)? else {
        return Err(crate::errors::BadInput("a tyre screen car without a device").into());
    };
    set_texture(race.shared.tyre_change.device_texture.get());
    for (value, state) in STATES {
        set_render_state(value, state);
    }
    crate::drive::tyre_quad::tyre_quad_cells(
        &race.shared.tyre_change.panel_texture,
        &race.shared.tyre_change.panel,
        0xFF_FFFF,
    )?;
    crate::drive::tyre_quad::tyre_quad_cells(
        &race.shared.tyre_change.wheel_texture,
        &race.shared.tyre_change.wheel,
        0xFF_FFFF,
    )?;
    crate::drive::tyre_turned_quad::tyre_turned_quad(
        read_all(&race.shared.tyre_change.turning_texture),
        &race.shared.tyre_change.turning_rect,
        0xFF_FFFF,
        device,
    )?;
    crate::drive::tyre_screen_parts::tyre_screen_parts()
}
