//! `net::connection_entry` (0x418530): the record of connection eax (0x54 bytes each in the
//! table `dword_52AC38`, `dword_52AC34` of them): +0 its name (0x40 bytes), +0x40 its
//! service provider's GUID, +0x50 the connection's DirectPlay address.
//!
//! In:  eax = the connection's number
//! Out: eax = its record; the flags of the `add` (no caller reads them)
//! A number not below the count (unsigned) is fatal error 0x80A61010, no table fatal error
//! 0x80A61011. Callers: `net::keep_connection`, `net::find_saved_connection`,
//! `net::connection_found`, `net::build_address`, `net::connections_frame`. Shadow mode: both
//! run, eax compared. Self-test: a copy whose jumps to the fatal stubs land in the test,
//! against the Rust version, on numbers in and past the list and no table.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest};
use crate::net::lobby::{Part, Saved, connection, dword, fatal_ret_1, fatal_ret_2, p, run_both};

/// The Rust version: the registers it leaves.
pub fn connection_entry(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::net::lists::record` (in `net::lobby::record_of`).
    Ok(Regs {
        eax: connection(entry.eax)?,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        connection_entry(r, &mut crate::net::lobby::game_part)
    })
}

/// The test of the two record functions: a copy, its fatal jumps (at +6 and +0x16) to the
/// landings.
pub fn entry_test(
    hook: &RawHook,
    address: usize,
    entry: [u8; 6],
    count: usize,
    table: usize,
    codes: [u32; 2],
    rust: fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match crate::net::lobby::copy(address, 0x23, &entry, &[], &[]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    unsafe {
        copy.retarget(0x6 + 2, fatal_ret_1 as *const () as usize);
        copy.retarget(0x16 + 2, fatal_ret_2 as *const () as usize);
    }
    let regions = [dword(count), dword(table)];
    let saved = Saved::new(&regions);
    for (index, records, at) in [
        (0u32, 3u32, 0x0123_4560u32),
        (2, 3, 0x0123_4560),
        (3, 3, 0x0123_4560),
        (0xFFFF_FFFF, 3, 0x0123_4560),
        (0, 0, 0x0123_4560),
        (1, 5, 0),
        (9, 5, 0),
    ] {
        run_both(
            &mut t,
            &format!("record {index:#x} of {records}, table {at:#x}"),
            copy.addr(),
            Regs {
                eax: index,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &codes,
            &|| {
                p(count, records);
                p(table, at);
            },
            &rust,
        );
    }
    drop(saved);
    t.finish()
}
