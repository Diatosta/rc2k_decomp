//! `saves::keep_player_settings` (0x4719F5, `maybeCopySomeData` in IDA): the players'
//! own settings copied into the saved game's four cars before those are put over the
//! players, so that a loaded game keeps the controls and choices made since: for each of
//! the four players, the bytes +0x1C .. +0x27 (the keys), the dwords +0x2C .. +0x78 and
//! +0x178, into the saved car at esi + n x 0x28C.
//!
//! In:  esi the saved cars (four of 0x28C bytes)
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `saves::load_savegame` (when `dword_755144` is 0). Refuses (before anything) a
//! missing player or esi 0. Self-test: a buffer of the test's (0xAA) from the game's
//! players, compared whole.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::player::{PLAYERS, player_at};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "saves::keep_player_settings",
    rva(0x4719F5),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CAR_LEN: u32 = rc2k_core::saves::cars::SAVED_CAR_LEN as u32;

pub unsafe fn keep_player_settings(cars: u32) -> Result<(), Refused> {
    if cars == 0 || (0..PLAYERS).any(|n| player_at(n) == 0) {
        return Err(BadInput("a player missing, or no saved cars").into());
    }
    for n in 0..PLAYERS {
        // Converted: `rc2k_core::saves::cars::keep_settings`.
        let (from, to) = (player_at(n), cars.wrapping_add(n * CAR_LEN));
        let player = unsafe { crate::bridge::Raw::new(from as *const u8) };
        crate::bridge::read_through(rc2k_core::saves::cars::keep_settings(
            &player,
            &mut |at, b| unsafe { *(to.wrapping_add(at as u32) as *mut u8) = b },
        ));
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { keep_player_settings(r.esi)? };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 4 * CAR_LEN as usize)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if (0..PLAYERS).any(|n| player_at(n) == 0) {
        return t.skip("a player missing from g_players");
    }
    let mut buffer = vec![0xAAu8; 4 * CAR_LEN as usize];
    let at = buffer.as_mut_ptr() as u32;
    let regions = [(at, buffer.len())];
    check_state(
        &mut t,
        "the game's players",
        call_original,
        Regs {
            esi: at,
            ..Regs::SENTINEL
        },
        &|| unsafe { std::ptr::write_bytes(at as *mut u8, 0xAA, 4 * CAR_LEN as usize) },
        &regions,
        &rust,
    );
    drop(buffer);
    t.finish()
}
