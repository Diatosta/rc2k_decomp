//! `race::shadow_points` (0x458078): the car's shadow's points (see `race/car_shadow.rs`):
//! the ground under the work car (`physics::find_ground`), the sun's slant, each corner of
//! the car turned, moved along the slant, its angle (`fpatan`), distance, the farthest,
//! as whole numbers on the ground; x87 as the original.
//!
//! In:  nothing
//! Out: eax the ground's height (`dword_641AA0`), ebx the count, esi `unk_71A818`, edi past
//!      the last point (no `pusha`); ecx, edx, ebp kept
//!
//! Refuses (before anything) what `find_ground` refuses (its inputs put back) and no
//! corners or more than 0x40. Shadow mode: by state. Self-test: the work car's corners,
//! turn and place set by pattern, the sun's angles; all put back (skipped without a
//! race: the ground is the stage's).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDI, ESI, Regs, game_addr, game_read, rva};
use crate::race::car_shadow::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { shadow_points()? };
    Ok(left())
}

/// The registers it leaves (no `pusha`): eax the ground's height (`dword_641AA0`, its last
/// `mov eax`), ebx the count, esi the work car's turn (`unk_71A818`), edi past the last point.
pub fn left() -> Left {
    let count: u32 = unsafe { game_read(COUNT_RVA) };
    Left::nothing()
        .with(EAX, unsafe { game_read(GROUND_RVA) })
        .with(EBX, count)
        .with(ESI, game_addr(rva(0x71A818)) as u32)
        .with(EDI, game_addr(rva(0x641AA4)) as u32 + 0x20 * count)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
