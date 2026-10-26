//! `net::free_messages` (0x4185BB): the network's memory given back (`memory::free_list`
//! on `stru_52A928`) and four of its counts (`dword_52AC34`, `dword_52AC38`,
//! `dword_52ACF4`, `dword_52ACF8`) made 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `net/mod.rs`. One caller, `net::destroy_connection`. Its call through `part` (the
//! Rust `free_list`, which refuses the game's own heap before anything). Shadow mode runs
//! the original alone. Self-test: a copy whose list and counts are the test's and whose
//! call goes to a noting stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{FREE_LIST, Part, a, net_entry};

const LIST: usize = 0x52A928; // stru_52A928
const COUNTS: [usize; 4] = [0x52AC34, 0x52AC38, 0x52ACF4, 0x52ACF8];

/// The Rust version on the list at `list` and the counts at `counts`.
pub fn free_messages_with(
    r: &mut Regs,
    list: u32,
    counts: [u32; 4],
    part: Part,
) -> Result<(), Refused> {
    let mut g = Regs { ebx: list, ..*r };
    part(FREE_LIST, &mut g)?;
    for at in counts {
        unsafe { (at as *mut u32).write_volatile(0) };
    }
    Ok(())
}

/// The Rust version.
pub fn free_messages(r: &mut Regs, part: Part) -> Result<(), Refused> {
    free_messages_with(r, a(LIST), COUNTS.map(a), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, free_messages)
}
