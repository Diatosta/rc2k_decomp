//! `marks::make_wave_tables` (0x4C4E2F): `init_marks`' step 2, once: the 0x11 wave
//! levels made floats and the 0x140 sines.
//!
//! In:  nothing
//! Out: eax `g_marksWaveDone`'s address, edi past the sines; done before: every register
//!      preserved
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Refuses a mark that is neither 0
//! nor `nullsub_23`'s address (the original jumps to it). Self-test: the original and the
//! Rust version on the game's tables with levels of the test's, the mark 0 and set; the
//! refusal; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `make_wave_tables_plain`, through
//! `rc2k_core::marks::waves` (plain floats; the x87 version stays for `marks::init_marks`).
//! Shadow mode and the self-test compare within `service::floats`' tolerances: the levels
//! exact at double precision, the sines within `FLOAT` at both (`fsin` is not rounded by
//! the precision control).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_wave_tables_plain(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
