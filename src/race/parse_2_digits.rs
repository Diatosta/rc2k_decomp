//! `race::parse_2_digits` (0x437A25): the two decimal digits at `byte_6047B8` + ecx (the
//! cars' data file's text) as a number.
//!
//! In:  ecx = the place in the text
//! Out: carry clear, eax = the number; carry set, eax = 0 with any not a digit (`movzx`
//!      and `sub` of the low byte: a byte below '0' wraps within the byte); the rest
//!      preserved (ebx pushed and popped)
//!
//! See `race/race_cars.rs`. One caller, `sub_437971` (the cars' data). The failure path
//! (0x437A4B) is `parse_2_digits`' own end, which `parse_3_digits` jumps to. Shadow
//! mode: the Rust version and the original, registers and carry compared (nothing
//! written). Self-test: texts of the test's in the game's buffer (put back), every place
//! of a run of digits and letters, the bytes round '0' and '9', the last places.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(match unsafe { parse_2_digits(r.ecx) } {
        Some(n) => Left::carry(false).with(EAX, n),
        None => Left::carry(true).with(EAX, 0),
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
