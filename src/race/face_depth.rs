//! `race::face_depth` (0x4552D8): a course face's depth key into `dword_63DC40`.
//!
//! In:  ebx = the face's number (records of 0x40 bytes at `g_courseFaces`)
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! A kind (+0) of 0x10 or more: its first point's depth (`fistp`; `dword_63DC44` its x);
//! else, with its object's (+1) record +0xE bit 1, the largest of its four points' depths
//! (each `fistp`, through `dword_62400C`); otherwise the four depths summed as floats,
//! `fistp`'d and shifted right 2 (arithmetic). Then + 0x2000000 and shifted right 10. The
//! points' scene records are made when not there (as `sub_44CF33`). See
//! `race/frame_draw.rs`. Caller: `race::add_depth_entry`. Refuses (before anything) no point
//! records or points. Shadow mode: by state (the two dwords, the four records,
//! `dword_62400C`). Self-test: test faces (`g_courseFaces` pointed at them) on the game's
//! points 0 .. 7, each kind of face, the object flag on and off, records cleared so they
//! are made; all put back (skipped without points).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// g_courseFaces

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { face_depth(r.ebx) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
