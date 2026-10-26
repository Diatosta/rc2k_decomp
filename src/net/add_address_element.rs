//! `net::add_address_element` (0x41896E): one more element of the compound address being
//! built (`unk_52AC48`, 0x18 bytes each, `dword_52AC3C` of them: a DPCOMPOUNDADDRESSELEMENT,
//! the data type's GUID, the data's size, the data).
//!
//! In:  eax = the data type's GUID, ecx = the data's size, ebx = the data
//! Out: edx = the element's offset (number * 0x18); the rest kept; the flags of the count's
//!      `inc` (no caller reads them)
//! No limit on the count. Caller: `net::build_address` (five times; three at most in one
//! go). Shadow mode: both run, the element and the count compared. Self-test: a copy
//! against the Rust version, on counts 0, 1 and 2.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{ELEMENT_TABLE, ELEMENTS, Part, a, g, p};
use rc2k_core::net::address;

/// The Rust version: the registers it leaves.
pub fn add_address_element(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::address` (`element_at`, `element`).
    r.edx = address::element_at(g(ELEMENTS));
    p(ELEMENTS, g(ELEMENTS).wrapping_add(1));
    let at = a(ELEMENT_TABLE).wrapping_add(r.edx);
    let data_type = unsafe { (r.eax as *const [u8; 16]).read_unaligned() };
    let bytes = address::element(&data_type, r.ecx, r.ebx);
    unsafe { std::ptr::copy(bytes.as_ptr(), at as *mut u8, bytes.len()) };
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        add_address_element(r, &mut crate::net::lobby::game_part)
    })
}
