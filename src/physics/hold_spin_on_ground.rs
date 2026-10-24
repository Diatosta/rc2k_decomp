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
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables against the
//! converted version), 24 cases; compares and copies of floats only: exact at any
//! precision.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::framework::{game_addr, rva};
use rc2k_core::physics::motion::hold_spin;

/// The four wheels, the limit, x, z, the flag.
const VARIABLES: [usize; 8] = [
    0x71A5D8, 0x71A5DC, 0x71A5E0, 0x71A5E4, 0x613EA0, 0x71A15C, 0x71A164, 0x614C24,
];

/// The converted version (`rc2k_core::physics::motion`) on the variables at `v`. The
/// registers it leaves (ecx the wheels on the ground, ebx 4) are read by none of its
/// callers.
// Converted: `rc2k_core::physics::motion::hold_spin`.
pub unsafe fn hold_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
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
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (compares and copies of floats only: the same at any precision); `r`
/// the registers it leaves are read by none of its callers.
pub unsafe fn hold_spin_on_ground() {
    unsafe { hold_at(&game()) }
}
