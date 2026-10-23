//! `physics::face_under_point` (0x442006): whether the point (`dword_639824`,
//! `dword_63982C`) is inside course face ebx (`g_courseFaces`, 0x40 bytes: kind byte,
//! flags word at +2, corners +4 .. +0x10; a triangle when +0xC = +0x10). `dword_5CCC70`
//! = ebx first. Out (CF clear) for a kind of 0x10 or more, flags in the mask
//! `dword_620470`, or an edge (`physics::edge_side`) whose ends are the same or that has
//! the point on its other side. A triangle's edges 4-8, 8-0xC, 0xC-4; a quad's 8-0xC,
//! then 0xC-4, 4-8 (its first half) or, the point past 8-0xC, 0x10-0xC, 8-0x10 (its
//! second). Inside (CF set): `dword_5CCC74` = the face, `dword_5CCC6C` the half (0, 1),
//! the half's corners (4, 8, 0xC or 0x10, 0xC, 8) into 0x639890 .. and 0x5CCC60 ..,
//! `physics::face_shares`, `dword_639820` = 1, `physics::plane_height`.
//!
//! In:  ebx the face
//! Out: CF; the registers as the path leaves them (see `ground::face_under_point_at`):
//!      out at the kind: al the kind; at the flags: eax the masked flags; at an edge:
//!      eax `edge_side`'s, ebx the edge's second corner, ecx = esi the face's address,
//!      ebp, edi the edge's points; inside: ebx the face, esi its address, eax, ecx as
//!      `face_shares` leaves them, ebp, edi the last edge's. edx kept.
//!
//! See `physics/ground.rs`. Callers: `physics::try_last_face` and
//! `physics::search_record_faces` (CF read with `jb`), and unreferenced code at
//! 0x441F63 (a search of every face).
//! Refuses a face at or past `g_courseFaceCount` (before anything).
//! Self-test: a copy of the original (its calls to copies of `edge_side`,
//! `face_shares`, `plane_height`) on the test's ground: every face of a grid laid out
//! three ways against points in, on and out, masks 0 and 0x100, both SSE ways; the
//! cases inside counted; the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: `r` as the original leaves it, and the carry.
pub unsafe fn face_under_point(r: &mut Regs) -> Result<bool, Refused> {
    unsafe { face_under_point_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    let carry = unsafe { face_under_point(&mut g) }?;
    Ok(left_of(&g, carry))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
