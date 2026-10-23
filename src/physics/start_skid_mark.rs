//! `physics::start_skid_mark` (0x43F272): when either of the two qwords at
//! `qword_71A5D8`, `qword_71A5E0` is not 0 and `dword_71A664` is: `dword_71A660` = it less
//! 1, `dword_71A66C` 1, `dword_71A674` 0, `dword_71A670` 10, `dword_71A668` the step count
//! `dword_719FE0`; then `dword_71A664` 0. By the look of it a mark's start (a reading).
//!
//! In:  nothing
//! Out: eax the step count (`dword_71A664` not 0), 0 (it 0), kept (both qwords 0); no
//!      flags read
//!
//! Caller: `sub_43B635` (0x43B6FB), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 12 cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, rva};
use crate::physics::cells::{get, put_changed};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::ground::SkidStart;

const VARIABLES: [usize; 11] = [
    0x71A5D8, 0x71A5DC, 0x71A5E0, 0x71A5E4, 0x71A664, 0x71A660, 0x71A66C, 0x71A674, 0x71A670,
    0x719FE0, 0x71A668,
];

/// The Rust version on the variables at `v`; `r` as the original leaves it.
// Converted: `rc2k_core::physics::ground::SkidStart::start`.
pub unsafe fn start_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let wheels = [get(v[0]), get(v[1]), get(v[2]), get(v[3])];
        let was = SkidStart {
            pending: get(v[4]),
            from: get(v[5]),
            on: get(v[6]),
            w674: get(v[7]),
            w670: get(v[8]),
            step: get(v[10]),
        };
        let mut s = was;
        if let Some(eax) = s.start(wheels, || get(v[9])) {
            r.eax = eax;
        }
        for (k, before, now) in [
            (5, was.from, s.from),
            (6, was.on, s.on),
            (7, was.w674, s.w674),
            (8, was.w670, s.w670),
            (10, was.step, s.step),
            (4, was.pending, s.pending),
        ] {
            put_changed(v[k], before, now);
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version; `r` as the original leaves it.
pub unsafe fn start_skid_mark(r: &mut Regs) {
    unsafe { start_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { start_skid_mark(&mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
