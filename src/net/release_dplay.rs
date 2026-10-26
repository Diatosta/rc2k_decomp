//! `net::release_dplay` (0x41864E): with a DirectPlay object (`g_dplay` not 0), "NET:
//! Destroy dplay" logged (level 2, a line), the object released (`IDirectPlay4A` +8) and
//! `g_dplay` made 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); without an object the flags of
//!      `cmp g_dplay, 0`, else those `Release` left (not modelled: the Rust version leaves
//!      the compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. The log through `part`, `Release` from Rust
//! (`net::dplay`, section 15). Refuses nothing. Shadow mode runs the original alone.
//! Self-test: a copy whose variable is the test's, whose log goes to a noting stand-in and
//! whose DirectPlay is a test object noting its arguments, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{RELEASE, method};
use crate::net::lobby::{cmp_flags, log_text};
use crate::net::message_vars::{MessageVars, get, set};
use crate::net::{LOG_TEXT_2_NL, Part, a, net_entry};

const TEXT: usize = 0x5C2E4C; // "NET: Destroy dplay"

/// The Rust version on the variables at `v`.
pub fn release_dplay_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let object = get(v.dplay);
    cmp_flags(r, object, 0);
    if object != 0 {
        log_text(LOG_TEXT_2_NL, a(TEXT), r, part)?;
        unsafe { method(get(v.dplay), RELEASE, &[]) };
        set(v.dplay, 0);
    }
    Ok(())
}

/// The Rust version.
pub fn release_dplay(r: &mut Regs, part: Part) -> Result<(), Refused> {
    release_dplay_with(r, &MessageVars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, release_dplay)
}
