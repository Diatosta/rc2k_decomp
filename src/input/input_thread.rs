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
//! through the game (DirectInput, section 15). In the standalone program SDL stands for
//! DirectInput: nothing is made, read (its events fill `g_keysDown`) or released there, and
//! DirectInput counts as made. Each tick passes a gate (`hook_enter` / `hook_leave`), so a
//! self-test's pause holds the thread between ticks. Ungated thunk (it runs for the
//! game's whole life). Refuses what its steps refuse (the original then runs: its flag is
//! set, so it goes to its end). Shadow mode runs the original alone (the thread is the
//! game's).
//! Self-test: none (it runs until the game ends); its steps carry theirs.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
    standalone,
};
use crate::input::devices::RECORDS;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::input_thread", rva(0x4078DA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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
            loop {
                unsafe { game_write(TICKS_RVA, g(TICKS_RVA).wrapping_add(1)) };
                gone -= TICK_MS;
                last = last.wrapping_add(TICK_MS);
                if gone < TICK_MS {
                    break;
                }
            }
            unsafe { game_write(LAST_TICK_RVA, last) };
            // One gate a tick: a self-test's pause waits for this one and holds the next.
            crate::framework::hook_enter();
            let ticked = (|| {
                part(6, &mut r)?;
                part(7, &mut r)?;
                unsafe { crate::input::input_tick::input_tick_in_game() }
            })();
            crate::framework::hook_leave();
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

/// In the game: through the game (place 1 then says whether DirectInput was made). In the
/// standalone program the platform's parts are SDL's: DirectInput counts as made and
/// nothing else is done but the priority.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if standalone() && place != 5 {
        if place == 1 {
            r.eax = 1;
        }
        return Ok(());
    }
    unsafe { call_game(r, game_addr(rva(CALLEES[place as usize]))) };
    if place == 1 {
        r.eax = (g(DIRECT_INPUT_RVA) != 0) as u32;
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { input_thread(&mut game_part) } {
        Ok(()) => unsafe { windows_sys::Win32::System::Threading::ExitThread(0) },
        // The original from the start: it finds its flag set and goes to its end.
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("it is the input thread: it runs until the game ends; its steps carry their tests")
}
