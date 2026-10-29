//! `physics::lever_pushes` (0x43DDDF): `dword_6140E4` and `dword_6140E8` 0; then, only
//! with `dword_621884` 1 and `dword_71A22C` 0: the lever `flt_614B44`, `B48` 0 and
//! `flt_614C44` 0; a share A / (B + A) of A = |+0x210| + |+0x21C| and B = |+0x214| +
//! |+0x218| (`g_car`, `dword_71A1E8` .. `dword_71A1F4`; a reading: the wheels' loads)
//! into `flt_614C44`, held to `flt_5C5D90` (0) ..= `flt_5C5D38` (1). If B + A is 0 (or no
//! number) the share is A itself and it ends there. Else `flt_614B4C` = `flt_71ACD4` and
//! `flt_614C48` the share, `physics::lever_push`; `flt_614B4C` = `flt_71ACC8` and
//! `flt_614C48` 1 less it, `physics::lever_push`. It ends through the `retn` at 0x43DFB0.
//!
//! In:  nothing
//! Out: as the second `lever_push` leaves them when it ran, else all kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B831, 0x43B8AA), every physics step.

use crate::math::x87;
use crate::math::x87_run::{lever_share, one_less};
use crate::physics::car_store::CAR;
use crate::physics::lever_push::lever_push;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::race::switches_store::FRAME_SWITCHES;
use std::cmp::Ordering;

/// `fld limit; fld share; fcompp` and its C0: share below limit, or unordered.
fn below(share: u32, limit: u32) -> bool {
    !matches!(
        f32::from_bits(share).partial_cmp(&f32::from_bits(limit)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version.
pub fn lever_pushes() {
    PHYSICS.lever_pushes[0].set(0);
    PHYSICS.lever_pushes[1].set(0);
    if FRAME_SWITCHES.levers_on.get() != 1 || CAR.all_wheels_off.get() != 0 {
        return;
    }
    PHYSICS.lever_push_vector[0].set(0);
    PHYSICS.lever_push_vector[1].set(0);
    PHYSICS.lever_share_start.set_bits(0);
    let (share, zero) = lever_share(
        CAR.grip_forces[0].bits(),
        CAR.grip_forces[3].bits(),
        CAR.grip_forces[1].bits(),
        CAR.grip_forces[2].bits(),
        NUMBERS.zero.bits(),
    );
    PHYSICS.lever_share_start.set_bits(share);
    if zero {
        return;
    }
    if below(PHYSICS.lever_share_start.bits(), NUMBERS.zero.bits()) {
        PHYSICS
            .lever_share_start
            .set_bits(x87::loaded(NUMBERS.zero.bits()));
    }
    if !below(PHYSICS.lever_share_start.bits(), NUMBERS.one.bits()) {
        PHYSICS
            .lever_share_start
            .set_bits(x87::loaded(NUMBERS.one.bits()));
    }
    PHYSICS.lever_push_vector[2].set(x87::loaded(CAR.probe_point_z[3].bits()));
    PHYSICS
        .lever_share_left
        .set_bits(x87::loaded(PHYSICS.lever_share_start.bits()));
    lever_push();
    PHYSICS.lever_push_vector[2].set(x87::loaded(CAR.probe_point_z[0].bits()));
    PHYSICS.lever_share_left.set_bits(one_less(
        crate::physics::tables_store::PHYSICS
            .lever_share_left
            .bits(),
    ));
    lever_push();
}
