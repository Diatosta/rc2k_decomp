//! `marks::smooth_steps` (0x4BE928): two cosine-eased steps from ebx to ecx (floats), step
//! eax, into ecx and ebx.
//!
//! In:  eax the step, ebx a, ecx b (floats)
//! Out: ecx, ebx the two steps; eax kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BD122` (twice), `sub_4BEB25` (four). Self-test:
//! the original and the Rust version on values and steps by lot; the two scratch
//! variables put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `turns::smooth_steps_plain`, through
//! `rc2k_core::marks::turns::smooth_pair` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { smooth_steps_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
