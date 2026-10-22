//! `race::gather_visible_faces` (0x454B71): `race::gather_record_faces` of each course
//! record in `g_courseGrid` (numbers below 0x0FFFFFFF: the far cells' marked ones are left
//! out), then of the last record once more.
//!
//! In:  nothing
//! Out: ecx 0x400, ebp the last record's number; the rest kept (each gathering is inside
//!      `pusha`; a `call` follows)
//!
//! Its test for the last record having been gathered sets ebp inside the `pusha`, so it is
//! lost and the last record is always gathered again (`notes/bugs.md`). See
//! `race/frame_draw.rs`. Caller: `sub_4557AB` (0x455817), each view drawn. Refuses (before
//! anything) what `gather_record_faces` would for any of the records. Shadow mode: by
//! state. Self-test: the grid set to a few of the game's records (and marked cells), the
//! lists' counts set; all put back (skipped without a stage).

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { gather_visible_faces(r)? }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
