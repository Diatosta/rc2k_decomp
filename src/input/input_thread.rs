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
//! Takes: the thread's parameter (not read)
//! Out: does not return
//!
//! Started by `threading::start_threads` (a slot's routine). In
//! the standalone program DirectInput is made in Rust (`input::di_objects`, on SDL's
//! joysticks, `platform_input.rs`) and released; the keyboard and the mouse are SDL's
//! events (they fill `g_keysDown`): nothing is made, read or released for them. Refuses what its
//! steps refuse.

use crate::errors::Refused;
use crate::input::devices::records;
use crate::input::owner::Input;
use crate::system::owner::system;

const START_TEXT_RVA: &std::ffi::CStr = c"INPUTTDB:START"; // "INPUTTDB:START"
const QUIT_TEXT_RVA: &std::ffi::CStr = c"INPUTTDB:QUIT"; // "INPUTTDB:QUIT"
const TICK_MS: u32 = 20;

/// "INPUTTDB:.." to the game's log (level 2, a line).
fn log(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 2);
}

/// The thread's work up to its `ExitThread` (see the header).
pub fn input_thread(input: &Input, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let system = system();
    use crate::threading::set_priority::set_priority;
    let flag = &crate::threading::thread_slot::slot_ref(crate::threading::thread_slot::INPUT)?.flag;
    'work: {
        if flag.get() == 1 {
            break 'work;
        }
        flag.set(1);
        log(START_TEXT_RVA);
        // DirectInput made (in Rust, on SDL's joysticks); the keyboard and the mouse are
        // SDL's events: nothing to make for them.
        let direct_input = crate::input::di_objects::create_input();
        input.devices.direct_input.set(direct_input);
        if direct_input == 0 {
            break 'work;
        }
        crate::input::list_joysticks::list_joysticks(input);
        set_priority(2);
        let mut last = crate::system::tick_count::tick_count();
        input.devices.last_tick.set(last);
        loop {
            let mut gone;
            loop {
                crate::system::sleep::sleep(5);
                last = input.devices.last_tick.get();
                gone = crate::system::tick_count::tick_count().wrapping_sub(last);
                if gone >= TICK_MS {
                    break;
                }
            }
            loop {
                system
                    .session
                    .clocks
                    .key_ticks
                    .set(system.session.clocks.key_ticks.get().wrapping_add(1));
                gone -= TICK_MS;
                last = last.wrapping_add(TICK_MS);
                if gone < TICK_MS {
                    break;
                }
            }
            input.devices.last_tick.set(last);
            crate::input::input_tick::input_tick_in_game(input, sound)?;
            if system.session.flags.window_focused.get() != 1 {
                crate::system::sleep::sleep(1000);
            }
            if system.session.flags.quitting.get() == 1 {
                break;
            }
        }
    }
    for record in records(&input.devices) {
        { crate::input::release_device::release_device_in_game(&input.devices, record) }?;
    }
    let direct_input = input.devices.direct_input.get();
    if direct_input != 0 {
        crate::input::di_objects::release(direct_input);
        input.devices.direct_input.set(0);
    }
    log(QUIT_TEXT_RVA);
    set_priority(0);
    flag.set(0);
    Ok(())
}

/// The thread's routine as its slot starts it (by address: `routes.rs`): the work; its end
/// ends the thread (the original's `ExitThread`).
pub fn thread_routine(input: &crate::input::owner::Input) -> Result<(), Refused> {
    let sound = crate::sound::owner::sound();
    input_thread(input, sound)
}
