//! `race::face_materials` (0x45265C): face esi's two triangles' textures, animated by car
//! 0's clock for objects with a speed (`byte_609C4C`), and their kinds' bits (see
//! `race/course_faces.rs`).
//!
//! In:  esi = the face
//! Out: eax, ebx, ecx as it leaves them, CF clear (callers go on with `mov` / calls)
//!
//! Callers: `sub_452393` (three places). Shadow mode: by state. Self-test: test faces
//! (objects with and without a speed, kinds' bits), the clock set; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::course_faces::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let out = unsafe { face_materials(r, r.esi) };
    let mut left = all_regs(&out);
    left.carry = Some(false);
    Ok(left)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
