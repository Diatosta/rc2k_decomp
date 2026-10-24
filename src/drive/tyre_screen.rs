//! `drive::tyre_screen` (0x41C5FA): the tyre-change screen for the mode eax (0: one table;
//! 2: two, for car esi, the wheel's place 0xAA further left; else two more) and car esi:
//! its layout's 0x20 bytes (`unk_5BEB30` .. `unk_5BEBB0`) into `dword_5BEAD0` (16) and
//! `dword_5BEBD0` (16), the wheel's place (`dword_5BEAE0`, `5BEAE4`: the panel's middle
//! less the wheel's half), the car into `dword_5BEBEC`; the texture (`dword_5BEBE4`,
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
use crate::physics::flow::{a, g, p};
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
pub unsafe fn tyre_screen(mode: u32, car: u32) -> Result<(), Refused> {
    if car > 7 {
        return Err(crate::errors::BadInput("a car past 7 (g_inputDevices)").into());
    }
    let (number, shift) = layout(mode, car);
    let table = 0x5BEB30 + 0x20 * number as usize;
    unsafe {
        std::ptr::copy(a(table) as *const u8, a(0x5BEAD0) as *mut u8, 16);
        std::ptr::copy((a(table) + 16) as *const u8, a(0x5BEBD0) as *mut u8, 16);
    }
    let panel = std::array::from_fn(|k| g(0x5BEAD0 + 4 * k));
    let [x, y] = wheel_place(panel, [g(0x5BEAE8), g(0x5BEAEC)], shift);
    p(0x5BEAE0, x);
    p(0x5BEAE4, y);
    p(0x5BEBEC, car);
    let device = g(crate::input::store::DeviceSlot::at(0x4F6EE8) + 4 * car as usize);
    set_texture(g(0x5BEBE4));
    for (value, state) in STATES {
        set_render_state(value, state);
    }
    unsafe {
        crate::drive::tyre_quad::tyre_quad(a(0x5BEB00), a(0x5BEAD0), 0xFF_FFFF)?;
        crate::drive::tyre_quad::tyre_quad(a(0x5BEB10), a(0x5BEAE0), 0xFF_FFFF)?;
        crate::drive::tyre_turned_quad::tyre_turned_quad(
            a(0x5BEB20),
            a(0x5BEAF0),
            0xFF_FFFF,
            device,
        )?;
        crate::drive::tyre_screen_parts::tyre_screen_parts()
    }
}
