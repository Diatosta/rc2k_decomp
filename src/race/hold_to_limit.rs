//! `race::hold_to_limit` (0x4650A9): eax held to `dword_71C6D4` (unsigned).
//!
//! In:  eax
//! Out: eax; the rest kept (the callers go on with a `mov`: no flags read)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45497E`, code at 0x454BD4, `sub_45C7EE`.
//! Shadow mode: by state (registers). Self-test: values below, at and above limits;
//! `dword_71C6D4` put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_71C6D4

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, hold_to_limit(r.eax)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
