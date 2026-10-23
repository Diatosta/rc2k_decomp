//! `marks::turn_kept_vectors` (0x4C13CA): the three kept vectors turned: the sines and cosines of `flt_76923E` and `flt_76924A`, then
//! `marks::turn_kept_xz` and `marks::turn_kept_zy` (`marks/kept_turns.rs`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`. Self-test: the inputs by lot, compared on what it writes; put back
//! afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `kept_turns::turn_kept_vectors_plain`,
//! through `rc2k_core::marks::view::turn_kept_vectors` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::kept_turns::{FIRST_RVA, SECOND_RVA, turn_kept_vectors_plain, vector_dwords};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at};

fn written() -> Vec<usize> {
    let mut written = vector_dwords();
    written.extend(FIRST_RVA);
    written.extend(SECOND_RVA);
    written
}

/// What it writes.
pub fn regions() -> Vec<Region> {
    written().into_iter().map(at).collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { turn_kept_vectors_plain() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
