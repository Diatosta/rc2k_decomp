//! `race::gather_record_faces` (0x454C99): the faces of one course record into the draw
//! lists (see `race/frame_draw.rs`).
//!
//! In:  esi = the course record (0x20 bytes: +8 first face, +0xC last, -1 none)
//! Out: the registers as it leaves them (ebx the face after the last; eax, ecx, edx as the
//!      last face's tests and entry left them); its callers call it inside `pusha`
//!
//! Each face, `dword_5CCCD0` the face: kinds 0xFF and below 0x10 that `race::face_hidden`
//! lets through go to `race::add_depth_entry` (the object's flag in `byte_63DD5C` /
//! `byte_63DC5C`) or `race::add_sort_entry` (the object's kind byte, `byte_63DF5C` /
//! `byte_63DE5C`); for kinds below 0x10 the object is +1, or +0x21 when +1 is 0. Refuses
//! (before anything) a last face of -1, no points, or more faces than either list has room
//! for. Shadow mode: by state (the lists, their counts, the tests' dwords, the faces'
//! point records). Self-test: test records over the game's faces 0 .. 39 (ranges, one face,
//! none, first above last), the lists' counts set; all put back (skipped without a stage).
//! Caller: `race::gather_visible_faces`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { gather_record_faces(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// Whether a stage is there to gather from: faces, point records and points.
pub fn stage_there() -> bool {
    unsafe {
        game_read::<u32>(rva(0x5CCCBC)) != 0
            && game_read::<u32>(crate::scene::project_point::RECORDS_RVA) != 0
            && game_read::<u32>(crate::scene::project_point::POINTS_RVA) != 0
            && game_read::<u32>(rva(0x5CCD1C)) >= 40 // g_courseFaceCount
    }
}
