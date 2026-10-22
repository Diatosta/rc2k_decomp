//! `input::yes_no_for_key` (0x407E27): `yes_no_key` for a key the caller already has.
//!
//! In:  eax = the key
//! Out: eax = 1 for yes, 0xFFFFFFFF for no, 2 for the third answer, else 0; everything else
//!      preserved
//! Sets `dword_4F7FD8` to 1 around the call and puts back what it held. One caller,
//! `menu::handle_yes_no`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::input::key_to_char::has_layout;
use crate::input::yes_no_key::answer_for_key;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub unsafe fn yes_no_for_key(eax: u32) -> Result<u32, crate::errors::BadInput> {
    if !has_layout() {
        return Err(crate::errors::BadInput("no keyboard layout"));
    }
    unsafe { answer_for_key(eax) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { yes_no_for_key(r.eax) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
