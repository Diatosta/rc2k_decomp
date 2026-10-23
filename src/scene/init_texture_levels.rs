//! `scene::init_texture_levels` (0x424001): the stage's texture slots, their levels and
//! tags (`LOADLEVEL_INIT : init ed textures`).
//!
//! In:  nothing
//! Out: see `scene::texture_levels::init_texture_levels`
//!
//! See `scene/texture_levels.rs`. One caller, `sub_44426F` (loadlevel_init). Shadow mode
//! runs the original alone (it logs and takes the frame's input). Self-test: a redirected
//! copy whose reset, log and input calls note themselves (its runs on their hook),
//! against the Rust version noting the same: 0, 1, 3 and 9 cars, `dword_63DB94` 1 and
//! not, `dword_621514` 0 and not, prefs entries of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::scene::texture_levels::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { init_texture_levels(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
