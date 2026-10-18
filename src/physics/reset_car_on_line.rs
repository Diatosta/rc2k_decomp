//! `physics::reset_car_on_line` (0x43FB00): the work car put back on the line at the
//! checkpoint nearest it: `race::nearest_checkpoint` into `dword_71A070`, `6C`, `74`
//! (`g_car` +0x98, +0x94, +0x9C), `dword_6365E4` and `dword_6365EC`;
//! `race::place_car_on_line`, `race::reset_car_motion`; then +0xFB0 and +0xFB4
//! (`dword_71AF88`, `8C`) 8 and +0xFAC (`dword_71AF84`) 10.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: as `reset_car_motion` leaves them; no flags read (the caller's next is a `mov`)
//!
//! It begins with a call: hooked by a private copy with its three calls relocated and a
//! hand-made patch.
//! Caller: `physics::reset_car` (its end).
//! Refuses what the three calls refuse. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose calls go to the stand-ins, against the Rust version with the
//! same, on stream A's cases; the work car and the start's variables put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::physics::reset_parts::{both, copy_of, dispatch_reset, regions};
use crate::race::car_starts::{Case, Part, prepare};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::reset_car_on_line",
    rva(0x43FB00),
    thunk,
    &ORIGINAL,
)
.private_copy(0x47)
.relocate_calls(&[0, 0x1E, 0x23])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version, the calls through `part` (places 8, 6, 9).
pub unsafe fn reset_car_on_line(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        part(8, &mut g)?;
        for v in [0x71A070usize, 0x71A06C, 0x71A074, 0x6365E4, 0x6365EC] {
            game_write(rva(v), g.eax);
        }
        part(6, &mut g)?;
        part(9, &mut g)?;
        game_write(rva(0x71AF88), 8u32);
        game_write(rva(0x71AF8C), 8u32);
        game_write(rva(0x71AF84), 10u32);
    }
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, reset_car_on_line);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x43FB00,
        0x47,
        &[0xE8, 0x4A, 0x18, 0x00, 0x00],
        &[0, 0x1E, 0x23],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = regions();
    let saved = capture(&all);
    for seed in 1..=4u32 {
        let fill = || {
            prepare(
                &Case {
                    mode: seed % 4,
                    no_place: 0,
                    side: 0,
                    cars_2: 1,
                    current: seed % 4,
                    cars: 4,
                },
                seed.wrapping_mul(0x2026_1017),
            )
        };
        let entry = Regs {
            ebx: seed,
            ..Regs::SENTINEL
        };
        both(
            &mut t,
            &format!("seed {seed}"),
            &copy,
            0,
            entry,
            &fill,
            reset_car_on_line,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
