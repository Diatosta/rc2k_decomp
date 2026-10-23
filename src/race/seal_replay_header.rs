//! `race::seal_replay_header` (0x41F0A3): the replay header's sealed part
//! (`dword_719B54`, 0x100 bytes): version 6, the player's name, the settings, a checksum
//! (also `dword_4F66F9`), then its 32 blocks chained through `race::seal_block` with the
//! key "halfmoon" (the game's key put back).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/replay.rs`. One caller, `race::open_replay`. Shadow mode: the original
//! alone (its result is the seal's, the cipher state changes). Self-test: a copy whose two
//! calls (the name, the seal) go to stand-ins, against the Rust version on the same, with
//! the part's dwords filled two ways.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { seal_replay_header(r, &mut game_part) }?;
        *r = Regs { esp: r.esp, ..g }
    }
    Ok(())
}
