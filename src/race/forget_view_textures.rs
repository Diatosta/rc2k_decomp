//! `race::forget_view_textures` (0x45B200): the views' cars' changing textures (number
//! plates, dirt, the two extra slots) forgotten so they load again, their bits in
//! `dword_6415F4`; with the mirror on its picture copied or uploaded (see
//! `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) a null view block or a slot past 0x3FF. The upload
//! (`sub_44E828`) is the platform's (`call_game`). Caller: `sub_45429B` (0x45443D). Shadow
//! mode runs the original alone (the mirror's copy writes a surface). Self-test: a copy whose
//! two calls out go to stand-ins (the forgetting real), against the Rust version with the
//! same: test view blocks, each switch on and off, the mirror each way.

use crate::framework::Regs;
use crate::race::frame_views::{forget_view_textures, view_textures_part};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { forget_view_textures(r, &mut view_textures_part) }?;
    Ok(())
}
