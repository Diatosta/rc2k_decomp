//! `physics::camera_low_switch` (0x438BF4): with `unk_71A0A0` below 0 (or unordered), no
//! switch under way (`unk_71A66C` 0) and the camera in mode 0 or 1 (`unk_71A65C`): the
//! ground under the camera (`unk_71A67C`, `unk_71A684`; `physics/camera_ground.rs`);
//! found and the camera less than 0x800 above it (`unk_71A680` - `dword_639828`, by the
//! sign of the `cmp`): the switch begun: `unk_71A664` = the mode before (`unk_71A660`)
//! plus 1, `unk_71A660` = 4, `unk_71A66C` = 1, `unk_71A674` = 0, `unk_71A670` = 10,
//! `unk_71A668` = the clock (`unk_719FE0`).
//!
//! In:  nothing
//! Out: eax as the steps leave it, the rest as `count_ground_probe` leaves them; no flags
//!      read (the caller goes on)
//!
//! Caller: `sub_437A53`.
//! Refuses what `count_ground_probe` refuses. Shadow mode runs the original alone.
//! Self-test: a copy whose probe goes to a stand-in, against the Rust version with the
//! same: every early way out, found or not, low or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::camera_ground::{FOUND, both_carry, dispatch_carry, probe_at, test_regions};
use crate::physics::flow::{c0, g, p};
use crate::physics::reset_parts::copy_of;
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use rc2k_core::physics::camera_ground::{is_low, low_switch, low_switch_looks, over};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::camera_low_switch",
    rva(0x438BF4),
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

/// The Rust version, the probe through `part`.
// Converted: `rc2k_core::physics::camera_ground::{low_switch_looks, is_low, low_switch}`.
pub unsafe fn camera_low_switch(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if !low_switch_looks(c0(g(0x71A0A0), g(0x5C5D90)), g(0x71A66C), g(0x71A65C)) {
        return Ok(r);
    }
    r.eax = g(0x71A67C);
    let x = r.eax;
    r.eax = g(0x71A684);
    let z = r.eax;
    if !probe_at(&mut r, part, x, z, 0x71AEC8)? {
        return Ok(r);
    }
    r.eax = over(g(0x71A680), g(0x639828));
    if !is_low(r.eax) {
        return Ok(r);
    }
    let s = low_switch(g(0x71A660), g(0x719FE0));
    p(0x71A664, s.before);
    p(0x71A660, s.mode);
    p(0x71A66C, s.switching);
    p(0x71A674, s.w674);
    p(0x71A670, s.w670);
    p(0x71A668, s.clock);
    r.eax = s.clock;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_carry(&HOOK, r, call_original, camera_low_switch);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x438BF4,
        0xC6,
        &[0xD9, 0x05, 0xA0, 0xA0, 0x71, 0x00],
        &[0x5C],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = crate::physics::reset_parts::regions();
    all.extend(test_regions());
    let saved = capture(&all);
    for case in 0..24u32 {
        let fill = || {
            restore(&all, &saved);
            p(
                0x71A0A0,
                [0xBF80_0000u32, 0x3F80_0000, 0x8000_0000, 0x7FC0_0000][(case % 4) as usize],
            );
            p(0x71A66C, (case % 5 == 4) as u32);
            p(0x71A65C, [0u32, 1, 2, 0][(case / 4 % 4) as usize]);
            p(0x71A660, case);
            p(0x71A67C, 0x1234_0000);
            p(0x71A684, 0x0567_0000);
            p(0x71AEC8, 0x77);
            p(0x5CCC74, 0x99);
            p(FOUND, (case / 2 % 2) ^ (case / 8 % 2));
            p(0x639828, 0x1000);
            p(
                0x71A680,
                [0x1000u32, 0x17FF, 0x1800, 0x0800, 0x8000_0000][(case % 5) as usize],
            );
            p(0x719FE0, 0xABCD + case);
        };
        both_carry(
            &mut t,
            &format!("case {case}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            camera_low_switch,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
