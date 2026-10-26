//! `physics::find_car_checkpoint` (0x44139D): the work car's checkpoint looked for near
//! the last one. The car's x and z into `dword_639824` / `dword_63982C`; with no
//! checkpoints eax 0. Else `race::nearest_checkpoint_in` over `g_car` +0x98 - 1 ..
//! +0x98 + 1 (ecx, edx); its distance (edi) through `dword_62400C`, / 65536 into
//! `g_nearestCheckpointDistance`; above 0xA0000 (unsigned) `race::nearest_checkpoint`
//! over them all (eax), else eax the nearest (ebp).
//!
//! In:  nothing (the registers are passed on)
//! Out: eax the checkpoint; the rest as the calls leave them; no flags read (the caller
//!      stores eax)
//!
//! Caller: `sub_44157B`.
//! Refuses what the two calls refuse. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose calls go to the stand-ins (places 17 and 8), against the Rust
//! version with the same: no checkpoints, distances below and above 0xA0000.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87_run::{ext_div2, int_80};
use crate::physics::reset_parts::{NEAREST_IN, both_with, copy_of, dispatch_reset, regions};
use crate::race::car_starts::{Case, Part, prepare};
use crate::service::harness::{Region, capture, restore};
use rc2k_core::physics::progress;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::find_car_checkpoint",
    rva(0x44139D),
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

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const COUNT: usize = 0x623640; // g_checkpointCount
const DISTANCE: usize = 0x614C68; // g_nearestCheckpointDistance

/// The Rust version, the calls through `part` (places 17 and 8).
// Converted: `rc2k_core::physics::progress::{near_window, search_all}`.
pub unsafe fn find_car_checkpoint(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(0x639824, g(0x71A10C));
    r.eax = g(0x71A114);
    p(0x63982C, r.eax);
    if g(COUNT) == 0 {
        r.eax = 0;
        return Ok(r);
    }
    (r.ecx, r.edx) = progress::near_window(g(0x71A070));
    part(NEAREST_IN, &mut r)?;
    p(0x62400C, r.edi);
    p(
        DISTANCE,
        ext_div2(int_80(r.edi as i32, false), g(0x5C5D98), 0x3F80_0000),
    );
    if progress::search_all(r.edi) {
        part(8, &mut r)?;
    } else {
        r.eax = r.ebp;
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, find_car_checkpoint);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x44139D,
        0x5B,
        &[0xA1, 0x0C, 0xA1, 0x71, 0x00],
        &[0x27, 0x55],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let extra: [Region; 4] = [
        (a(0x639824), 12),
        (a(0x62400C), 4),
        (a(DISTANCE), 4),
        (a(COUNT), 4),
    ];
    let mut all = regions();
    all.extend_from_slice(&extra);
    let saved = capture(&all);
    for (count, edi) in [
        (0u32, 5u32),
        (40, 0x9_FFFF),
        (40, 0xA_0000),
        (40, 0xA_0001),
        (40, u32::MAX),
    ] {
        let fill = || {
            prepare(
                &Case {
                    mode: 2,
                    no_place: 0,
                    side: 0,
                    cars_2: 1,
                    current: 1,
                    cars: 4,
                },
                edi ^ count,
            );
            p(COUNT, count);
            p(0x71A070, 7);
        };
        let entry = Regs {
            edi,
            ebp: 0x1234,
            ..Regs::SENTINEL
        };
        both_with(
            &mut t,
            &format!("count {count}, distance {edi:#x}"),
            &copy,
            0,
            entry,
            &fill,
            find_car_checkpoint,
            &extra,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
