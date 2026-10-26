//! `physics::camera_over_ground` (0x438CEC): whether there is ground under the camera
//! and a half-unit (0x8000) to either side across its heading (`unk_71A688`,
//! `math::turn_point_xz` of (-0x8000, y, 0) and (0x8000, y, 0)), each probe from its own
//! last face (`unk_71AEC8`, `unk_71AECC`, `unk_71AED0`; `physics/camera_ground.rs`).
//! CF clear when all three are found; the first not found sets CF and ends it.
//!
//! In:  nothing (ebx passed to the turns as y)
//! Out: CF; eax, ecx, edx as the turns and probes leave them, the rest as
//!      `count_ground_probe` leaves them (the caller reads CF)
//!
//! Caller: `sub_43907F` (four).
//! Refuses what `count_ground_probe` refuses. Shadow mode runs the original alone.
//! Self-test: a copy whose probes go to a stand-in, against the Rust version with the
//! same: found and not, headings by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::turn_point_xz::turn_point_xz;
use crate::physics::camera_ground::{FOUND, both_carry, dispatch_carry, probe_at, test_regions};
use crate::physics::flow::{g, p};
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use rc2k_core::physics::camera_ground::SIDES;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::camera_over_ground",
    rva(0x438CEC),
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

// Converted: `rc2k_core::physics::camera_ground::SIDES`.
fn steps(r: &mut Regs, part: Part) -> Result<bool, Refused> {
    r.eax = g(0x71A67C);
    let x = r.eax;
    r.eax = g(0x71A684);
    let z = r.eax;
    if !probe_at(r, part, x, z, 0x71AEC8)? {
        return Ok(false);
    }
    for (side, slot) in SIDES.into_iter().zip([0x71AECC, 0x71AED0]) {
        p(0x5CCC78, g(slot));
        let [x, _, z, edx] = unsafe { turn_point_xz(side as i32, r.ebx as i32, 0, g(0x71A688)) };
        r.eax = x.wrapping_add(g(0x71A67C));
        r.ecx = z.wrapping_add(g(0x71A684));
        r.edx = edx;
        let (x, z) = (r.eax, r.ecx);
        if !probe_at(r, part, x, z, slot)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The Rust version, the probes through `part`; CF in the registers left.
pub unsafe fn camera_over_ground(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let all = steps(&mut r, part)?;
    r.set_carry(!all);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_carry(&HOOK, r, call_original, camera_over_ground);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Only the probes go to the stand-in: the two turns (0x5A, 0xAC) are the game's (the
    // Rust version calls `turn_point_xz` itself).
    crate::install::stand_ins::set_handler(crate::physics::reset_parts::handler);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            crate::framework::game_addr(rva(0x438CEC)),
            0xE6,
            &[0xFF, 0x35, 0xC8, 0xAE, 0x71, 0x00],
            &[0x20, 0x5A, 0x76, 0xAC, 0xC8],
            &|c| {
                (c == rva(0x438E22)).then(|| {
                    crate::install::stand_ins::stand_in(crate::physics::reset_parts::COUNT_PROBE)
                })
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = crate::physics::reset_parts::regions();
    all.extend(test_regions());
    let saved = capture(&all);
    for case in 0..12u32 {
        let fill = || {
            restore(&all, &saved);
            p(
                0x71A67C,
                0x1234_0000u32.wrapping_add(case.wrapping_mul(0x3_1415)),
            );
            p(
                0x71A684,
                0x0567_0000u32.wrapping_sub(case.wrapping_mul(0x2_7182)),
            );
            p(0x71A688, case.wrapping_mul(0x1357_9BDF));
            p(0x71AEC8, 0x11);
            p(0x71AECC, 0x22);
            p(0x71AED0, 0x33);
            p(0x5CCC74, 0x99);
            p(FOUND, (case % 3 != 2) as u32);
        };
        both_carry(
            &mut t,
            &format!("case {case}"),
            &copy,
            crate::framework::Regs {
                ebx: case.wrapping_mul(0x1111),
                ..Regs::SENTINEL
            },
            &fill,
            camera_over_ground,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
