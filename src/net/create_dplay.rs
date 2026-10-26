//! `net::create_dplay` (0x41861D): "NET: Create dplay", and DirectPlay's object made
//! (`CoCreateInstance` of CLSID_DirectPlay `stru_5AB330`, IID_IDirectPlay4A `stru_5AB310`,
//! in-process) into `g_dplay`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed, else the flags of
//!      `test eax, eax` on its result
//! Caller: `net::find_connections`. Its calls through `part` (the log; `CoCreateInstance`
//! from Rust, `net::dplay::create_instance`: the platform). Shadow mode runs the original
//! alone (it makes an object). Self-test: a copy whose calls go to stand-ins, against the
//! Rust version: made, failing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{CO_CREATE, DPLAY, LOG_L2_NL, Part, a, lobby_entry, log_text, test_eax};

const TEXT: usize = 0x5C2E3A; // "NET: Create dplay"
pub const CLSID_DIRECTPLAY: usize = 0x5AB330; // stru_5AB330
pub const IID_DIRECTPLAY4A: usize = 0x5AB310; // stru_5AB310

/// `CoCreateInstance` through `part` (eax the class, ebx the interface, ecx where): its
/// result in eax, the other registers as they were.
pub fn create(
    r: &mut Regs,
    class: u32,
    interface: u32,
    out: u32,
    part: Part,
) -> Result<(), Refused> {
    let mut c = Regs {
        eax: class,
        ebx: interface,
        ecx: out,
        ..*r
    };
    part(CO_CREATE, &mut c)?;
    r.eax = c.eax;
    Ok(())
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn create_dplay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    create(
        &mut r,
        a(CLSID_DIRECTPLAY),
        a(IID_DIRECTPLAY4A),
        a(DPLAY),
        part,
    )?;
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, create_dplay)
}
