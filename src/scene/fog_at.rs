//! `scene::fog_at` (0x44E260): the fog's alpha at a depth: eax (0 when below 0) >> 13,
//! held below `g_fogTableLength` (the last entry past it), its byte of `g_fogTable` in the
//! top byte, flipped (`xor 0xFF000000`).
//!
//! In:  eax the depth (whole)
//! Out: eax the alpha in its top byte; the rest kept
//!
//! Callers: the marks' drawers (16). The callers go on with a `mov`: no flags read.
//! Refuses a table length of 0 or past the table's 0x1000 bytes (the original reads past
//! it). Its first five bytes hold a short jump: private copy, manual patch. Self-test: the original and the Rust version, depths by lot and at the edges.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, rva};

const TABLE_RVA: usize = rva(0x63661C); // g_fogTable: 0x1000 bytes
const LENGTH_RVA: usize = rva(0x63761C); // g_fogTableLength

/// The Rust version (see the header).
pub unsafe fn fog_at(depth: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::scene::fog::fog_at`.
    Ok(rc2k_core::scene::fog::fog_at(
        depth,
        unsafe { game_read(LENGTH_RVA) },
        |n| unsafe { game_read(TABLE_RVA + n) },
    )?)
}
