//! `input::input_thread` (0x4078DA): the input thread's routine ("INPUTTDB", its slot
//! `slot` 0x5184F4). Unless its slot's flag (+0xC) is already 1 (then straight to the end):
//! the flag 1, "INPUTTDB:START" logged; DirectInput made (`sub_4079DC`, `dword_4F6904`; none:
//! to the end), the keyboard, the mouse and the joysticks (`sub_407A1F`, `sub_407FFE`,
//! `sub_408216`), the priority 2 (`threading::set_priority`). Then every 20 ms (waits of
//! 5 ms on `system::tick_count`, kept in `dword_4F68D8`): `dword_518998` one more for each
//! 20 ms gone, the keyboard and the mouse read (`sub_407AC2`, `sub_4080F2`), a tick of the
//! devices (`input::input_tick`), a second's sleep while the window is not the active one
//! (`dword_518378` not 1); until `g_quitting` is 1. The end: both devices given back
//! (`input::release_device`), the mouse, the keyboard and DirectInput released
//! (`sub_40809E`, `sub_407A84`, `sub_4079FE`), "INPUTTDB:QUIT" logged, the priority 0, the
//! flag 0, `ExitThread`.
//!
//! In:  the thread's parameter (not read)
//! Out: does not return
//!
//! Started by `threading::start_threads` (a slot's routine). Its calls through `part`:
//! 0 the log (eax the text), 1 DirectInput made (eax 1 when it was), 2 .. 4 the keyboard,
//! the mouse, the joysticks made, 5 the priority (eax, esi the slot), 6 the keyboard read,
//! 7 the mouse read, 8 .. 10 the mouse, the keyboard, DirectInput released; in the game
//! through the game (DirectInput, section 15), the joysticks `input::list_joysticks`. In
//! the standalone program DirectInput is made in Rust (`input::di_objects`, on SDL's
//! joysticks, `platform_input.rs`) and released; the keyboard and the mouse are SDL's
//! events (they fill `g_keysDown`): nothing is made, read or released for them. Each tick passes a gate (`hook_enter` / `hook_leave`), so a
//! self-test's pause holds the thread between ticks. Refuses what its steps refuse (the
//! original then runs: its flag is set, so it goes to its end).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva, standalone};
use crate::input::devices::RECORDS;

// Kept for the hooked game (2026-10-23): the game's `start_threads` starts the thread at
// its own address. Without this hook the original ran, with no gate, so a self-test's
// pause could not hold it and tests that set `dword_518998` met its count
// (`race::replay_viewer`, shadow on). Removed when the hooked DLL goes.
const SLOT_RVA: usize = rva(0x5184F4); // slot: the input thread's
const START_TEXT_RVA: usize = rva(0x4DA4B2); // "INPUTTDB:START"
const QUIT_TEXT_RVA: usize = rva(0x4DA4C1); // "INPUTTDB:QUIT"
const DIRECT_INPUT_RVA: usize = rva(0x4F6904); // dword_4F6904
const LAST_TICK_RVA: usize = rva(0x4F68D8); // dword_4F68D8
const TICKS_RVA: usize = rva(0x518998); // dword_518998
const ACTIVE_RVA: usize = rva(0x518378); // dword_518378
const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
const TICK_MS: u32 = 20;

