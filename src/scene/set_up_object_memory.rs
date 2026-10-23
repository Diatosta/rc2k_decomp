//! `scene::set_up_object_memory` (0x4433C9): the objects' memory and the kinds' records.
//!
//! In:  nothing
//! Out: as `scene::allocate_object_memory` leaves them
//!
//! `scene::allocate_object_memory`, then `scene::read_kind_records`.
//! Two callers (the front end's 3D set-up and a race's).
//! Manual patch with a private copy: the function begins with a call.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::allocate_object_memory::allocate_object_memory_in_game;
use crate::scene::read_kind_records::read_kind_records;

/// The calls made, as places in `CALLEES`.
const ORDER: [usize; 2] = [0, 1];

// ---- Rust version ----

/// `step`: what stands for each call, by its place in `ORDER`; it is entered with the
/// registers the call before left.
pub unsafe fn set_up_object_memory(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for n in 0..ORDER.len() {
        step(n, r)?;
    }
    Ok(())
}

/// With the Rust versions of what it calls.
pub unsafe fn set_up_object_memory_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        set_up_object_memory(r, &mut |n, r| {
            match n {
                0 => allocate_object_memory_in_game()?.apply(r),
                _ => read_kind_records()?,
            }
            Ok(())
        })
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    if let Err(e) = unsafe { set_up_object_memory_in_game(r) } {
        *r = entry;
        return Err(e);
    }
    Ok(())
}
