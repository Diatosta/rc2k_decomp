//! `race::set_object_kinds` (0x425BB4): the object slots' kinds, the objects loaded, the
//! cars' collision points.
//!
//! In:  nothing
//! Out: the registers as the last call leaves them
//!
//! See `race/objects.rs`. One caller, `race::reload_objects`. Refuses more cars than the
//! eight blocks. It begins with its call: manual patch, private copy (its six calls
//! relocated). Shadow mode runs the original alone (it loads). Self-test: a redirected copy
//! whose six calls note themselves, 0, 2 and 8 cars, the game's variables put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::objects::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_object_kinds(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
