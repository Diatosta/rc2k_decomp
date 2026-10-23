//! `race::add_depth_entry` (0x454D2A): `race::face_depth` of face ebx, then entry
//! `dword_63DC4C` of `dword_9D3730` (0x1000 of 8 bytes): +0 the key, +4 ebx; the count + 1.
//!
//! In:  ebx = the face's number
//! Out: edx the entry, ecx the key, ebx & 0x7FFFFFFF, CF clear (`clc`; its callers in
//!      `sub_454C99` jump on: not read)
//!
//! See `race/frame_draw.rs`. It begins with its call: private copy (0x2D, the call
//! relocated), manual patch. Refuses (before anything) an entry past the 0x1000, or what
//! `face_depth` refuses. Shadow mode: by state. Self-test: a test face (`g_courseFaces`
//! pointed at it) on the game's points, counts 0, 7, 0xFFF; all put back (skipped without
//! points); 0x1000 refused.
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
    unsafe { add_depth_entry(r.ebx) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
