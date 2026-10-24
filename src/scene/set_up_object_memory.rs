//! `scene::set_up_object_memory` (0x4433C9): the objects' memory and the kinds' records.
//!
//! In:  nothing
//! Out: as `scene::allocate_object_memory` leaves them
//!
//! `scene::allocate_object_memory`, then `scene::read_kind_records`. Two callers (the front
//! end's 3D set-up and a race's).

use crate::errors::Refused;
use crate::scene::allocate_object_memory::allocate_object_memory_in_game;
use crate::scene::read_kind_records::read_kind_records;

// ---- Rust version ----

/// The objects' memory allocated, then the kinds' records read.
pub unsafe fn set_up_object_memory() -> Result<(), Refused> {
    unsafe {
        allocate_object_memory_in_game()?;
        read_kind_records()?;
        Ok(())
    }
}
