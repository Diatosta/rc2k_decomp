//! `physics::car_angles` (0x43BEF4): `physics::matrix_angles` of `g_crashMatrix`, then
//! each angle times 65536 (`flt_5C5D98`), 4096 (`flt_5C5E28`) and 1 / 2pi (`flt_5C5DD4`),
//! `fistp`, shifted left 4: `flt_5C70D4` into `dword_71A99C`, `D8` into `dword_71A998`,
//! `DC` into `dword_71A9A0` (`g_car` +0x9C4, +0x9C0, +0x9C8): the car's angles in the
//! game's whole-number units (a reading).
//!
//! In:  nothing
//! Out: esi = `g_crashMatrix`; the rest kept; no flags read
//!
//! Caller: `physics::update_car_angles`.
//! Self-test: a copy of the original (the matrix, angles and ints the test's, its call to
//! `matrix_angles`' test copy) on matrices by lot, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ESI, Regs, game_addr, game_read, rva};
use crate::math::x87_run::mul3_to_int;
use crate::physics::matrix_angles::{AnglePlaces, matrix_angles_at};
use crate::race::stage_prefs::CRASH_MATRIX_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

const INTS_RVA: usize = rva(0x71A998); // dword_71A998 (from D8), 99C (from D4), 9A0 (from DC)

/// The Rust core: the matrix at `m`, `matrix_angles`' variables at `p`, the three ints at
/// `ints` (+4 from the first angle, +0 from the second, +8 from the third).
pub unsafe fn car_angles_at(m: u32, p: &AnglePlaces, ints: u32) {
    unsafe {
        matrix_angles_at(m, p);
        let scale = |a: u32| {
            mul3_to_int(
                a,
                game_read::<u32>(rva(0x5C5D98)),
                game_read::<u32>(rva(0x5C5E28)),
                game_read::<u32>(rva(0x5C5DD4)),
            )
            .wrapping_shl(4)
        };
        let angle = |k: u32| (p.angles as *const u32).add(k as usize).read_unaligned();
        for (k, place) in [(0u32, 4u32), (1, 0), (2, 8)] {
            let v = scale(angle(k));
            ((ints + place) as *mut i32).write_unaligned(v);
        }
    }
}

/// The Rust version.
pub unsafe fn car_angles() -> Left {
    let m = game_addr(CRASH_MATRIX_RVA) as u32;
    unsafe { car_angles_at(m, &AnglePlaces::game(), game_addr(INTS_RVA) as u32) };
    Left::nothing().with(ESI, m)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { car_angles() })
}

/// What the function writes.
pub fn regions() -> Vec<Region> {
    let mut r = AnglePlaces::game().regions();
    r.push((game_addr(INTS_RVA) as u32, 12));
    r
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
