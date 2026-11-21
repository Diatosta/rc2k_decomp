//! The tyre-change screen for a mode and a car: its
//! layout's 0x20 bytes (`TyreChange::layouts`) into `panel` (16) and `parts` (16), the wheel's
//! place (`wheel`: the panel's middle less the wheel's half), the car into `player`; the texture
//! (`device_texture`, place 0) and eleven render states (place 1); the panel and wheel quads (place
//! 2), the wheel turned by the car's input device (place 3), the fixed parts (place 4).
//!
//! Refuses a car past 7.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::race::owner::Race;
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
pub fn tyre_screen(
    race: &Race,
    mode: u32,
    car: u32,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = &race.shared;
    if car > 7 {
        return Err(crate::errors::BadInput("a car past 7 (g_inputDevices)").into());
    }
    let (number, shift) = layout(mode, car);
    let table = 8 * number as usize;
    {
        let mut tyre = shared.tyre_change_mut();
        for k in 0..4 {
            tyre.panel[k] = tyre.layouts[table + k];
        }
        for k in 0..4 {
            tyre.parts[k] = tyre.layouts[table + 4 + k];
        }
        let panel = tyre.panel;
        let [x, y] = wheel_place(panel, [tyre.wheel[2], tyre.wheel[3]], shift);
        tyre.wheel[0] = x;
        tyre.wheel[1] = y;
        tyre.player = car;
    }
    let Some(device) = input.devices.input_device(car as usize)? else {
        return Err(crate::errors::BadInput("a tyre screen car without a device").into());
    };
    // What the quads are drawn from; none of it changes while they are.
    let (device_texture, panel, panel_texture, wheel, wheel_texture, turning_texture, turning_rect) = {
        let tyre = shared.tyre_change();
        (
            tyre.device_texture,
            tyre.panel,
            tyre.panel_texture,
            tyre.wheel,
            tyre.wheel_texture,
            tyre.turning_texture,
            tyre.turning_rect,
        )
    };
    let part = scene.scratch.game_scratch();
    set_texture(part, device_texture);
    for (value, state) in STATES {
        set_render_state(part, value, state);
    }
    crate::drive::tyre_quad::quad_of(
        panel_texture,
        panel,
        0xFF_FFFF,
        race.viewer_panel(),
        system,
        part,
        // The dark backdrop fills the whole screen.
        Spread::Fill,
    )?;
    crate::drive::tyre_quad::quad_of(
        wheel_texture,
        wheel,
        0xFF_FFFF,
        race.viewer_panel(),
        system,
        part,
        Spread::Centred,
    )?;
    crate::drive::tyre_turned_quad::tyre_turned_quad(
        shared,
        turning_texture,
        turning_rect,
        0xFF_FFFF,
        device,
        system,
        race.viewer_panel(),
        part,
    )?;
    crate::drive::tyre_screen_parts::tyre_screen_parts(race, scene, system)
}
