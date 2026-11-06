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
//! Caller: `input::input_tick` (each device, each tick). Refuses (before anything) a controller number past the
//! table's 16.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::input::types::{Controller, InputDevice};

pub const CONTROLLERS: u32 = 16;

/// The record's controller entry, or the refusal.
pub fn controller(record: &InputDevice) -> Result<&'static Controller, Refused> {
    let n = record.controller.get();
    if n >= CONTROLLERS {
        return Err(BadInput("a controller number past the table's 16").into());
    }
    Ok(&crate::input::owner::input().devices.controllers[n as usize])
}

/// The pending changes of the device `record`'s effects.
pub fn device_effects(record: &InputDevice) -> Result<(), Refused> {
    if record.effects_on.get() != 1 || record.device.get() == 0 {
        return Ok(());
    }
    if controller(record)?.force_feedback.get() != 1 {
        return Ok(());
    }
    for effect in &record.effects {
        if effect.object.get() == 0 {
            continue;
        }
        let change = effect.pending.get();
        effect.pending.set(0);
        if change == 0 {
            continue;
        }
        // Converted: `rc2k_core::input::effects::change`.
        let (what, a, b) = rc2k_core::input::effects::change(change);
        let function = effect.change.get();
        crate::input::effect_changes::effect_change_at(
            function, record, effect, what, a as u32, b as u32,
        )?;
    }
    Ok(())
}
