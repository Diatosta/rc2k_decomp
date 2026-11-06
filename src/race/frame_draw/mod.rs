//! The race's drawing (`sub_4541BF`'s tree, IDA 2026-10-17): its leaves that are game logic,
//! not platform. The panel's fields at 0x6D14xx are a reading.

use crate::errors::BadInput;

mod course_view;
mod environment;
mod lights;
mod panel;
mod scene_objects;
mod view;

pub use course_view::*;
pub use environment::*;
pub use lights::*;
pub use panel::*;
pub use scene_objects::*;
pub use view::*;

/// Field `offset` of course face `face`.
fn face_field<T: crate::race::course_places::Whole>(face: u32, offset: u32) -> Result<T, BadInput> {
    crate::race::course_places::face_field(face, offset, "a course face")
}

/// Field `offset` of course record `record`.
fn record_field(record: u32, offset: u32) -> Result<u32, BadInput> {
    crate::race::course_places::record_field(record, offset, "a course record")
}
