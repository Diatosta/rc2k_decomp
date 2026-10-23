//! `race::reload_objects` (0x425B2F): the objects reloaded for the stage, with their lines
//! in the log (`LOADLEVEL_INIT : reloadobjects`).
//!
//! In:  nothing
//! Out: the registers as `race::set_object_kinds` leaves them
//!
//! See `race/objects.rs`. Two callers (`race::loadlevel_init` and one more). Shadow mode
//! runs the original alone (it loads and logs). Self-test: a redirected copy whose eleven
//! calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::objects::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { reload_objects(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
