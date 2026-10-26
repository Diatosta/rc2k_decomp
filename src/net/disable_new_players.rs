//! `net::disable_new_players` (0x418D8A): on the host (state bit 0x10 of `byte_52AD64`), "NET:
//! Disable new players", and with DirectPlay's object there the session's description
//! (`dword_5AB2AC`) given DPSESSION_NEWPLAYERSDISABLED and DPSESSION_JOINDISABLED (0x21) and
//! set (`SetSessionDesc`, flags 0); its result logged.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if not the host, no object or it
//!      failed, else the flags of `test eax, eax`
//! Caller: `net::close_if_host`. Its calls through `part`; `SetSessionDesc` from Rust
//! (`net::dplay`). Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins and whose DirectPlay is a test object, against the Rust version: not the host,
//! no object, failing, done.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{SET_SESSION_DESC, call};
use crate::net::lobby::{
    DPLAY, LOG_HEX_L2_NL, LOG_L2, OWN, Part, SESSION_DESC, a, cmp_flags, g, gb, lobby_entry,
    log_text, p, set_sign, test_eax,
};

const TEXT: usize = 0x5C2F26; // "NET: Disable new players"
const CLOSED: u32 = 0x21; // DPSESSION_NEWPLAYERSDISABLED | DPSESSION_JOINDISABLED

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn disable_new_players(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    if gb(OWN) & 0x10 == 0 {
        // The flags of `test byte_52AD64, 10h`, and the carry.
        r.set_zero(true);
        set_sign(&mut r, 0);
        r.set_carry(true);
        return Ok(left(&r));
    }
    log_text(LOG_L2, a(TEXT), &r, part)?;
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    p(SESSION_DESC + 4, g(SESSION_DESC + 4) | CLOSED);
    r.eax = unsafe { call(DPLAY, SET_SESSION_DESC, &[a(SESSION_DESC), 0]) };
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
    lobby_entry(r, disable_new_players)
}
