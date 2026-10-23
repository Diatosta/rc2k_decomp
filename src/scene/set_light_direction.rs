//! `scene::set_light_direction` (0x427326): the direction of the light in the view.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The turning matrices about x and y for the light's two angles (`dword_635098` and
//! `dword_635094`, each times 256 and negated); the first times the second to
//! `flt_5EA10C`; that times the view's turning `dword_5EA24C` to `flt_5EA14C`; and the
//! point (0, 0, 1) through that (`scene::rotate_point`, inlined) is `g_lightDirection`.
//! Three callers.
//! Self-test: 40 settings by lot on the game's matrices, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_light_direction_plain`, through
//! `rc2k_core::scene::matrices::set_light_direction` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::matrix::{self};
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { matrix::set_light_direction_plain() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
