//! `net::destroy_connection` (0x4187EE): with a DirectPlay object (`dword_5AB288`), the
//! network race ended (`dword_52A910` 0), "NET: Destroy connection" logged, and through
//! DirectPlay the own player destroyed (`sub_41916B`), the session closed (`sub_418DD3`),
//! the object released (`sub_41864E`) and the lobby's (`sub_41900F`); either way the
//! network's memory given back (`net::free_messages`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `test eax, eax` (not read)
//!
//! See `net/mod.rs`. Two callers: `net::stop` and `sub_41976F` (a message from DirectPlay,
//! 0x31). Its calls through `part`: DirectPlay's four through the game (section 15), the
//! log's and `free_messages` the Rust versions. Shadow mode runs the original alone.
//! Self-test: a copy whose two variables are the test's and whose calls go to noting
//! stand-ins, against the Rust version, with an object and without.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{
    CLOSE_SESSION, DESTROY_PLAYER, FREE, LOG_TEXT_2_NL, ON, Part, RELEASE_DPLAY, RELEASE_LOBBY, a,
    net_entry,
};

/// dword_5AB288: DirectPlay's object (IDirectPlay), 0 without one.
pub const DPLAY: usize = 0x5AB288;
const TEXT: usize = 0x5C2EA5; // "NET: Destroy connection"

/// The Rust version on the object's pointer at `dplay` and the switch at `on`.
pub fn destroy_connection_with(
    r: &mut Regs,
    dplay: u32,
    on: u32,
    part: Part,
) -> Result<(), Refused> {
    let mut g = *r;
    if unsafe { (dplay as *const u32).read_volatile() } != 0 {
        unsafe { (on as *mut u32).write_volatile(0) };
        let mut l = Regs { eax: a(TEXT), ..g };
        part(LOG_TEXT_2_NL, &mut l)?;
        for place in [DESTROY_PLAYER, CLOSE_SESSION, RELEASE_DPLAY, RELEASE_LOBBY] {
            g = *r;
            part(place, &mut g)?;
        }
    }
    g = *r;
    part(FREE, &mut g)?;
    r.set_carry(false);
    r.set_zero(r.eax == 0);
    Ok(())
}

/// The Rust version.
pub fn destroy_connection(r: &mut Regs, part: Part) -> Result<(), Refused> {
    destroy_connection_with(r, a(DPLAY), a(ON), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, destroy_connection)
}
