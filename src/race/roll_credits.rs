//! `race::roll_credits` (0x415FC2): the credits rolling up (see `race/frame_text.rs`):
//! eax 0 starts them (clock, top, tune 2), else they are drawn from the top down by their
//! script, the top moving up a pixel each 0x12 ms; carry once they have rolled off and the
//! music has stopped.
//!
//! In:  eax = 0 to start
//! Out: every register preserved (`pusha` .. `popa`); CF as above, else ZF by eax
//!
//! Callers: `sub_4547AE` (over the race with `dword_71C3E0`), the credits' screen
//! (0x415F9F, 0x415FB7), `race::credits_race`. Shadow mode runs the original alone (it draws,
//! reads the clock and starts a tune). Self-test: the script drawn at tops above, on and
//! below the screen with the clock ahead (so it does not move), the music on and off, by
//! the drawing harness; the variables put back (skipped without the blitter's recording).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_text::{credits_tune, roll_credits};
use crate::text::draw_text::GameCanvas;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("race::roll_credits") {
        return Ok(());
    }
    let rolled = unsafe { roll_credits(r.eax, &mut GameCanvas, &mut credits_tune) }?;
    r.set_carry(rolled.carry);
    r.set_zero(rolled.zero);
    Ok(())
}
