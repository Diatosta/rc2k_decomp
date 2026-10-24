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
use crate::scene::allocate_object_memory::allocate_object_memory_in_game;
use crate::scene::read_kind_records::read_kind_records;
use crate::screens::harness::Left;

// ---- Rust version ----

/// The objects' memory allocated, then the kinds' records read. What it leaves in the
/// registers is the allocation's (the records' reading changes none).
pub unsafe fn set_up_object_memory() -> Result<Left, Refused> {
    unsafe {
        let left = allocate_object_memory_in_game()?;
        read_kind_records()?;
        Ok(left)
    }
}
