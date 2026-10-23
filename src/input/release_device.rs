//! `input::release_device` (0x4087F3): a device given back: with a DirectInput device (+4),
//! `Unacquire`, its effects stopped (`sub_4095AE`) when its controller's entry
//! (`dword_4F78E4` + 0x50 its number, +0x1C) is 1, `Release`, +4 0; then +0x24 (acquired)
//! and +0x28 (effects on) 0 and its readings cleared (`input::clear_device_readings`).
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `input::device_command` (three times), the input thread's end. Its calls
//! through `part`: 0 `Unacquire` (eax the device), 1 the effects stopped (esi), 2
//! `Release` (eax); in the game the device's methods and `input::end_force`'s Rust
//! version. Refuses (before anything) a controller number past 16 with
//! a device. Shadow mode runs the original alone (it releases the device). Self-test: a
//! copy whose effects' call goes to a stand-in, against the Rust version, on records of
//! the test's with a device of the test's or none, the controller's entry 1 or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::{Regs, game_addr};
use crate::input::clear_device_readings::clear_device_readings;
use crate::input::device_effects::{CONTROLLERS_RVA, controller};
use crate::input::devices::{at, com_call, put};
use crate::input::types::InputDevice;

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn release_device(entry: &Regs) -> Result<Regs, Refused> {
    let record = entry.esi;
    let device = at(record, field!(InputDevice, device));
    if device != 0 {
        let effects = controller(record, 0x1C)? == 1;
        let mut r = Regs {
            eax: device,
            ..*entry
        };
        crate::input::release_device::game_part(0, &mut r)?;
        if effects {
            let mut e = Regs {
                edi: game_addr(CONTROLLERS_RVA) as u32
                    + crate::input::di_harness::CONTROLLER_LEN
                        * at(record, field!(InputDevice, controller)),
                ..r
            };
            crate::input::release_device::game_part(1, &mut e)?;
        }
        r.eax = device;
        crate::input::release_device::game_part(2, &mut r)?;
        put(record, field!(InputDevice, device), 0);
    }
    put(record, field!(InputDevice, acquired), 0);
    put(record, field!(InputDevice, effects_on), 0);
    clear_device_readings(record);
    Ok(*entry)
}

/// In the game: the device's methods, `input::end_force`.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => r.eax = com_call(r.eax, 0x20, &[]),
            1 => crate::input::end_force::end_force(r.esi),
            _ => r.eax = com_call(r.eax, 0x08, &[]),
        }
    }
    Ok(())
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn release_device_in_game(record: u32) -> Result<(), Refused> {
    let r = Regs {
        esi: record,
        ..Regs::default()
    };
    unsafe { release_device(&r) }.map(|_| ())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { release_device(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
