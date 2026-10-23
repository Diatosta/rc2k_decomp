//! `race::face_hide_mask` (0x456056): `physics::probe_ground`, then, unless the face under
//! the car (`g_car` +0xE00) is -1, `dword_64167C` from its +2 word: bits 1 and 3 flipped
//! when either is set, or'd with bits 4 and 5 likewise.
//!
//! In:  nothing
//! Out: as `probe_ground` leaves them, then ebx (the face, or its bits 4/5), esi (the face
//!      record), eax (the mask) when the face is there (a `call` follows)
//!
//! It begins with its call: private copy (0x3B, the call relocated), manual patch. See
//! `race/frame_draw.rs`. Caller: `race::prepare_course_view` (0x4548B2). Refuses what
//! `probe_ground` refuses (before anything). Shadow mode: by state (`probe_ground`'s
//! regions and the mask). Self-test: on the game's state (the ground as it is); all put
//! back (skipped without a stage). Its branches show in `stats` under play only.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, rva};
use crate::physics::ground::Ground;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

pub fn mask_regions() -> Vec<Region> {
    let mut v = crate::physics::probe_ground::regions(&Ground::game());
    v.push(at(rva(0x64167C)));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { face_hide_mask(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
