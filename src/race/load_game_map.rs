//! `race::load_game_map` (0x41B480): the in-race map (`LOADLEVEL_INIT : loadgamemap`).
//!
//! In:  eax = the map picture's path (`srcmaps\mapxx.pcx`), ebx = the key's
//!      (`srcmaps\keyxx.pcx`)
//! Out: with the packed file read, eax = `g_packedPath`; every other register preserved
//!
//! See `race/game_map.rs`. One caller, `sub_44426F` (loadlevel_init). A key not 8-bit is
//! fatal error 0x80E48073, raised from Rust. Shadow mode runs the original alone (it
//! allocates and reads). Self-test: a redirected copy whose `file_exists`, `read_sheet`
//! and `read_picture_file` are stand-ins (the sheet and picture of the test's, written
//! into the picture buffer; the packed case reads stage 21's real map), against the Rust
//! version with the same; compared with the pointers taken out, freed after each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::game_map::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { load_game_map(r, &mut game_part) }?;
        r.eax = g.eax
    }
    Ok(())
}
