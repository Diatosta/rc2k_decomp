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
//! Started by `threading::start_threads` (a slot's routine). In
//! the standalone program DirectInput is made in Rust (`input::di_objects`, on SDL's
//! joysticks, `platform_input.rs`) and released; the keyboard and the mouse are SDL's
//! events (they fill `g_keysDown`): nothing is made, read or released for them. Refuses what its
//! steps refuse.

use crate::errors::Refused;
use crate::input::devices::record_addrs;
use crate::input::store::DEVICES;
use crate::pointers::GamePtr;
use crate::session::CLOCKS;
use crate::session::FLAGS;

const START_TEXT_RVA: &std::ffi::CStr = c"INPUTTDB:START"; // "INPUTTDB:START"
const QUIT_TEXT_RVA: &std::ffi::CStr = c"INPUTTDB:QUIT"; // "INPUTTDB:QUIT"
const TICK_MS: u32 = 20;

/// "INPUTTDB:.." to the game's log (level 2, a line).
fn log(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 2);
}

/// The thread's work up to its `ExitThread` (see the header).
pub unsafe fn input_thread() -> Result<(), Refused> {
    use crate::threading::set_priority::set_priority;
    let slot = (crate::threading::thread_slot::slot(crate::threading::thread_slot::INPUT)) as u32;
    let flag = GamePtr::new(slot + 0xC);
    'work: {
        if flag.read::<u8>("the input thread's slot")? == 1 {
            break 'work;
        }
        flag.write(1u8, "the input thread's slot")?;
        log(START_TEXT_RVA);
        // DirectInput made (in Rust, on SDL's joysticks); the keyboard and the mouse are
        // SDL's events: nothing to make for them.
        let input = crate::input::di_objects::create_input();
        DEVICES.direct_input.set(input);
        if input == 0 {
            break 'work;
        }
        crate::input::list_joysticks::list_joysticks();
        set_priority(2);
        let mut last = crate::system::tick_count::tick_count();
        DEVICES.last_tick.set(last);
        loop {
            let mut gone;
            loop {
                crate::system::sleep::sleep(5);
                last = DEVICES.last_tick.get();
                gone = crate::system::tick_count::tick_count().wrapping_sub(last);
                if gone >= TICK_MS {
                    break;
                }
            }
            loop {
                CLOCKS.key_ticks.set(CLOCKS.key_ticks.get().wrapping_add(1));
                gone -= TICK_MS;
                last = last.wrapping_add(TICK_MS);
                if gone < TICK_MS {
                    break;
                }
            }
            DEVICES.last_tick.set(last);
            unsafe { crate::input::input_tick::input_tick_in_game() }?;
            if FLAGS.window_focused.get() != 1 {
                crate::system::sleep::sleep(1000);
            }
            if FLAGS.quitting.get() == 1 {
                break;
            }
        }
    }
    for record in record_addrs() {
        { crate::input::release_device::release_device_in_game(record) }?;
    }
    let input = DEVICES.direct_input.get();
    if input != 0 {
        crate::input::di_objects::release(input);
        DEVICES.direct_input.set(0);
    }
    log(QUIT_TEXT_RVA);
    set_priority(0);
    flag.write(0u8, "the input thread's slot")?;
    Ok(())
}

/// The thread's routine as its slot starts it (by address: `routes.rs`): the work; its end
/// ends the thread (the original's `ExitThread`).
pub fn thread_routine() -> Result<(), Refused> {
    unsafe { input_thread() }
}
