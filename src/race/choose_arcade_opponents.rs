//! `race::choose_arcade_opponents` (0x449B79): the arcade race's three opponents' kinds
//! (`dword_62385C`, `60`, `64`) drawn from the noise seed and then `math::next_random`:
//! usable kinds within five places of the first car's, not taken; each logged.
//!
//! In:  nothing
//! Out: (mode 2) eax = the last place difference, ebx = its place, ecx = as
//!      `index_of_kind`, edx = its kind, ebp = the first car's place; else nothing
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_arcade_cars`. Refuses, before
//! anything, a first kind with fewer than three possible opponents (the original would
//! draw for ever). Shadow mode runs the original alone (it logs). Self-test: a copy whose
//! log calls note themselves (the draws and the kind tests the real ones, from the same
//! states), against the Rust version on the same: first kinds with many and with just
//! three opponents, seeds by lot, mode not 2; the refusal on the Rust version alone.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::race_cars::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::choose_arcade_opponents",
    rva(0x449B79),
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { choose_arcade_opponents(r, &mut cars_part) } {
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
        let g = unsafe { choose_arcade_opponents(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

const CALLS: [usize; 28] = [
    0x12, 0x20, 0x2C, 0x39, 0x45, 0x52, 0x5B, 0x6D, 0x7B, 0x82, 0xAC, 0xB8, 0xC5, 0xCE, 0xD4, 0xE2,
    0xE9, 0x11B, 0x127, 0x134, 0x13D, 0x143, 0x151, 0x158, 0x192, 0x19E, 0x1AB, 0x1B4,
];

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x449B79,
        0x1BB,
        &[0x83, 0x3D, 0x9C, 0x38, 0x62, 0x00, 0x02],
        &CALLS,
        CAR_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = opponent_regions();
    let saved = capture(&regions);
    for (mode, first, seed, random) in [
        (2u32, 0u32, 0x1234_5678u32, 0x2026_1017u32),
        (2, 15, 0xDEAD_BEEF, 1),
        (2, 3, 0, 0x8000_0001),
        (2, 7, 77, 0x5555_5555),
        (2, 8, 0xFFFF_FFFF, 0xAAAA_AAAA),
        (0, 1, 5, 5),
    ] {
        run_cars(
            &mut t,
            &format!("mode {mode}, first {first}, seed {seed:#x}, random {random:#x}"),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(MODE_RVA, mode);
                game_write(CAR_KINDS_RVA, first);
                game_write(rva(0x60519C), seed);
                game_write(crate::math::next_random::STATE_RVA, random);
            },
            &|r| unsafe { choose_arcade_opponents(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
