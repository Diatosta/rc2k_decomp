//! `physics::lever_pushes` (0x43DDDF): `dword_6140E4` and `dword_6140E8` 0; then, only
//! with `dword_621884` 1 and `dword_71A22C` 0: the lever `flt_614B44`, `B48` 0 and
//! `flt_614C44` 0; a share A / (B + A) of A = |+0x210| + |+0x21C| and B = |+0x214| +
//! |+0x218| (`g_car`, `dword_71A1E8` .. `dword_71A1F4`; a reading: the wheels' loads)
//! into `flt_614C44`, held to `flt_5C5D90` (0) ..= `flt_5C5D38` (1). If B + A is 0 (or no
//! number) the share is A itself and it ends there. Else `flt_614B4C` = `flt_71ACD4` and
//! `flt_614C48` the share, `physics::lever_push`; `flt_614B4C` = `flt_71ACC8` and
//! `flt_614C48` 1 less it, `physics::lever_push`. It ends through the `retn` at 0x43DFB0.
//!
//! Callers: `physics::car_physics_step` (0x43B831, 0x43B8AA), every physics step.

use crate::math::x87;
use crate::math::x87_run::{lever_share, one_less};
use crate::physics::lever_push::lever_push;
use crate::race::owner::Race;
use std::cmp::Ordering;

/// `fld limit; fld share; fcompp` and its C0: share below limit, or unordered.
fn below(share: u32, limit: u32) -> bool {
    !matches!(
        f32::from_bits(share).partial_cmp(&f32::from_bits(limit)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version.
pub fn lever_pushes(race: &Race, scene: &crate::scene::owner::Scene) {
    race.shared.physics_tables.lever_pushes[0].set(0);
    race.shared.physics_tables.lever_pushes[1].set(0);
    if race.shared.switches.levers_on.get() != 1 || race.shared.car.all_wheels_off.get() != 0 {
        return;
    }
    race.shared.physics_tables.lever_push_vector[0].set(0);
    race.shared.physics_tables.lever_push_vector[1].set(0);
    race.shared.physics_tables.lever_share_start.set_bits(0);
    let (share, zero) = lever_share(
        race.shared.car.grip_forces[0].bits(),
        race.shared.car.grip_forces[3].bits(),
        race.shared.car.grip_forces[1].bits(),
        race.shared.car.grip_forces[2].bits(),
        race.course.numbers.zero.bits(),
    );
    race.shared.physics_tables.lever_share_start.set_bits(share);
    if zero {
        return;
    }
    if below(
        race.shared.physics_tables.lever_share_start.bits(),
        race.course.numbers.zero.bits(),
    ) {
        race.shared
            .physics_tables
            .lever_share_start
            .set_bits(x87::loaded(race.course.numbers.zero.bits()));
    }
    if !below(
        race.shared.physics_tables.lever_share_start.bits(),
        race.course.numbers.one.bits(),
    ) {
        race.shared
            .physics_tables
            .lever_share_start
            .set_bits(x87::loaded(race.course.numbers.one.bits()));
    }
    race.shared.physics_tables.lever_push_vector[2]
        .set(x87::loaded(race.shared.car.probe_point_z[3].bits()));
    race.shared
        .physics_tables
        .lever_share_left
        .set_bits(x87::loaded(
            race.shared.physics_tables.lever_share_start.bits(),
        ));
    lever_push(race, scene);
    race.shared.physics_tables.lever_push_vector[2]
        .set(x87::loaded(race.shared.car.probe_point_z[0].bits()));
    race.shared
        .physics_tables
        .lever_share_left
        .set_bits(one_less(race.shared.physics_tables.lever_share_left.bits()));
    lever_push(race, scene);
}
