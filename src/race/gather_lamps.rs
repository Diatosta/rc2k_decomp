//! `race::gather_lamps` (0x454E08): the lamps among the lit faces (object 0x3B), at most
//! 16, into `g_lamps` (+0 the face's first point, +4 its +0x11 byte, +8 0xFFFFFF), each with
//! a depth entry (`add_lamp_entry`, 0x4573FA: the point through `g_viewMatrix`, its depth
//! rounded from the x87 into the key, the entry's second word `g_lampCount` + 0x3000000,
//! `dword_641590` + 1).
//!
//! In:  nothing
//! Out: the registers as it leaves them (ecx the lit faces' count; ebx, esi, eax, edi the
//!      last face's) (a `call` follows)
//!
//! See `race/frame_draw.rs`. `sub_4573FA` is covered here: its only caller is this
//! function. Caller: `sub_4557AB` (0x455821), each view drawn. Refuses (before anything)
//! more than 0x100 lit faces (`g_litFaces` holds 0x100) or a depth list without room for
//! 16.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { gather_lamps(r)? }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// g_courseFaces
// g_litFaceCount
// g_litFaces
