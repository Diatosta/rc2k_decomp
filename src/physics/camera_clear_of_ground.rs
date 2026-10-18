//! `physics::camera_clear_of_ground` (0x438DD2): whether the camera is a whole unit
//! (0x10000) or more above the ground under it (`unk_71A680` - `dword_639828`, by the
//! sign of the `cmp`; `physics/camera_ground.rs`): CF clear when found and so high, else
//! CF.
//!
//! In:  nothing
//! Out: CF; eax as the steps leave it, the rest as `count_ground_probe` leaves them (the
//!      caller reads CF)
//!
//! Caller: `sub_43907F` (three).
//! Refuses what `count_ground_probe` refuses. Shadow mode runs the original alone.
//! Self-test: a copy whose probe goes to a stand-in, against the Rust version with the
//! same: found or not, heights about the limit.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::camera_ground::{FOUND, both_carry, dispatch_carry, probe_at, test_regions};
use crate::physics::flow::{g, p};
use crate::physics::reset_parts::copy_of;
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::camera_clear_of_ground",
    rva(0x438DD2),
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

/// The Rust version, the probe through `part`; CF in the registers left.
pub unsafe fn camera_clear_of_ground(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(0x71A67C);
    let x = r.eax;
    r.eax = g(0x71A684);
    let z = r.eax;
    let mut clear = probe_at(&mut r, part, x, z, 0x71AEC8)?;
    if clear {
        r.eax = g(0x71A680).wrapping_sub(g(0x639828));
        clear = (r.eax.wrapping_sub(0x10000) as i32) >= 0;
    }
    r.set_carry(!clear);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_carry(&HOOK, r, call_original, camera_clear_of_ground);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x438DD2,
        0x50,
        &[0xFF, 0x35, 0xC8, 0xAE, 0x71, 0x00],
        &[0x20],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = crate::physics::reset_parts::regions();
    all.extend(test_regions());
    let saved = capture(&all);
    for case in 0..10u32 {
        let fill = || {
            restore(&all, &saved);
            p(0x71A67C, 0x1234_0000);
            p(0x71A684, 0x0567_0000);
            p(0x71AEC8, 0x11);
            p(0x5CCC74, 0x99);
            p(FOUND, (case % 5 != 4) as u32);
            p(0x639828, 0x2_0000);
            p(
                0x71A680,
                [0x3_0000u32, 0x2_FFFF, 0x2_0000, 0x8002_0000, 0x3_0000][(case % 5) as usize],
            );
        };
        both_carry(
            &mut t,
            &format!("case {case}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            camera_clear_of_ground,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
