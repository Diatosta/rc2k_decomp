//! `race::copy_mirror_picture` (0x455DAD): the mirror's surface copied onto slot
//! `dword_5D0E08`'s with alpha 0xFF, pixel by pixel through the two formats (see
//! `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as the unlocks leave them (its caller goes on with a `mov`)
//!
//! The surfaces' lock and unlock are the platform's (`sub_41AA4B` / `sub_41AB02`,
//! `call_game`). Refuses a destination without its format (after the locks: it is known only
//! then; the original would fault). Caller: `race::forget_view_textures`. Shadow mode runs
//! the original alone (it writes a surface). Self-test: a copy whose locks and unlocks go to
//! stand-ins handing out test surfaces, against the Rust version with the same: 16 and 32
//! bits each side, test formats, a failed lock of each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_views::copy_mirror_picture;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { copy_mirror_picture(r) }?;
    Ok(())
}
