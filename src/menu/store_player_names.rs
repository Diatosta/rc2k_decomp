//! `menu::store_player_names` (0x470649): copies each player's name into that player's car
//! and writes it to the game's log.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four car objects: the 0x20 bytes of the player's name to its +0x124,
//! and that copy as a line of the log (`logfile::log_write_l1_nl`). A name that fills its
//! 0x20 bytes is logged with what follows it in the car object.
//! One caller, `sub_47486C`, after the main menu is left.
//! Shadow mode: the original first, then the Rust version without the log lines on the
//! cars put back. Self-test: a redirected copy whose log calls note the text's address.
//! Refuses a missing car object.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::menu::player_names::{CAR_NAME, NAME_LEN, cars, name, regions, write_test_names};
use crate::menu::text_entry::log_l1;
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_replayed_state, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::store_player_names", rva(0x470649), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `log` gets the address of each line's text.
pub unsafe fn store_player_names(log: &mut dyn FnMut(*const u8)) -> Result<(), BadInput> {
    unsafe {
        for (player, car) in cars()?.into_iter().enumerate() {
            let copy = (car as usize + CAR_NAME) as *mut u8;
            std::ptr::copy(name(player), copy, NAME_LEN);
            log(copy);
        }
        Ok(())
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { store_player_names(&mut |line| log_l1(line, true)) }?;
    Ok(Left::nothing())
}

fn quiet(_: &Regs) -> Result<Left, Refused> {
    unsafe { store_player_names(&mut |_| {}) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    match unsafe { cars() } {
        Ok(cars) => {
            dispatch_replayed_state(&HOOK, r, call_original, &regions(&cars), &rust, &quiet)
        }
        Err(e) => {
            HOOK.count_call();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
        }
    }
}

// ---- self-test ----

const LEN: usize = 0xC3;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x50, 0xB8, 0x00, 0x00, 0x00, 0x00, 0x8B];
/// The four log calls.
const CALLS: [usize; 4] = [0x2B, 0x5B, 0x8B, 0xBB];

static LOGGED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn note_line(_: u32, r: &mut Regs) {
    LOGGED.lock().unwrap().push(r.eax);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(cars) = (unsafe { cars() }) else {
        return t.skip("a car object is missing");
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x470649)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions(&cars);
    let saved = capture(&regions);
    let before = Regs::SENTINEL;
    let run = |rust: bool| unsafe {
        restore(&regions, &vec![0xAA; saved.len()]);
        write_test_names();
        LOGGED.lock().unwrap().clear();
        set_handler(note_line);
        let ended = if rust {
            store_player_names(&mut |line| LOGGED.lock().unwrap().push(line as u32)).is_err() as u32
        } else {
            let mut ro = before;
            call_game(&mut ro, copy.addr());
            !ro.preserved_except(&before, 0) as u32
        };
        (ended, capture(&regions), LOGGED.lock().unwrap().clone())
    };
    let (theirs, mine) = (run(false), run(true));
    t.check(theirs == mine && theirs.0 == 0, || {
        format!(
            "four names: original ended {} logged {:x?}; rust ended {} logged {:x?}; first different byte left {:?}",
            theirs.0,
            theirs.2,
            mine.0,
            mine.2,
            theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
        )
    });
    restore(&regions, &saved);
    t.finish()
}
