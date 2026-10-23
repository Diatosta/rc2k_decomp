//! `race::face_hidden` (0x454D92): whether a course face is left out (carry).
//!
//! In:  esi = the face (0x40 bytes), eax
//! Out: CF (read by `jb` in `sub_454C99`); eax the face's +2 word & `dword_64167C` when
//!      that test is reached and gives 0 or carry; the rest kept
//!
//! With the shown view's (`g_car` +0x684) `dword_606478` entry 0 and `dword_621480` set,
//! carry when the face's +2 word has a bit of `dword_64167C`. Otherwise, inside `pusha`, the
//! first point's (+4) scene record (made when not there, as `sub_44CF33`): `dword_635004` =
//! |x| and `dword_635010` = depth + `flt_63F188` (`fistp`), carry when the depth is below
//! |x| (the sign of the difference). See `race/frame_draw.rs`. Refuses (before anything)
//! no point records or points. Shadow mode: by state (the two dwords, the record,
//! `dword_62400C`) and the carry. Self-test: a test face on the game's points 0 .. 3 (records
//! cleared so they are made, and kept), the hiding test on and off; all put back
//! (skipped without points).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_606478[]
// g_car +0x684
// dword_621480
// dword_64167C

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (eax, carry) = unsafe { face_hidden(r.esi, r.eax) }?;
    Ok(Left::carry(carry).with(EAX, eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
