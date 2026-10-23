//! `scene::show_bad_polygon` (0x443AEC): edi `g_polygonVertices` and
//! `scene::log_bad_polygon`: the polygon in the buffer dumped to the log.
//!
//! In:  nothing
//! Out: edi = `g_polygonVertices`; the rest kept
//!
//! Caller: `scene::submit_polygon` (a vertex off the screen; its Rust version calls this
//! one's directly). The dump's log calls through `part` as `log_bad_polygon`'s. Shadow mode
//! runs the original alone (it writes the log). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::menu::parts::{Part, parted_entry};

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// The Rust version: the registers it leaves. Its one call (0, the dump) through `part`.
pub unsafe fn show_bad_polygon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs {
        edi: game_addr(VERTICES_RVA) as u32,
        ..*entry
    };
    part(0, &mut r)?;
    Ok(r)
}

/// In the game: the dump's Rust version with the Rust log functions.
pub fn game_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::log_bad_polygon::{game_part as log_part, log_bad_polygon};
    *r = unsafe { log_bad_polygon(r, &mut log_part) }?;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(
        r,
        &|r, part| unsafe { show_bad_polygon(r, part) },
        game_part,
    )
}
