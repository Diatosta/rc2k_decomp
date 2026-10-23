//! `net::find_saved_connection` (0x4185DF): the connection listed whose service provider's
//! GUID (record +0x40) is the one saved (`unk_52AC94`, `system::same_guid`).
//!
//! In:  nothing
//! Out: eax = its number, carry clear (the flags of `test eax, eax`); none (or none listed):
//!      eax = -1, carry set; ebx, ecx kept; edx as the calls leave it
//! Callers: `net::start_link`, `net::settings_done`. Its calls through `part`. Shadow mode
//! runs the original alone. Self-test: a copy whose calls go to stand-ins (the records
//! given, the second the same), against the Rust version: none listed, found, not found.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::lobby::{
    CONNECTIONS, Part, SAME_GUID, SAVED_CONNECTION, a, g, lobby_entry, test_eax,
};
use rc2k_core::net::lists;

const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry

/// The Rust version: the registers it leaves.
pub fn find_saved_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::lists::find_listed`.
    r.ebx = a(SAVED_CONNECTION);
    let found = lists::find_listed(g(CONNECTIONS), |n| -> Result<bool, Refused> {
        r.ecx = n;
        r.eax = n;
        part(CONNECTION_ENTRY, &mut r)?;
        r.eax = r.eax.wrapping_add(0x40);
        part(SAME_GUID, &mut r)?;
        Ok(r.eflags & ZERO_FLAG != 0)
    })?;
    if let Some(n) = found {
        r.eax = n;
        test_eax(&mut r);
        return Ok(Regs {
            ebx: entry.ebx,
            ecx: entry.ecx,
            ..r
        });
    }
    // The flags of the compare that came out equal (the count with 0, or the number with
    // the count), and the carry set.
    r.eax = u32::MAX;
    r.set_zero(true);
    crate::net::lobby::set_sign(&mut r, 0);
    r.set_carry(true);
    Ok(Regs {
        ebx: entry.ebx,
        ecx: entry.ecx,
        ..r
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, find_saved_connection)
}
