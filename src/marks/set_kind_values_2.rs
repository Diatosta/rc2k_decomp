//! `marks::set_kind_values_2` (0x4C7FA4): `init_marks`' step 9, the marks' kind's
//! values of set 2.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the kind's copied three, ebp the kind times 12
//!
//! See `marks/kind_values.rs`. One caller, `marks::init_marks`. Refuses a kind past the
//! tables (0x22). Self-test: the original and the Rust version on the game's tables
//! filled with values of the test's, kinds 0 .. 0x21; the refusal; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_kind_values_plain`, through
//! `rc2k_core::marks::kind_values::values` (plain floats; the x87 version stays for
//! `marks::init_marks`). Shadow mode and the self-test compare within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::kind_values::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { set_kind_values_plain(r, &SECOND) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
