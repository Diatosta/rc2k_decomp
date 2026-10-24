//! `physics::set_handling` (0x43EB5D): `physics::grade_or_reset`, then nine handling
//! figures from the car's setup (whole numbers in the work copy, 0 .. 256 by the look of
//! it): each lo + n / 256 (hi - lo) between two constants of 0x613F1C .. 0x613F88:
//! `flt_614C0C`, `flt_614C08` (by `dword_71A014`), `flt_614BF0` (`dword_719FF8`),
//! `flt_614BF4`, `flt_614BFC` (`dword_71A008`; the second from the pair for
//! `g_carDriveKind`, then times `dword_614C18`), `flt_614BEC`, `flt_614BF8`, `flt_614C00`,
//! `flt_614C04` (`dword_71A00C`). What each is, is a reading.
//!
//! In:  nothing
//! Out: ebx `g_carDriveKind`; the rest as `grade_or_reset` leaves them; no flags read
//!
//! Callers: `sub_43B635` (every physics step) and `race::settle_car`. Refuses (before
//! anything) a drive kind above 1 (the cars' records have 0 and 1; the pair tables hold
//! two).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::int_lerp;
use crate::physics::grade_or_reset::grade_or_reset_at;
use crate::physics::mirror::Mirror;

const KIND: u32 = 0x71A018; // g_carDriveKind
const K256: u32 = 0x5C5E2C; // flt_5C5E2C, 256.0

/// (out, setting, hi, lo) in the original's order, but the drive kind's.
const BEFORE: [(u32, u32, u32, u32); 4] = [
    (0x614C0C, 0x71A014, 0x613F28, 0x613F24),
    (0x614C08, 0x71A014, 0x613F30, 0x613F2C),
    (0x614BF0, 0x719FF8, 0x613F20, 0x613F1C),
    (0x614BF4, 0x71A008, 0x613F48, 0x613F44),
];
const AFTER: [(u32, u32, u32, u32); 4] = [
    (0x614BEC, 0x71A00C, 0x613F88, 0x613F84),
    (0x614BF8, 0x71A00C, 0x613F70, 0x613F6C),
    (0x614C00, 0x71A00C, 0x613F78, 0x613F74),
    (0x614C04, 0x71A00C, 0x613F80, 0x613F7C),
];

/// The Rust core on the variables `m` reaches (the exact one, for `race::settle_car` and
/// `physics::car_physics_step`).
pub unsafe fn set_handling_at(m: &Mirror) -> Result<(), Refused> {
    unsafe {
        let kind = m.get(KIND);
        if kind > 1 {
            return Err(BadInput("a drive kind above 1 (the pair tables hold two)").into());
        }
        grade_or_reset_at(m);
        let k256 = m.get(K256);
        let lerp = |(out, n, hi, lo): (u32, u32, u32, u32)| {
            m.put(out, int_lerp(m.get(n) as i32, k256, m.get(hi), m.get(lo)));
        };
        for v in BEFORE {
            lerp(v);
        }
        lerp((0x614BFC, 0x71A008, 0x613F54 + 4 * kind, 0x613F4C + 4 * kind));
        m.put(0x614BFC, x87::mul(m.get(0x614BFC), m.get(0x614C18)));
        for v in AFTER {
            lerp(v);
        }
        Ok(())
    }
}

/// The Rust version (the exact one).
pub unsafe fn set_handling() -> Result<(), Refused> {
    unsafe { set_handling_at(&Mirror::game()) }
}
