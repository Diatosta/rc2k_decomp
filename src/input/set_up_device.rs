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
//! `input::acquire_device`'s Rust version). Shadow mode runs the original alone (it acquires the device).
//! Self-test: a copy whose acquiring goes to a stand-in (which says the device acquired or
//! not), against the Rust version, on records and controls of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::Regs;
use crate::input::devices::{at, put};
use crate::input::types::InputDevice;

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn set_up_device(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (record, controls) = (entry.esi, entry.edi);
    unsafe {
        crate::system::zero_memory::zero_memory((record + 0x414) as *mut u8, 0xC8)?;
    }
    let n = at(controls, 0);
    if n != 0 {
        put(record, field!(InputDevice, controller), n - 1);
        for (to, from) in [
            (0x418, 0x10),
            (0xC, 0x1C),
            (0x10, 0x18),
            (0x14, 0x14),
            (0x18, 0xC),
        ] {
            put(record, to, at(controls, from));
        }
        // Converted: `rc2k_core::input::devices::device_kind`.
        let kind = at(controls, 8);
        put(record, field!(InputDevice, sensitivity), kind);
        let kind = rc2k_core::input::devices::device_kind(kind);
        put(record, field!(InputDevice, kind), kind);
        let mut r = Regs {
            eax: kind,
            ..*entry
        };
        part(0, &mut r)?;
    }
    // Converted: `rc2k_core::input::devices::binding` over `DEVICE_BINDINGS`.
    for (key, word, to) in rc2k_core::input::devices::DEVICE_BINDINGS {
        let key = unsafe { ((controls + key as u32) as *const u8).read() };
        let b = rc2k_core::input::devices::binding(
            key,
            at(controls, word as u32),
            at(record, 0x24) == 1,
        );
        for (n, v) in b.into_iter().enumerate() {
            put(record, to as u32 + 4 * n as u32, v);
        }
    }
    Ok(*entry)
}

/// In the game: the acquiring through the game.
/// In the game: `input::acquire_device`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::input::acquire_device::acquire_device(r.esi, &mut crate::input::make_effect::game_part)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { set_up_device(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
