//! `physics::reset_when_stuck` (0x43EFBA): with the grades' mode (`dword_71A234`, `g_car`
//! +0x25C) 0 or 2 the stuck count (+0x1E8, `dword_71A1C0`) is set to 0; else it goes up,
//! and at 700 or more (unsigned) `physics::reset_car` puts the car back on the line.
//!
//! In:  nothing
//! Out: when `reset_car` ran, the registers as it leaves them; else all kept; no flags
//!      read (the caller's next is a call)
//!
//! Caller: `sub_43B635` (0x43B900), every physics step.
//! Refuses what `reset_car` refuses. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose call goes to a stand-in (place 15), against the Rust version
//! with the same: modes 0 .. 3 and counts about 700.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::physics::reset_parts::{RESET_CAR, both, copy_of, dispatch_reset, regions};
use crate::race::car_starts::{Case, Part, prepare};
use crate::service::harness::{capture, restore};
use rc2k_core::physics::start::stuck_tick;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::reset_when_stuck", rva(0x43EFBA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MODE: usize = rva(0x71A234); // g_car + 0x25C
const STUCK: usize = rva(0x71A1C0); // g_car + 0x1E8

/// The Rust version, `reset_car` through `part` (place 15).
// Converted: `rc2k_core::physics::start::stuck_tick`.
pub unsafe fn reset_when_stuck(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        let mode: u32 = game_read(MODE);
        let mut stuck = if mode == 2 || mode == 0 {
            0
        } else {
            game_read::<u32>(STUCK)
        };
        let reset = stuck_tick(mode, &mut stuck);
        game_write(STUCK, stuck);
        if reset {
            part(RESET_CAR, &mut r)?;
        }
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, reset_when_stuck);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x43EFBA,
        0x35,
        &[0x83, 0x3D, 0x34, 0xA2, 0x71, 0x00, 0x02],
        &[0x24],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = regions();
    let saved = capture(&all);
    for mode in 0..4u32 {
        for stuck in [0u32, 698, 699, 700, 0xFFFF_FFFF] {
            let fill = || unsafe {
                prepare(
                    &Case {
                        mode: 2,
                        no_place: 0,
                        side: 0,
                        cars_2: 1,
                        current: mode,
                        cars: 4,
                    },
                    (mode + stuck).wrapping_mul(0x2026_1017),
                );
                game_write(MODE, mode);
                game_write(STUCK, stuck);
            };
            both(
                &mut t,
                &format!("mode {mode}, stuck {stuck:#x}"),
                &copy,
                0,
                Regs::SENTINEL,
                &fill,
                reset_when_stuck,
            );
        }
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
