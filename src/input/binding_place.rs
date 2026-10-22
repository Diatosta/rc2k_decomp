//! `input::binding_place` (0x408DD6): a binding's word as where the record reads it: for an
//! acquired device (+0x24 1), the axis (bit 0x400: +0x2C negated, 0x800: +0x2C, 0x200:
//! +0x34, 0x100: +0x30, else none) and the button (the low byte n: bit n - 1; 0 none);
//! else none of them.
//!
//! In:  eax = the binding's word, esi = the device's record
//! Out: eax = the axis's place in the record (0 none), edx = its sign (1, -1; 0 when not
//!      acquired), ebx = the button's bit (0 none), ecx = the low byte (less one when not
//!      0) when acquired; no flags read
//!
//! Caller: `sub_408BCD` (twelve times, a binding each). Self-test: the original and the
//! Rust version on words of each kind and buttons 0 .. 0x20 and past, acquired or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, Regs};
use crate::input::devices::at;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub fn binding_place(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::input::devices::binding_place`.
    let place = rc2k_core::input::devices::binding_place(entry.eax, at(entry.esi, 0x24) == 1);
    Regs {
        eax: place.axis,
        edx: place.sign,
        ebx: place.button,
        ecx: place.ecx.unwrap_or(entry.ecx),
        ..*entry
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let o = binding_place(r);
    Ok(Left::nothing()
        .with(EAX, o.eax)
        .with(EBX, o.ebx)
        .with(ECX, o.ecx)
        .with(EDX, o.edx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
