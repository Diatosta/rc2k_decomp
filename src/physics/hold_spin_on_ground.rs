//! `physics::hold_spin_on_ground` (0x43BAA4): with all four wheels down (the four dwords
//! from `qword_71A5D8` not 0), the spin's x and z (`dword_71A15C`, `dword_71A164`) held
//! to -`flt_613EA0` .. `flt_613EA0` (by negating that limit in place, twice for each, so
//! it comes back), and `dword_614C24` = whether either now has the limit's size (C3: also
//! when no number). A reading: no tumbling on the ground.
//!
//! In:  nothing
//! Out: ecx the wheels down, ebx 4; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B986), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::session::Cell32;
use rc2k_core::physics::motion::hold_spin;

/// The converted version (`rc2k_core::physics::motion`) on the variables' cells `v`. The
/// registers it leaves (ecx the wheels on the ground, ebx 4) are read by none of its
/// callers.
// Converted: `rc2k_core::physics::motion::hold_spin`.
pub fn hold_at(v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    if (0..4).any(|k| get(k) == 0) {
        return;
    }
    let f = |k: usize| f32::from_bits(get(k));
    let (mut limit, mut x, mut z) = (f(4), f(5), f(6));
    let flag = hold_spin(&mut limit, &mut x, &mut z);
    put(4, limit.to_bits());
    put(5, x.to_bits());
    put(6, z.to_bits());
    put(7, flag);
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &crate::race::owner::race().shared.car.wheel_squeeze[0] as &dyn Cell32,
        &crate::race::owner::race().shared.car.wheel_squeeze[1],
        &crate::race::owner::race().shared.car.wheel_squeeze[2],
        &crate::race::owner::race().shared.car.wheel_squeeze[3],
        &crate::race::owner::race()
            .shared
            .physics_tables
            .ground_spin_limit,
        &crate::race::owner::race().shared.car.spin[0],
        &crate::race::owner::race().shared.car.spin[2],
        &crate::race::owner::race().shared.physics_tables.ground_spin,
    ]
}

/// The Rust version (compares and copies of floats only: the same at any precision); `r`
/// the registers it leaves are read by none of its callers.
pub fn hold_spin_on_ground() {
    hold_at(&game());
}
