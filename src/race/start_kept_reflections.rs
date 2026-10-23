//! `race::start_kept_reflections` (0x451864): the 0x400 kept reflection normals
//! (`g_keptReflectionNormals`) set to -999 and `g_reflectionsKept` 1, before an object is
//! drawn whose reflections are worked out once (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: eax -999, ecx 0, edi past the normals (a `call` follows)
//!
//! Callers: `race::draw_slot_object`, `sub_4616AA` (0x461DF2). Shadow mode: by state.
//! Self-test: the normals and the switch filled with 0xAA first; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

pub fn kept_regions() -> Vec<Region> {
    vec![
        (game_addr(KEPT_NORMALS_RVA) as u32, 0x1000),
        at(REFLECTIONS_KEPT_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { start_kept_reflections() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
