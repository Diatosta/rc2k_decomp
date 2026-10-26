//! `input::set_up_device` (0x408BCD): a device's record made from a player's controls
//! (edi: `dword_75AE28` player 1, `dword_75AF94` player 2): +0x414 .. +0x4DB 0
//! (`system::zero_memory`, 0xC8 bytes); with a controller chosen ([edi] n, not 0): +0 n - 1,
//! +0x418 [edi+0x10], +0xC [edi+0x1C], +0x10 [edi+0x18], +0x14 [edi+0x14], +0x18 [edi+0xC],
//! +0x20 [edi+8] and +8 the same or 2 for 0, and the controller acquired (`input::acquire_device`);
//! then the twelve bindings: each its key (a byte of [edi+0x54 ..]) and the place, sign
//! and button of its word (`input::binding_place` of a dword of [edi+0x24 ..]), in the order
//! of `BINDINGS`.
//!
//! In:  esi = the device's record, edi = the player's controls
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::device_command` (the race's start, both players). The acquiring through
//! `part` (0, esi and edi as here, eax [edi+8] or 2; in the game
//! `input::acquire_device`'s Rust version).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::field;
use crate::input::devices::{at, put};
use crate::input::types::InputDevice;
use crate::pointers::GamePtr;

/// The device `record` set up from its controls at `controls` (see the header).
pub unsafe fn set_up_device(record: u32, controls: u32) -> Result<(), Refused> {
    GamePtr::new(record)
        .offset(0x414)
        .fill(0, 0xC8, "a device record's bindings")?;
    let n = at(controls, 0)?;
    if n != 0 {
        put(record, field!(InputDevice, controller), n - 1)?;
        for (to, from) in [
            (0x418, 0x10),
            (0xC, 0x1C),
            (0x10, 0x18),
            (0x14, 0x14),
            (0x18, 0xC),
        ] {
            put(record, to, at(controls, from)?)?;
        }
        // Converted: `rc2k_core::input::devices::device_kind`.
        let kind = at(controls, 8)?;
        put(record, field!(InputDevice, sensitivity), kind)?;
        let kind = rc2k_core::input::devices::device_kind(kind);
        put(record, field!(InputDevice, kind), kind)?;
        crate::input::acquire_device::acquire_device(record)?;
    }
    // Converted: `rc2k_core::input::devices::binding` over `DEVICE_BINDINGS`.
    for (key, word, to) in rc2k_core::input::devices::DEVICE_BINDINGS {
        let key: u8 = GamePtr::new(controls)
            .offset(key as u32)
            .read("a device's control key")?;
        let b = rc2k_core::input::devices::binding(
            key,
            at(controls, word as u32)?,
            at(record, 0x24)? == 1,
        );
        for (n, v) in b.into_iter().enumerate() {
            put(record, to as u32 + 4 * n as u32, v)?;
        }
    }
    Ok(())
}
