//! `input::update_effects` (0x40F2C0): the race's force feedback brought up to date, once
//! each 40 ms (`input::step_effects_clock`): with sound objects in use
//! (`g_raceSoundObjectCount`), `input::update_car_effects` for player A's car on the first
//! device (`g_inputDevices[0]`, the first view block) if that device has effects; then, if
//! the two devices are one player's (`dword_4F68FC` not 0), for car A again on the second
//! device. Otherwise (no effects on the first device, or one device a player), unless one
//! player (`g_onePlayer` 1), for player B's car on the second device with the second view
//! block. Each only if that device has effects (+0x28 1).
//!
//! In:  nothing
//! Out: eax, ebx, esi, edi as last loaded (the car's number, the view block, the device,
//!      the car's sound object), or as `update_car_effects` leaves them after a call;
//!      flags not read (the caller's `test eax, eax` is inside its `pusha`)
//!
//! See `input/effects.rs`. One caller, `input::step_effects_clock`. Its calls through
//! `part` (`update_car_effects`). Refuses (before anything: the way through is worked out
//! first, nothing it calls changes what decides it) a car number past the eight sound
//! objects and a null device on the way taken. Shadow mode runs the original alone (requests are sent).
//! Self-test: a copy whose fifteen references to the variables go to the test's and whose
//! calls go to the noting stand-in, against the Rust version: every way through.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::input::devices::at;
use crate::input::effects::HAS_EFFECTS;

/// g_playerCarA, g_playerCarB, g_onePlayer, g_raceSoundObjectCount (four dwords).
const CARS: usize = 0x514F70;
/// g_inputDevices: the two records' addresses.
const DEVICES: usize = 0x4F6EE8;
/// g_viewBlocks: the two views' blocks.
const VIEWS: usize = 0x63DBB0;
/// g_raceSoundObjects: eight sound objects.
const OBJECTS: usize = 0x512A10;
/// dword_4F68FC: 1 when both devices are player A's.
const TWO_DEVICES: usize = 0x4F68FC;

/// Where the variables are (the game's or a test's).
#[derive(Clone, Copy)]
pub struct Vars {
    pub cars: u32,
    pub devices: u32,
    pub views: u32,
    pub objects: u32,
    pub two_devices: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            cars: a(CARS),
            devices: a(DEVICES),
            views: a(VIEWS),
            objects: a(OBJECTS),
            two_devices: a(TWO_DEVICES),
        }
    }
}

fn rd(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

/// The Rust version on the variables in `v`.
/// Converted: `rc2k_core::input::effects::loads` (the way worked out before any call:
/// nothing a call changes decides it).
pub fn update_effects_with(v: &Vars) -> Result<(), Refused> {
    use rc2k_core::input::effects::{Players, loads};
    if rd(v.cars + 0xC) == 0 {
        return Ok(());
    }
    let p = Players {
        cars: [rd(v.cars), rd(v.cars + 4)],
        one_player: rd(v.cars + 8) == 1,
        devices: [rd(v.devices), rd(v.devices + 4)],
        views: [rd(v.views), rd(v.views + 4)],
        one_players_devices: rd(v.two_devices) != 0,
    };
    for load in loads(&p, |device| at(device, HAS_EFFECTS) == 1)? {
        if load.update {
            let object = rd(v.objects + 4 * load.car);
            crate::input::update_car_effects::update_car_effects(load.view, object, load.device)?;
        }
    }
    Ok(())
}

/// The race's force feedback brought up to date.
pub fn update_effects() -> Result<(), Refused> {
    update_effects_with(&Vars::game())
}
