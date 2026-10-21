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

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::input::devices::put;
use crate::race::mud::{copy, run_both_from, test_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::device_command", rva(0x408AE5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const DEVICES_RVA: usize = rva(0x4F6EE8); // g_inputDevices: the two records
const EFFECTS_ON_RVA: usize = rva(0x518090); // dword_518090
const CONTROLS_RVA: [usize; 2] = [rva(0x75AE28), rva(0x75AF94)]; // dword_75AE28, dword_75AF94
const TWO_DEVICES_RVA: usize = rva(0x4F68FC); // dword_4F68FC
const ONE_PLAYER_RVA: usize = rva(0x514F78); // g_onePlayer
const CALLEES: [usize; 7] = [
    0x4087F3, 0x408473, 0x40F2B8, 0x408A56, 0x408BCD, 0x41100C, 0x40F29D,
];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn device(n: usize) -> u32 {
    unsafe { game_read(DEVICES_RVA + 4 * n) }
}

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn device_command(entry: &Regs, part: Part) -> Result<Regs, Refused> {
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
                    part(0, &mut r)?;
                }
                Acquire(n) => {
                    r.esi = device(n);
                    part(1, &mut r)?;
                }
                EndEffects => part(2, &mut r)?,
                Tick => part(3, &mut r)?,
                SetUp(n) => {
                    r.esi = device(n);
                    r.edi = game_addr(CONTROLS_RVA[n]) as u32;
                    part(4, &mut r)?;
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
                    part(5, &mut r)?;
                }
                BeginEffects => part(6, &mut r)?,
            }
        }
    }
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => {
                *r = crate::input::release_device::release_device(
                    r,
                    &mut crate::input::release_device::game_part,
                )?
            }
            3 => {
                *r = crate::input::input_tick::input_tick(
                    r,
                    &mut crate::input::input_tick::game_part,
                )?
            }
            4 => {
                *r = crate::input::set_up_device::set_up_device(
                    r,
                    &mut crate::input::set_up_device::game_part,
                )?
            }
            1 => crate::input::acquire_device::acquire_device(
                r.esi,
                &mut crate::input::make_effect::game_part,
            )?,
            5 => crate::system::sleep::sleep(r.eax),
            2 => crate::input::end_effects::end_effects(r, &mut crate::input::effects::game_part)?,
            6 => crate::input::begin_effects::begin_effects(
                r,
                &mut crate::input::effects::game_part,
            )?,
            p => call_game(r, game_addr(rva(CALLEES[p as usize]))),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { device_command(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = Box::leak(
        CALLEES
            .iter()
            .enumerate()
            .map(|(p, &a)| (a, p as u32))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // pusha; cmp al, 1; jnz short
    let copy = match copy(
        0x408AE5,
        0xE8,
        &[0x60, 0x3C, 0x01, 0x75, 0x19],
        &[
            0xF, 0x14, 0x2C, 0x31, 0x3C, 0x51, 0x75, 0xB9, 0xCD, 0xD2, 0xD7,
        ],
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    // Two records of the test's in `g_inputDevices`.
    let mut records = [vec![0u8; 0x4F8], vec![0u8; 0x4F8]];
    let (p0, p1) = (
        records[0].as_mut_ptr() as u32,
        records[1].as_mut_ptr() as u32,
    );
    let mut regions: Vec<Region> = vec![
        (game_addr(DEVICES_RVA) as u32, 8),
        at(EFFECTS_ON_RVA),
        at(CONTROLS_RVA[0]),
        at(CONTROLS_RVA[1]),
        at(TWO_DEVICES_RVA),
        at(ONE_PLAYER_RVA),
    ];
    let saved = capture(&regions);
    regions.extend([(p0, 0x4F8), (p1, 0x4F8)]);
    for (command, two, one, same) in [
        (0x001u32, 0u32, 1u32, false),
        (0x101, 0, 1, false),
        (0x002, 0, 1, false),
        (0x003, 0, 1, false),
        (0x003, 0, 2, false),
        (0x003, 1, 1, true),
        (0x003, 0, 0, true),
        (0x004, 0, 1, false),
        (0xFF03, 0, 2, false),
    ] {
        run_both_from(
            &mut t,
            &format!(
                "command {command:#x}, two devices {two}, one player {one}, same controls {same}"
            ),
            &copy,
            &regions,
            &|| unsafe {
                restore(&regions[..6], &saved);
                game_write(DEVICES_RVA, [p0, p1]);
                for p in [p0, p1] {
                    std::ptr::write_bytes(p as *mut u8, 0xAA, 0x4F8);
                }
                game_write(TWO_DEVICES_RVA, two);
                game_write(ONE_PLAYER_RVA, one);
                game_write(CONTROLS_RVA[0], 2u32);
                game_write(CONTROLS_RVA[1], if same { 2u32 } else { 3 });
            },
            Regs {
                eax: command,
                ..Regs::SENTINEL
            },
            &|r| unsafe { device_command(r, &mut test_part) },
        );
    }
    restore(&regions[..6], &saved);
    drop(records);
    t.finish()
}
