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

use crate::errors::Refused;
use crate::field;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::input::devices::{RECORDS, at, put};
use crate::input::types::InputDevice;
use crate::race::mud::{copy, run_both, test_part};
use crate::service::harness::{Region, at as at_rva, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::input_tick", rva(0x408A56), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
        let [first, second] = records;
        for offset in [0x4DC, 0x4E0, 0x4E4] {
            put(
                first,
                offset,
                at(first, offset).wrapping_add(at(second, offset)),
            );
        }
        put(
            first,
            0x4E8,
            (at(second, field!(InputDevice, bits)) & 0x70FF) | at(first, field!(InputDevice, bits)),
        );
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { input_tick(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov eax, 0
    let copy = match copy(
        0x408A56,
        0x8F,
        &[0x60, 0xB8, 0x00, 0x00, 0x00, 0x00],
        &[0x17, 0x1C, 0x27, 0x2C, 0x88],
        &[(0x408E38, 1), (0x4096B4, 2), (0x408AE5, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let readings = |rva: usize| (game_addr(rva) as u32 + 0x4DC, 0x10);
    let regions: Vec<Region> = vec![
        at_rva(COMMAND_RVA),
        at_rva(TWO_DEVICES_RVA),
        readings(RECORDS[0]),
        readings(RECORDS[1]),
    ];
    let saved = capture(&regions);
    for (command, two) in [(0u32, 0u32), (0, 1), (3, 0), (2, 1), (0xFFFF_FFFF, 0)] {
        run_both(
            &mut t,
            &format!("command {command:#x}, two devices {two}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(COMMAND_RVA, command);
                game_write(TWO_DEVICES_RVA, two);
                for (k, rva) in RECORDS.into_iter().enumerate() {
                    for (j, offset) in [0x4DCusize, 0x4E0, 0x4E4, 0x4E8].into_iter().enumerate() {
                        game_write(
                            rva + offset,
                            (0x1234_0000 * (k as u32 + 1)) ^ (0x9_7531 * (j as u32 + 1)),
                        );
                    }
                }
            },
            &|r| unsafe { input_tick(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
