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
//! Self-test: a copy of the original (its variables the test's) against the converted
//! version: both drive kinds, speeds about the limit, words by lot, exact at double
//! precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::{abs_fidiv, int_fidiv, one_less_times_plus};
use std::cmp::Ordering;

/// The variables (IDA addresses), in the order the core takes them.
pub const VARIABLES: [usize; 8] = [
    0x62400C, 0x71A018, 0x71A184, 0x62401C, 0x624020, 0x71A000, 0x614758, 0x614750,
];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The core on the variables' addresses: eax as the original leaves it (n less the
/// word, its `sub eax, unk_71A000`).
pub unsafe fn drive_torque_share_at(v: &[u32]) -> u32 {
    unsafe {
        let n = if at(v[1]) == 1 {
            k(0x613EEC)
        } else {
            k(0x613EF0)
        };
        put(v[0], n);
        put(v[3], abs_fidiv(at(v[2]), k(0x613EF4) as i32));
        let one = k(0x5C5D38);
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
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::drive_force` and
/// `physics::test_drive_run`): eax as the original leaves it.
pub unsafe fn drive_torque_share() -> u32 {
    unsafe { drive_torque_share_at(&game()) }
}
