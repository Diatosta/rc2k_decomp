//! `race::car_label` (0x45B1B7): seven bytes at edi, " CAR nn" (nn eax's two digits: eax
//! / 10 and eax % 10, each + '0' as a byte), or seven spaces when ecx is -1.
//!
//! In:  edi = where, eax = the number, ecx = -1 for none
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45AD36` (0x45ADB0, 0x45AE0A), the panel's
//! texts; a `mov` follows both. Shadow mode: by state (the seven bytes). Self-test: a test
//! buffer filled with 0xAA, numbers 0, 7, 42, 99, 100, 255, 0xFFFFFFFE, and ecx -1.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { car_label(r.edi, r.eax, r.ecx) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
