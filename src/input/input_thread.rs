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
//! `routes.rs`). Plain since 2026-10-23 (phase 3): its steps called directly; the two log
//! lines, which the standalone had dropped, written again.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::input::devices::RECORDS;

// Kept for the hooked game (2026-10-23): the game's `start_threads` starts the thread at
// its own address. Without this hook the original ran, with no gate, so a self-test's
// pause could not hold it and tests that set `dword_518998` met its count
// (`race::replay_viewer`, shadow on). Removed when the hooked DLL goes.
const START_TEXT_RVA: usize = rva(0x4DA4B2); // "INPUTTDB:START"
const QUIT_TEXT_RVA: usize = rva(0x4DA4C1); // "INPUTTDB:QUIT"
const DIRECT_INPUT_RVA: usize = rva(0x4F6904); // dword_4F6904
const LAST_TICK_RVA: usize = rva(0x4F68D8); // dword_4F68D8
const TICKS_RVA: usize = rva(0x518998); // dword_518998
const ACTIVE_RVA: usize = rva(0x518378); // dword_518378
const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
const TICK_MS: u32 = 20;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// "INPUTTDB:.." to the game's log (level 2, a line).
fn log(text: usize) {
    unsafe { crate::logfile::log_write::log_write(game_addr(text) as *const u8, true, 2) };
}

/// The thread's work up to its `ExitThread` (see the header).
pub unsafe fn input_thread() -> Result<(), Refused> {
    use crate::threading::set_priority::set_priority;
    let slot = crate::threading::thread_slot::slot(crate::threading::thread_slot::INPUT) as u32;
    let flag = (slot + 0xC) as *mut u8;
    'work: {
        if unsafe { *flag } == 1 {
            break 'work;
        }
        unsafe { *flag = 1 };
        log(START_TEXT_RVA);
        // DirectInput made (in Rust, on SDL's joysticks); the keyboard and the mouse are
        // SDL's events: nothing to make for them.
        let input = crate::input::di_objects::create_input();
        unsafe { game_write(DIRECT_INPUT_RVA, input) };
        if input == 0 {
            break 'work;
        }
        crate::input::list_joysticks::list_joysticks();
        set_priority(2);
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
            loop {
                unsafe { game_write(TICKS_RVA, g(TICKS_RVA).wrapping_add(1)) };
                gone -= TICK_MS;
                last = last.wrapping_add(TICK_MS);
                if gone < TICK_MS {
                    break;
                }
            }
            unsafe { game_write(LAST_TICK_RVA, last) };
            unsafe { crate::input::input_tick::input_tick_in_game() }?;
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
    let input = g(DIRECT_INPUT_RVA);
    if input != 0 {
        crate::input::di_harness::method(input, crate::input::di_harness::slot::RELEASE, &[]);
        unsafe { game_write(DIRECT_INPUT_RVA, 0u32) };
    }
    log(QUIT_TEXT_RVA);
    set_priority(0);
    unsafe { *flag = 0 };
    Ok(())
}

/// The thread's routine as its slot starts it (by address: `routes.rs`): the work, then
/// `ExitThread`.
pub fn thread_routine() -> Result<(), Refused> {
    unsafe { input_thread() }?;
    unsafe { windows_sys::Win32::System::Threading::ExitThread(0) }
}
