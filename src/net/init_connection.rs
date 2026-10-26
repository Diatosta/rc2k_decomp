//! `net::init_connection` (0x418794): "NET: Init connection index" and the number logged;
//! with DirectPlay's object there, connection eax's address built (`net::build_address`)
//! and DirectPlay set to it (`InitializeConnection`, `dword_52AC40`, flags 0); "NET: Init"
//! and its result logged.
//!
//! In:  eax = the connection's number
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed, else the flags of
//!      `test eax, eax`
//! Caller: `net::connecting`. Its calls through `part`; `InitializeConnection` from Rust
//! (`net::dplay`). Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins and whose DirectPlay is a test object, against the Rust version: no object,
//! no address, failing, done.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{INITIALIZE_CONNECTION, call};
use crate::net::lobby::{
    ADDRESS, DPLAY, LOG_HEX_L2_NL, LOG_INT_L2_NL, LOG_L2, Part, a, cmp_flags, g, lobby_entry,
    log_text, test_eax,
};

const TEXT: usize = 0x5C2E80; // "NET: Init connection index"
const INIT: usize = 0x5C2E9B; // "NET: Init"
const BUILD_ADDRESS: u32 = 0x41882C; // net::build_address

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn init_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2, a(TEXT), &r, part)?;
    let mut l = r;
    part(LOG_INT_L2_NL, &mut l)?;
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    part(BUILD_ADDRESS, &mut r)?;
    if r.carry() {
        return Ok(left(&r));
    }
    r.eax = unsafe { call(DPLAY, INITIALIZE_CONNECTION, &[g(ADDRESS), 0]) };
    log_text(LOG_L2, a(INIT), &r, part)?;
    let mut l = r;
    part(LOG_HEX_L2_NL, &mut l)?;
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(left(&r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, init_connection)
}
