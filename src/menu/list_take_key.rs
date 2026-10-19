//! `menu::list_take_key` (0x41469A): a list menu's key: the pending key taken
//! (`input::take_key`, eax 0 first); none: a millisecond's sleep (`system::sleep`). A key:
//! the repeat count `dword_51D40F` reset unless it is the last key again (`byte_51D413`)
//! and `dword_51D41F - (dword_51D423 + 6)` is negative (`js`), then one more; the key
//! into `byte_51D413`.
//!
//! In:  nothing
//! Out: eax the key (0 for none); with the last key again ebx `dword_51D41F` and ecx
//!      `dword_51D423 + 6`; the flags of `test al, al` (no caller reads them)
//!
//! Callers: `race::replay_viewer`, `sub_4144CF`, `sub_4168CF`, `sub_4174C5`,
//! `sub_417726`, `sub_417A0D`. Its two calls through `part`: 1 `input::take_key`, 2
//! `system::sleep` (eax ms); in the game their Rust versions. Shadow mode: the original
//! first on the real key, the Rust version on the same key replayed. Self-test: a copy
//! whose two calls go to stand-ins (the key given), against the Rust version: no key, a
//! new key, the last key again with the clocks either side of the sign.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    FRAME_CLOCK, LAST_KEY, MOVE_CLOCK, Part, REPEATS, ZF, answer, copy, dword, gb, get, left_of,
    pb, put, region, run_both, test_part,
};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_keyed_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::list_take_key", rva(0x41469A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves (the flags of `test al, al`).
pub fn list_take_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = 0;
    part(1, &mut r)?;
    let key = r.eax as u8;
    if key == 0 {
        let mut s = Regs { eax: 1, ..r };
        part(2, &mut s)?;
    } else {
        let mut reset = true;
        if key == gb(LAST_KEY) {
            r.ebx = get(FRAME_CLOCK);
            r.ecx = get(MOVE_CLOCK).wrapping_add(6);
            reset = (r.ebx.wrapping_sub(r.ecx) as i32) >= 0;
        }
        if reset {
            put(REPEATS, 0);
        }
        put(REPEATS, get(REPEATS).wrapping_add(1));
        pb(LAST_KEY, key);
    }
    r.set_carry(false);
    r.set_zero(key == 0);
    Ok(r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 1 {
        let key = crate::input::take_key::take_key();
        r.eax = r.eax & !0xFF | key as u32;
    } else {
        crate::system::sleep::sleep(r.eax);
    }
    Ok(())
}

fn regions() -> Vec<crate::service::harness::Region> {
    vec![dword(REPEATS), region(LAST_KEY, 1)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&list_take_key(r, &mut game_part)?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // xor eax, eax; call input::take_key
    let copy = match copy(
        0x41469A,
        0x4A,
        &[0x33, 0xC0, 0xE8, 0xF4, 0x38, 0xFF, 0xFF],
        &[0x2, 0x41],
        &[(0x407F95, 1), (0x41100C, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = regions();
    all.extend([dword(FRAME_CLOCK), dword(MOVE_CLOCK)]);
    let saved = crate::race::viewer::Saved::new(&all);
    for (key, last, clock, moved) in [
        (0u8, 0x66u8, 10u32, 0u32),
        (0x66, 0x6B, 10, 0),
        (0x66, 0x66, 10, 0),
        (0x66, 0x66, 3, 0),
        (0x66, 0x66, 0x8000_0010, 0),
        (0x1C, 0x1C, 6, 0),
    ] {
        let prepare = || {
            pb(LAST_KEY, last);
            put(REPEATS, 7);
            put(FRAME_CLOCK, clock);
            put(MOVE_CLOCK, moved);
            answer(Box::new(move |place, r| {
                if place == 1 {
                    r.eax = r.eax & !0xFF | key as u32;
                    r.set_zero(key == 0);
                }
            }));
        };
        run_both(
            &mut t,
            &format!("key {key:#x}, last {last:#x}, clock {clock:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &regions(),
            ZF,
            &prepare,
            &|r| list_take_key(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
