//! `marks::turn_kept_zy` (0x4C14D2): the three kept vectors turned: z and y by the first angle's sine and cosine (`marks/kept_turns.rs`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `marks::turn_kept_vectors`. Self-test: the inputs by lot, compared on what it writes; put back
//! afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `kept_turns::turn_kept_zy_plain`,
//! through `rc2k_core::marks::view::KeptVectors::turn_zy` (plain floats; the x87 version
//! stays for the Rust callers in unconverted code). Shadow mode and the self-test compare
//! within `service::floats`' tolerances (the self-test at both test precisions, exact at
//! double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::kept_turns::turn_kept_zy_plain;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { turn_kept_zy_plain() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
