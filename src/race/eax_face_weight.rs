//! `race::eax_face_weight` (0x45C881): a course face's weight in the EAX sound environment
//! (`g_eaxEnvironment`, a float sum; `dword_6D1AF0` counts the faces), by its distance from
//! car 0 in x and z (see `race/frame_draw.rs`).
//!
//! In:  ebx = the face's number
//! Out: esi the face; eax, ebx, ecx as its path leaves them (its caller `sub_45C84C` calls
//!      it inside `pusha`)
//!
//! Kind 0xFF with an object below 0x38: its +0x18 byte over the distance to its first
//! point; kinds below 0x10 with |+0x30| below 0.25: 3.0 over the distance to the middle of
//! its second and third points; each held to 16.0. The distance is the game's table root
//! (`g_rootTable` by the square sum's mantissa, `g_exponentRootTable` by its exponent,
//! x 1/65536). All x87 as the original. Shadow mode: by state (the sum, the count and the
//! dwords it passes through). Self-test: test faces and points (`g_courseFaces`,
//! `g_scenePoints` pointed at them), car 0's place set, each kind and test; all put back
//! (skipped while the root tables are empty).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { eax_face_weight(r.ebx, r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
