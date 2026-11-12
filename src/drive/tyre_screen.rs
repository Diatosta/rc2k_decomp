//! `drive::tyre_screen` (0x41C5FA): the tyre-change screen for a mode and a car: its
//! layout's 0x20 bytes (`TyreChange::layouts`) into `panel` (16) and `parts` (16), the wheel's
//! place (`wheel`: the panel's middle less the wheel's half), the car into `player`; the texture
//! (`device_texture`, place 0) and eleven render states (place 1); the panel and wheel quads (place
//! 2), the wheel turned by the car's input device (place 3), the fixed parts (place 4).
//!
//! Caller: `drive::tyre_change_screen`.
//! Refuses a car past 7 (`g_inputDevices`).

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::race::owner::Race;
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

/// The mode `mode`, the car `car`.
// Converted: `rc2k_core::drive::tyre_screen::{layout, wheel_place}`.
pub fn tyre_screen(
    race: &Race,
    mode: u32,
    car: u32,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let shared = &race.shared;
    if car > 7 {
        return Err(crate::errors::BadInput("a car past 7 (g_inputDevices)").into());
    }
    let (number, shift) = layout(mode, car);
    let table = 8 * number as usize;
    for k in 0..4 {
        shared.tyre_change.panel[k].set(shared.tyre_change.layouts[table + k].get());
    }
    for k in 0..4 {
        shared.tyre_change.parts[k].set(shared.tyre_change.layouts[table + 4 + k].get());
    }
    let panel = std::array::from_fn(|k| shared.tyre_change.panel[k].get());
    let [x, y] = wheel_place(
        panel,
        [
            shared.tyre_change.wheel[2].get(),
            shared.tyre_change.wheel[3].get(),
        ],
        shift,
    );
    shared.tyre_change.wheel[0].set(x);
    shared.tyre_change.wheel[1].set(y);
    shared.tyre_change.player.set(car);
    let Some(device) = input.devices.input_device(car as usize)? else {
        return Err(crate::errors::BadInput("a tyre screen car without a device").into());
    };
    set_texture(shared.tyre_change.device_texture.get());
    for (value, state) in STATES {
        set_render_state(value, state);
    }
    crate::drive::tyre_quad::tyre_quad_cells(
        &shared.tyre_change.panel_texture,
        &shared.tyre_change.panel,
        0xFF_FFFF,
        race.viewer_panel(),
        system,
    )?;
    crate::drive::tyre_quad::tyre_quad_cells(
        &shared.tyre_change.wheel_texture,
        &shared.tyre_change.wheel,
        0xFF_FFFF,
        race.viewer_panel(),
        system,
    )?;
    crate::drive::tyre_turned_quad::tyre_turned_quad(
        shared,
        read_all(&shared.tyre_change.turning_texture),
        &shared.tyre_change.turning_rect,
        0xFF_FFFF,
        device,
        system,
        race.viewer_panel(),
    )?;
    crate::drive::tyre_screen_parts::tyre_screen_parts(race, system)
}
