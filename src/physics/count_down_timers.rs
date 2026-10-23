//! `physics::count_down_timers` (0x43D4B5): the work car's step counters:
//! `dword_71AF8C`, `dword_71AF88`, `dword_71A1CC`, `dword_71A1A8`, `dword_71A438` less 1
//! when not 0; `dword_71A240` counts the steps standing in neutral in mode 1 (0 below 1000
//! steps, with a switch of 0x71A078 .. 0x71A080 set, with bits 1 .. 4 of `dword_71A270` all
//! set, or in gear; kept in other modes); `dword_71A244` on by one unless 0, -1 or 1;
//! `flt_71A5D4` raised to `flt_71AEE0` (the highest so far). Readings, all.
//!
//! In:  nothing
//! Out: eax `dword_71A270` & 0x1E when that is looked at, else kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B70A) and `sub_421848` (the car specification's run).
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables against the
//! converted version), 48 cases; one compare and copy of floats: exact at any precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::motion::{StepCounters, StepFacts, count_down};

const VARIABLES: [usize; 16] = [
    0x71AF8C, 0x71AF88, 0x719FEC, 0x71A080, 0x71A07C, 0x71A078, 0x62389C, 0x71A270, 0x71A1AC,
    0x71A240, 0x71A244, 0x71A1CC, 0x71A1A8, 0x71AEE0, 0x71A5D4, 0x71A438,
];

/// The converted version (`rc2k_core::physics::motion`) on the variables at `v`; `r` as
/// the original leaves it.
// Converted: `rc2k_core::physics::motion::count_down`.
pub unsafe fn count_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let mut c = StepCounters {
            first: [get(0), get(1)],
            standing: get(9),
            running: get(10),
            later: [get(11), get(12), get(15)],
            highest: f32::from_bits(get(14)),
        };
        let facts = StepFacts {
            steps: get(2),
            switches: [get(3), get(4), get(5)],
            mode: get(6),
            bits: get(7),
            gear: get(8),
            figure: f32::from_bits(get(13)),
        };
        if let Some(eax) = count_down(&mut c, &facts) {
            r.eax = eax;
        }
        for (k, x) in [
            (0, c.first[0]),
            (1, c.first[1]),
            (9, c.standing),
            (10, c.running),
            (11, c.later[0]),
            (12, c.later[1]),
            (14, c.highest.to_bits()),
            (15, c.later[2]),
        ] {
            if get(k) != x {
                put(k, x);
            }
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (one compare and copy of floats: the same at any precision); `r` as
/// the original leaves it.
pub unsafe fn count_down_timers(r: &mut Regs) {
    unsafe { count_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { count_at(&game(), &mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
