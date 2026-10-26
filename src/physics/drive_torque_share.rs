//! `physics::drive_torque_share` (0x440D56): a share of the drive by speed. n =
//! `dword_613EEC` (8) with `g_carDriveKind` 1, else `dword_613EF0` (32), into
//! `dword_62400C`; s = |`g_carSpeed`| / `dword_613EF4` (`fidiv`) into `flt_62401C`, held to
//! at most `flt_5C5D38` (1.0; one below it, or unordered, is left); d = (n - `g_car`
//! +0x28, `unk_71A000`) / n (`fild` / `fidiv`) into `dword_624020`; `flt_614758` = d +
//! (1 - d) s, and `flt_614750` times it.
//!
//! In:  nothing
//! Out: eax n less the word (no `pusha`); the rest kept; no flags read (the callers return
//!      or load floats next)
//!
//! Callers: `sub_421848`, `sub_43CAAB`, `sub_43CB14`.

use crate::math::x87;
use crate::math::x87_run::{abs_fidiv, int_fidiv, one_less_times_plus};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use std::cmp::Ordering;

fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::drive_torque_share's operand")
}

fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::drive_torque_share's operand")
}

/// The core on the variables' addresses: eax as the original leaves it (n less the
/// word, its `sub eax, unk_71A000`).
pub fn drive_torque_share_at(v: &[u32]) -> u32 {
    let n = if at(v[1]) == 1 {
        PHYSICS.torque_share_a.get()
    } else {
        PHYSICS.torque_share_b.get()
    };
    put(v[0], n);
    put(
        v[3],
        abs_fidiv(at(v[2]), PHYSICS.torque_share_c.get() as i32),
    );
    let one = NUMBERS.one.bits();
    let below = !matches!(
        f32::from_bits(at(v[3])).partial_cmp(&f32::from_bits(one)),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !below {
        put(v[3], x87::loaded(one));
    }
    let eax = at(v[0]).wrapping_sub(at(v[5]));
    put(v[4], eax);
    put(v[4], int_fidiv(at(v[4]) as i32, at(v[0]) as i32));
    put(v[6], one_less_times_plus(at(v[4]), at(v[3])));
    put(v[7], x87::mul(at(v[7]), at(v[6])));
    eax
}

fn game() -> Vec<u32> {
    vec![
        DRAW.scratch.addr(),
        CAR.drive_kind.addr(),
        CAR.local_velocity[2].addr(),
        DRAW.work_float_a.addr(),
        DRAW.work_float_b.addr(),
        CAR.gear_ratio_setting.addr(),
        PHYSICS.torque_share.addr(),
        PHYSICS.drive_torque.addr(),
    ]
}

/// The Rust version (the exact one, for `physics::drive_force` and
/// `physics::test_drive_run`): eax as the original leaves it.
pub fn drive_torque_share() -> u32 {
    drive_torque_share_at(&game())
}
