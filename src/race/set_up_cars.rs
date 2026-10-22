//! `race::set_up_cars` (0x43603C): race init's "cars": "gamemode" and `dword_62389C`
//! logged; both players' +0x5FC 0 and +0xA0C .. +0xA1C -1; then by the mode: 2 the
//! numbers, the times, the arcade set-up and `dword_601C68` (0x40 dwords) 0; 1, 3, 4
//! their set-ups; 0 the numbers and its set-up; others nothing.
//!
//! In:  nothing
//! Out: the registers as the mode's steps leave them (mode 2: edi past `dword_601C68`'s
//!      0x40 dwords, eax and ecx 0)
//!
//! See `race/race_cars.rs`. One caller, `sub_4462B9` (race init). Shadow mode runs the
//! original alone (it logs). Self-test: a copy whose calls note themselves, against the
//! Rust version on the same, modes 0 .. 5.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::race_cars::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: &[usize] = &[0x07, 0x13, 0xBF, 0xC4, 0xC9, 0xE1, 0xE7, 0xED, 0xF2, 0xF8];

pub static HOOK: RawHook =
    RawHook::new("race::set_up_cars", rva(0x43603C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_up_cars(r, &mut cars_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_up_cars(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

/// The stand-ins' part, but `scene::next_noise` (place 18) the real one, as the copy's.
fn part(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if place == 18 {
        cars_part(place, r)
    } else {
        crate::race::mud::test_part(place, r)
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::race::mud::copy(
        0x43603C,
        0xFE,
        &[0x60, 0x8D, 0x05, 0xEB, 0xB9, 0x4D, 0x00],
        CALLS,
        CAR_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = arcade_regions();
    let saved = capture(&regions);
    let _ = game_write::<u32> as unsafe fn(usize, u32);
    for mode in 0..6u32 {
        crate::race::race_cars::run_cars(
            &mut t,
            &format!("mode {mode}"),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| unsafe {
                prepare(0x4242_0000 ^ mode, mode, 1, 2, [0, 0]);
                prepare_set_up(0x22, 0, [0, 0]);
                game_write(rva(0x601C68), 0x5A5A_5A5Au32);
            },
            &|r| unsafe { set_up_cars(r, &mut part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
