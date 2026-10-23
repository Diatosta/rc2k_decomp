//! `input::device_command` (0x408AE5): the input thread's command (al):
//! - 1: device ah given back (`input::release_device`) and acquired again (`input::acquire_device`);
//! - 2 (the race's end): `dword_518090` 0, the effects ended (`input::end_effects`), a
//!   tick (`input::input_tick`), both devices given back and +0x414 0 (not read);
//! - 3 (the race's start): device 0 set up from player 1's controls (`dword_75AE28`,
//!   `input::set_up_device`) and read (+0x414 1); unless one player on one device
//!   (`dword_4F68FC` 0 and `g_onePlayer` 1), device 1 from player 2's (`dword_75AF94`, its
//!   controller first made none when it is player 1's), likewise; a second's sleep, the
//!   effects begun (`input::begin_effects`), a tick, `dword_518090` 1;
//! - others: nothing.
//!
//! In:  al = the command, ah = the device (for 1)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::input_tick` (when `g_inputCommand` was set). Its calls through `part`:
//! 0 the release (esi), 1 the acquiring (esi), 2 the effects' end, 3 the tick, 4 the set-up
//! (esi, edi), 5 the sleep (eax the milliseconds), 6 the effects' start; in the game the
//! Rust versions of all (the acquiring `input::acquire_device`). Refuses (before
//! anything) a device past 1 for command 1.
//! Shadow mode runs the original alone (it acquires devices). Self-test: a copy whose
//! calls go to stand-ins, against the Rust version: each command, each player setting.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::input::devices::put;

pub const DEVICES_RVA: usize = rva(0x4F6EE8); // g_inputDevices: the two records
const EFFECTS_ON_RVA: usize = rva(0x518090); // dword_518090
const CONTROLS_RVA: [usize; 2] = [rva(0x75AE28), rva(0x75AF94)]; // dword_75AE28, dword_75AF94
const TWO_DEVICES_RVA: usize = rva(0x4F68FC); // dword_4F68FC
const ONE_PLAYER_RVA: usize = rva(0x514F78); // g_onePlayer
const CALLEES: [usize; 7] = [
    0x4087F3, 0x408473, 0x40F2B8, 0x408A56, 0x408BCD, 0x41100C, 0x40F29D,
];

fn device(n: usize) -> u32 {
    unsafe { game_read(DEVICES_RVA + 4 * n) }
}

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn device_command(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::input::devices::command_steps`; the registers are carried
    // through the steps as the original has them.
    use rc2k_core::input::devices::{CommandStep::*, command_steps};
    let mut r = *entry;
    unsafe {
        let steps = command_steps(
            r.eax as u8,
            (r.eax >> 8) as u8,
            game_read(TWO_DEVICES_RVA),
            game_read(ONE_PLAYER_RVA),
        )?;
        for step in steps {
            match step {
                EffectsOn(on) => game_write(EFFECTS_ON_RVA, on as u32),
                Release(n) => {
                    r.esi = device(n);
                    crate::input::device_command::game_part(0, &mut r)?;
                }
                Acquire(n) => {
                    r.esi = device(n);
                    crate::input::device_command::game_part(1, &mut r)?;
                }
                EndEffects => crate::input::device_command::game_part(2, &mut r)?,
                Tick => crate::input::device_command::game_part(3, &mut r)?,
                SetUp(n) => {
                    r.esi = device(n);
                    r.edi = game_addr(CONTROLS_RVA[n]) as u32;
                    crate::input::device_command::game_part(4, &mut r)?;
                }
                Reading(n, on) => put(device(n), 0x414, on as u32),
                SecondApart => {
                    r.eax = game_read(CONTROLS_RVA[0]);
                    if r.eax == game_read::<u32>(CONTROLS_RVA[1]) {
                        game_write(CONTROLS_RVA[1], 0u32);
                    }
                }
                Sleep(ms) => {
                    r.eax = ms;
                    crate::input::device_command::game_part(5, &mut r)?;
                }
                BeginEffects => crate::input::device_command::game_part(6, &mut r)?,
            }
        }
    }
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => *r = crate::input::release_device::release_device(r)?,
            3 => *r = crate::input::input_tick::input_tick(r)?,
            4 => *r = crate::input::set_up_device::set_up_device(r)?,
            1 => crate::input::acquire_device::acquire_device(r.esi)?,
            5 => crate::system::sleep::sleep(r.eax),
            2 => crate::input::end_effects::end_effects()?,
            6 => crate::input::begin_effects::begin_effects()?,
            p => crate::calls::call(rva(CALLEES[p as usize]), r),
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { device_command(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
