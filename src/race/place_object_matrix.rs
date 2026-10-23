//! `race::place_object_matrix` (0x42739A): `g_madeObjectMatrix` for an object at
//! `dword_63501C` .. (see `race/frame_draw.rs`): turned by its three angles (with
//! `dword_6365D0` 0), facing the view (materials 1, 2), turned with the view (5) or by its
//! own angles and tilts (others), scaled by `dword_635038` / 16, placed and moved by the
//! view. On `scene::matrix` / `scene::view_matrix`'s Rust versions.
//!
//! In:  nothing
//! Out: eax and ebx the place's x and y, ecx 0, edx 0, esi `g_turnedMatrix`, edi
//!      `g_viewTurnMatrix`, ebp `g_madeObjectMatrix` (none changed with `dword_6365C8` 4)
//!
//! It begins with its call: private copy (0x478, its 41 calls relocated), manual patch.
//! Callers: `race::draw_flare`, `race::draw_lamp_flare`, `scene::draw_described_object`
//! (0x451DDF). Shadow mode: by state. Self-test: the angles, place, size, view matrices
//! and the object's material set, each way; all put back.
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
    Ok(all_regs(&unsafe { place_object_matrix(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
