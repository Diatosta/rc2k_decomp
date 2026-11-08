//! `scene::set_up_object_memory` (0x4433C9): the objects' memory and the kinds' records.
//!
//! `scene::allocate_object_memory`, then `scene::read_kind_records`. Two callers (the front
//! end's 3D set-up and a race's).

use crate::errors::Refused;
use crate::race::owner_course::Course;
use crate::scene::allocate_object_memory::allocate_object_memory_in_game;
use crate::scene::read_kind_records::read_kind_records;

// ---- Rust version ----

/// The objects' memory allocated, then the kinds' records read.
pub fn set_up_object_memory(
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    allocate_object_memory_in_game(course)?;
    read_kind_records(scene)?;
    Ok(())
}
