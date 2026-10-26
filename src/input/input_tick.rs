//! `input::input_tick` (0x408A56): one tick of the input thread's devices: the command
//! `g_inputCommand` taken (`xchg` with 0); when there is one, `input::device_command`;
//! else both devices (`unk_4F6EF0`, `unk_4F73E8`) read (`input::read_device`) and their
//! effects told (`input::device_effects`), and with one player on two devices
//! (`dword_4F68FC`) the second's readings added to the first's (its bits: those of 0x70FF).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the input thread (`sub_4078DA`, every 20 ms), `input::device_command`
//! (twice). Its calls through `part`: 0 the command (eax), 1 the reading (esi), 2 the
//! effects (esi); in the game their Rust versions. Shadow mode runs the original alone (it
//! polls the devices). Self-test: a copy whose calls go to stand-ins, against the Rust
//! version, a command or none, one device or two.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::input::devices::{RECORDS, at, put};
use crate::input::types::InputDevice;

pub const COMMAND_RVA: usize = rva(0x4F7DE4); // g_inputCommand
const TWO_DEVICES_RVA: usize = rva(0x4F68FC); // dword_4F68FC

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn input_tick(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let command = unsafe {
        let at = game_addr(COMMAND_RVA) as *const std::sync::atomic::AtomicU32;
        (*at).swap(0, std::sync::atomic::Ordering::SeqCst)
    };
    r.eax = command;
    if command != 0 {
        part(0, &mut r)?;
        return Ok(*entry);
    }
    let records = RECORDS.map(|rva| game_addr(rva) as u32);
    for record in records {
        r.esi = record;
        part(1, &mut r)?;
        part(2, &mut r)?;
    }
    if unsafe { game_read::<u32>(TWO_DEVICES_RVA) } != 0 {
        // Converted: `rc2k_core::input::devices::merge_readings`.
        let [first, second] = records;
        let mut a = [0x4DC, 0x4E0, 0x4E4, field!(InputDevice, bits)].map(|o| at(first, o));
        let b = [0x4DC, 0x4E0, 0x4E4, field!(InputDevice, bits)].map(|o| at(second, o));
        rc2k_core::input::devices::merge_readings(&mut a, &b);
        for (o, v) in [0x4DC, 0x4E0, 0x4E4, 0x4E8].into_iter().zip(a) {
            put(first, o, v);
        }
    }
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => {
                *r = crate::input::device_command::device_command(
                    r,
                    &mut crate::input::device_command::game_part,
                )?
            }
            1 => {
                *r = crate::input::read_device::read_device(
                    r,
                    &mut crate::input::read_device::game_part,
                )?
            }
            _ => {
                *r = crate::input::device_effects::device_effects(
                    r,
                    &mut crate::input::device_effects::game_part,
                )?
            }
        }
    }
    Ok(())
}

/// The Rust version with the game's calls: for the standalone's input thread.
pub unsafe fn input_tick_in_game() -> Result<(), Refused> {
    unsafe { input_tick(&Regs::default(), &mut game_part) }.map(|_| ())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { input_tick(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
