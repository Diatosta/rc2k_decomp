//! `input::device_effects` (0x4096B4): a device's force-feedback effects told their
//! changes: with its effects on (+0x28 1), a DirectInput device (+4) and its controller's
//! entry (`dword_4F78E4` + 0x50 its number, +0x1C) 1, each of its six effects (from +0x3C,
//! 0xA4 bytes each) that is on (+0x30) and has a change waiting (+0x2C, taken and 0): its
//! function (+0) called with eax the change's low byte, ecx bits 8 .. 19 and edx bits 20 ..
//! 31 (each signed), esi the record and edi the effect.
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::input_tick` (each device, each tick). The effects' functions through
//! `part` (eax .. edi as above; in the game the function's address through the game:
//! DirectInput's, section 15). Refuses (before anything) a controller number past the
//! table's 16. Self-test: a copy whose calls go to stand-ins can not be made (the calls
//! are through the effects' pointers): the original and the Rust version on records of
//! the test's whose effects' functions are a noting stand-in of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::input::devices::{at, put};
use crate::input::store::DeviceSlot;
use crate::input::types::{ForceEffect, InputDevice};
use crate::store::game_read;

pub const CONTROLLERS_RVA: DeviceSlot = DeviceSlot::at(0x4F78E4); // dword_4F78E4: 16 entries of 0x50
pub const CONTROLLERS: u32 = 16;
const EFFECTS: u32 = 0x3C;
const EFFECT_LEN: u32 = 0xA4;

/// The controller entry's dword at `offset` for the record's controller, or the refusal.
pub fn controller(record: u32, offset: usize) -> Result<u32, Refused> {
    let n = at(record, field!(InputDevice, controller));
    if n >= CONTROLLERS {
        return Err(BadInput("a controller number past the table's 16").into());
    }
    Ok(unsafe { game_read(CONTROLLERS_RVA + 0x50 * n as usize + offset) })
}

/// The pending changes of the device `record`'s effects.
pub unsafe fn device_effects(record: u32) -> Result<(), Refused> {
    if at(record, field!(InputDevice, effects_on)) != 1
        || at(record, field!(InputDevice, device)) == 0
    {
        return Ok(());
    }
    if controller(record, 0x1C)? != 1 {
        return Ok(());
    }
    for k in 0..6 {
        let effect = record + EFFECTS + EFFECT_LEN * k;
        if at(effect, field!(ForceEffect, object)) == 0 {
            continue;
        }
        let change = at(effect, field!(ForceEffect, pending));
        put(effect, field!(ForceEffect, pending), 0);
        if change == 0 {
            continue;
        }
        // Converted: `rc2k_core::input::effects::change`.
        let (what, a, b) = rc2k_core::input::effects::change(change);
        let function = at(effect, field!(ForceEffect, change));
        crate::input::effect_changes::effect_change_at(
            function, record, effect, what, a as u32, b as u32,
        )?;
    }
    Ok(())
}
