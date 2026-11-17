//! The race's drawing (`drive::race_frame`'s tree): its leaves that are game logic, not platform.
//! The panel's fields are the HUD record the frame fills for the HUD drawers
//! (`race::panel_store`).

use crate::errors::BadInput;
use crate::race::owner_course::CourseReads;

mod course_view;
mod lights;
mod panel;
mod scene_objects;
mod view;

pub use course_view::*;
pub use lights::*;
pub use panel::*;
pub use scene_objects::*;
pub use view::*;

/// Field `offset` of course face `face`.
fn face_field<T: crate::race::course_places::Whole>(
    course: CourseReads<'_>,
    face: u32,
    offset: u32,
) -> Result<T, BadInput> {
    crate::race::course_places::face_field(course, face, offset)
}

/// Field `offset` of course record `record`.
fn record_field(course: CourseReads<'_>, record: u32, offset: u32) -> Result<u32, BadInput> {
    crate::race::course_places::record_field(course, record, offset)
}
