//! `drive::reset_request` (0x43A0D3): the reset key held. Once the race is past 0x320
//! (`unk_719FEC`, by the sign): with the keys' bit 0x80 (`unk_71A440`), outside mode 2
//! (`dword_62389C`) and the car's speed below `flt_613828` (or unordered): a notice set up
//! on the first step (`unk_71A6F4` 0xCD, `unk_71A708` 0x19, `unk_71A704` 0x12C), the count
//! `unk_71A23C` + 1, at 100 the time penalty (0x2BC less `unk_71A1C0`) added to
//! `unk_71A060`, the notice cleared and `physics::reset_car`. Else a count under way is
//! cleared with the notice.
//!
//! In:  nothing
//! Out: ebx `unk_71A440` past the first test; at the reset as `physics::reset_car` leaves
//!      the registers (eax the penalty); the rest kept; no flags read
//!
//! Callers: `drive::car_controls`, `sub_460835`, `sub_460B2F`.
//! Refuses what `physics::reset_car` refuses. Shadow mode runs the original alone at the
//! reset (`physics/reset_parts.rs`), else compares.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) by lot, counts below, at and past 100; the reset's call through `reset_parts`'
//! stand-in in a copy.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, c0, g, p};
use crate::physics::reset_parts::{RESET_CAR, both_with, copy_of, dispatch_reset};
use crate::race::car_starts::Part;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::reset_request", rva(0x43A0D3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COUNT: usize = 0x71A23C;

fn clear_notice() {
    for x in [0x71A6FC, 0x71A6F4, 0x71A704, 0x71A700] {
        p(x, 0);
    }
}

/// The Rust version, the reset through `part` (place 15).
pub unsafe fn reset_request(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if (g(0x719FEC).wrapping_sub(0x320) as i32) < 0 {
        return Ok(r);
    }
    r.ebx = g(0x71A440);
    let held = r.ebx & 0x80 != 0 && g(0x62389C) != 2 && c0(g(0x71A19C), g(0x613828));
    if !held {
        if g(COUNT) != 0 {
            clear_notice();
            p(COUNT, 0);
        }
        return Ok(r);
    }
    if g(COUNT) == 0 {
        p(0x71A6F4, 0xCD);
        p(0x71A708, 0x19);
        p(0x71A704, 0x12C);
    }
    p(COUNT, g(COUNT).wrapping_add(1));
    if g(COUNT) != 0x64 {
        return Ok(r);
    }
    r.eax = 0x2BCu32.wrapping_sub(g(0x71A1C0));
    p(0x71A060, g(0x71A060).wrapping_add(r.eax));
    clear_notice();
    part(RESET_CAR, &mut r)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, reset_request);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x43A0D3,
        0xF7,
        &[0x81, 0x3D, 0xEC, 0x9F, 0x71, 0x00, 0x20, 0x03, 0x00, 0x00],
        &[0xB4],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let extra: Vec<Region> = vec![
        (a(0x719FEC), 4),
        (a(0x71A440), 4),
        (a(0x62389C), 4),
        (a(0x71A19C), 4),
        (a(0x613828), 4),
        (a(COUNT), 4),
        (a(0x71A6F4), 0x18),
        (a(0x71A1C0), 4),
        (a(0x71A060), 4),
    ];
    let saved = capture(&extra);
    for case in 0..24u32 {
        let fill = || {
            p(0x719FEC, [0x400u32, 0x100, 0x320][(case % 3) as usize]);
            p(0x71A440, if case % 4 == 3 { 0 } else { 0x80 });
            p(0x62389C, if case % 5 == 4 { 2 } else { 1 });
            p(
                0x71A19C,
                [1.0f32, 5.0, f32::NAN][(case / 3 % 3) as usize].to_bits(),
            );
            p(0x613828, 3.0f32.to_bits());
            p(COUNT, [0u32, 0x63, 0x64, 7][(case / 6 % 4) as usize]);
            p(0x71A1C0, 0x55);
            p(0x71A060, 1000);
        };
        both_with(
            &mut t,
            &format!("case {case}"),
            &copy,
            0,
            Regs::SENTINEL,
            &fill,
            reset_request,
            &extra,
        );
    }
    restore(&extra, &saved);
    drop(copy);
    t.finish()
}
