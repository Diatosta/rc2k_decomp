//! `saves::save_game_data` (0x411499): writes what the game keeps between runs.
//!
//! In:  nothing
//! Out: every register preserved; the flags as `save_best_times` leaves them
//! `saves::save_options`, then `saves::save_best_times`.
//! Callers: the main loop after the front end is left, and `sub_48AD73`.
//! Shadow mode runs the original alone: its two parts are compared in their own hooks.
//! Manual patch with a private copy (0x0D bytes, both calls relocated): the first call
//! starts at the second byte.
//! Self-test: a redirected copy whose two calls only note themselves.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("saves::save_game_data", rva(0x411499), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The two parts, each saying whether it wrote its file. Returns what the second said.
pub unsafe fn save_game_data(
    save_options: &mut dyn FnMut() -> Result<bool, Refused>,
    save_best_times: &mut dyn FnMut() -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    save_options()?;
    save_best_times()
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let result = unsafe {
        save_game_data(
            &mut || {
                crate::saves::save_options::save_options(
                    game_addr(rva(0x5295D8)) as *const u8, // "var\ral.dat"
                )
            },
            &mut || crate::saves::save_best_times::in_game(),
        )
    };
    match result {
        Ok(written) => {
            r.set_carry(!written);
            if written {
                r.set_zero(false);
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x0D;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0xB8, 0x99, 0x09, 0x00, 0xE8, 0x92];
const CALLS: [usize; 2] = [0x01, 0x06];
const PARTS: [usize; 2] = [rva(0x4AAE57), rva(0x4AB236)];

/// The parts called, in order.
static CALLED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// Notes the call; the second part's carry is the test's.
fn handler(place: u32, r: &mut Regs) {
    let mut called = CALLED.lock().unwrap();
    called.push(place);
    if place == 1 {
        r.set_carry(called[0] == 9);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x411499)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                PARTS
                    .iter()
                    .position(|&part| part == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // With the second part saying it wrote, and saying it did not (a 9 put first in the
    // notes tells the handler which).
    for fails in [false, true] {
        let begin = || {
            let mut called = CALLED.lock().unwrap();
            called.clear();
            if fails {
                called.push(9);
            }
            set_handler(handler);
        };
        let before = Regs::SENTINEL;
        begin();
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            ro.preserved_except(&before, 0),
            !ro.carry(),
            CALLED.lock().unwrap().clone(),
        );
        begin();
        let part = |place: u32| {
            let mut r = Regs::default();
            act(place, &mut r);
            Ok(!r.carry())
        };
        let written = unsafe { save_game_data(&mut || part(0), &mut || part(1)) };
        let mine = (
            true,
            written.unwrap_or(false),
            CALLED.lock().unwrap().clone(),
        );
        t.check(theirs == mine, || {
            format!("the second part fails={fails}: original {theirs:?}; rust {mine:?}")
        });
    }
    t.finish()
}
