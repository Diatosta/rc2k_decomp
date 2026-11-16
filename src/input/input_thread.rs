//! The input thread's routine ("INPUTTDB"). Unless its slot's flag is already 1: the flag set,
//! "INPUTTDB:START" logged, DirectInput made (none: to the end), the keyboard, the mouse and the
//! joysticks set up, the priority raised. Then every 20 ms (waits of 5 ms on the tick count, kept
//! in `last_tick`): `clocks.key_ticks` one more for each 20 ms gone, the keyboard and the mouse
//! read, a tick of the devices, a wait of up to a second while the window is not the active one
//! (ended by its becoming active); until
//! the game is quitting. The end: both devices given back, the mouse, the keyboard and DirectInput
//! released, "INPUTTDB:QUIT" logged, the priority back, the flag 0.
//!
//! Started by `threading::start_threads`. DirectInput is Rust's here (`input::di_objects`, on
//! SDL's joysticks, `platform::input`); the keyboard and the mouse are SDL's events (they fill
//! the key table), so nothing is made, read or released for them. Refuses what its steps refuse.

use crate::errors::Refused;
use crate::input::devices::records;
use crate::input::owner::Input;
use crate::system::owner::System;

const START_TEXT: &std::ffi::CStr = c"INPUTTDB:START";
const QUIT_TEXT: &std::ffi::CStr = c"INPUTTDB:QUIT";
const TICK_MS: u32 = 20;

/// "INPUTTDB:.." to the game's log (level 2, a line).
fn log(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 2);
}

/// How long a tick waits for the window to be the active one again, and how often it looks.
const UNFOCUSED_WAIT_MS: u32 = 1000;
const UNFOCUSED_LOOK_MS: u32 = 10;

/// While the window is not the active one the devices are read only once a second; the wait
/// ends as soon as the window is active again or the game is quitting, so the controllers
/// answer at once after coming back to the window.
fn wait_while_unfocused(system: &System) {
    let flags = &system.session.flags;
    for _ in 0..UNFOCUSED_WAIT_MS / UNFOCUSED_LOOK_MS {
        if flags.window_focused.get() == 1 || flags.quitting.is_set() {
            return;
        }
        crate::system::sleep::sleep(UNFOCUSED_LOOK_MS);
    }
}

/// The thread's work up to its `ExitThread` (see the header).
pub fn input_thread(
    input: &Input,
    sound: &crate::sound::store::SoundShared,
    system: &System,
    race: &crate::race::owner::Race,
) -> Result<(), Refused> {
    use crate::threading::set_priority::set_priority;
    let flag = &crate::threading::thread_slot::slot_ref(crate::threading::thread_slot::INPUT)?.flag;
    'work: {
        if flag.get() == 1 {
            break 'work;
        }
        flag.set(1);
        log(START_TEXT);
        // DirectInput made (in Rust, on SDL's joysticks); the keyboard and the mouse are
        // SDL's events: nothing to make for them.
        let direct_input = crate::input::di_objects::create_input();
        input.devices.direct_input.set(direct_input);
        if direct_input == 0 {
            break 'work;
        }
        crate::input::list_joysticks::list_joysticks(input, system);
        set_priority(2);
        let mut last = crate::system::tick_count::tick_count(&system.session.picture_clock);
        input.devices.last_tick.set(last);
        loop {
            let mut gone;
            loop {
                crate::system::sleep::sleep(5);
                last = input.devices.last_tick.get();
                gone = crate::system::tick_count::tick_count(&system.session.picture_clock)
                    .wrapping_sub(last);
                if gone >= TICK_MS {
                    break;
                }
            }
            loop {
                system
                    .session
                    .key_clock
                    .ticks
                    .set(system.session.key_clock.ticks.get().wrapping_add(1));
                gone -= TICK_MS;
                last = last.wrapping_add(TICK_MS);
                if gone < TICK_MS {
                    break;
                }
            }
            input.devices.last_tick.set(last);
            crate::input::input_tick::input_tick(input, sound, system, race)?;
            wait_while_unfocused(system);
            if system.session.flags.quitting.is_set() {
                break;
            }
        }
    }
    for record in records(&input.devices) {
        crate::input::release_device::release_device(&input.devices, record)?;
    }
    let direct_input = input.devices.direct_input.get();
    if direct_input != 0 {
        crate::input::di_objects::release(direct_input);
        input.devices.direct_input.set(0);
    }
    log(QUIT_TEXT);
    set_priority(0);
    flag.set(0);
    Ok(())
}