/// The calls by place, as IDA shows them (0 the log, 5 the priority).
const CALLEES: [usize; 11] = [
    0x401439, // 0 logfile::log_write_l2_nl
    0x4079DC, // 1 DirectInput made
    0x407A1F, // 2 the keyboard made
    0x407FFE, // 3 the mouse made
    0x408216, // 4 the joysticks made
    0x410D39, // 5 threading::set_priority
    0x407AC2, // 6 the keyboard read
    0x4080F2, // 7 the mouse read
    0x40809E, // 8 the mouse released
    0x407A84, // 9 the keyboard released
    0x4079FE, // 10 DirectInput released
];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// The thread's work up to its `ExitThread` (see the header).
pub unsafe fn input_thread(part: Part) -> Result<(), Refused> {
    let slot = game_addr(SLOT_RVA) as u32;
    let flag = (slot + 0xC) as *mut u8;
    let mut r = Regs {
        esi: slot,
        ..Regs::default()
    };
    'work: {
        if unsafe { *flag } == 1 {
            break 'work;
        }
        unsafe { *flag = 1 };
        r.eax = game_addr(START_TEXT_RVA) as u32;
        part(0, &mut r)?;
        r.eax = 0;
        part(1, &mut r)?;
        if r.eax != 1 {
            break 'work;
        }
        for place in [2, 3, 4] {
            part(place, &mut r)?;
        }
        r.eax = 2;
        part(5, &mut r)?;
        let mut last = crate::system::tick_count::tick_count();
        unsafe { game_write(LAST_TICK_RVA, last) };
        loop {
            let mut gone;
            loop {
                crate::system::sleep::sleep(5);
                last = g(LAST_TICK_RVA);
                gone = crate::system::tick_count::tick_count().wrapping_sub(last);
                if gone >= TICK_MS {
                    break;
                }
            }
            // One gate a tick, before anything is written: a self-test's pause waits for this
            // one and holds the next (2026-10-23: the gate began after the count, so a pause
            // begun during the wait let one count through; `install::run_page`'s test saw it).
            loop {
                unsafe { game_write(TICKS_RVA, g(TICKS_RVA).wrapping_add(1)) };
                gone -= TICK_MS;
                last = last.wrapping_add(TICK_MS);
                if gone < TICK_MS {
                    break;
                }
            }
            unsafe { game_write(LAST_TICK_RVA, last) };
            let ticked = (|| {
                part(6, &mut r)?;
                part(7, &mut r)?;
                unsafe { crate::input::input_tick::input_tick_in_game() }
            })();
            ticked?;
            if g(ACTIVE_RVA) != 1 {
                crate::system::sleep::sleep(1000);
            }
            if g(QUITTING_RVA) == 1 {
                break;
            }
        }
    }
    for record in RECORDS {
        unsafe { crate::input::release_device::release_device_in_game(game_addr(record) as u32) }?;
    }
    for place in [8, 9, 10] {
        part(place, &mut r)?;
    }
    r.eax = game_addr(QUIT_TEXT_RVA) as u32;
    part(0, &mut r)?;
    r.esi = slot;
    r.eax = 0;
    part(5, &mut r)?;
    unsafe { *flag = 0 };
    Ok(())
}

/// In the game: through the game (place 1 then says whether DirectInput was made), but the
/// priority (5), which is `threading::set_priority`'s Rust version, and the joysticks' list
/// (4), `input::list_joysticks`'s. In the standalone program DirectInput is made in Rust
/// (`input::di_objects`, on SDL's joysticks) and given back at the end (10); the keyboard
/// and the mouse are SDL's events: nothing is made, read or given back for them.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 5 {
        // `threading::set_priority`'s Rust version (eax, ecx, edx kept, as there).
        crate::threading::set_priority::set_priority(r.eax as i32);
        return Ok(());
    }
    if place == 4 {
        crate::input::list_joysticks::list_joysticks();
        return Ok(());
    }
    if standalone() {
        match place {
            1 => {
                let input = crate::input::di_objects::create_input();
                unsafe { game_write(DIRECT_INPUT_RVA, input) };
                r.eax = (input != 0) as u32;
            }
            10 => {
                let input = g(DIRECT_INPUT_RVA);
                if input != 0 {
                    crate::input::di_harness::method(
                        input,
                        crate::input::di_harness::slot::RELEASE,
                        &[],
                    );
                    unsafe { game_write(DIRECT_INPUT_RVA, 0u32) };
                }
            }
            _ => {}
        }
        return Ok(());
    }
    crate::calls::call(rva(CALLEES[place as usize]), r);
    if place == 1 {
        r.eax = (g(DIRECT_INPUT_RVA) != 0) as u32;
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { input_thread(&mut game_part) }?;
    unsafe { windows_sys::Win32::System::Threading::ExitThread(0) }
}
