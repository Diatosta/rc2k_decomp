//! Readers of the game's memory for the converted cores, whose closures cannot fail: the
//! first bad address is kept, the reader answers 0 from then on, and `finish` refuses
//! afterwards (so the core's own answer on those zeros is never used).

use crate::errors::BadInput;
use std::cell::Cell;

/// Reads that remember their first refusal.
#[derive(Default)]
pub struct Reading {
    failed: Cell<Option<BadInput>>,
}

impl Reading {
    pub fn new() -> Self {
        Self::default()
    }

    /// `value`'s result kept as a read's: its default after a refusal (this one or one
    /// before).
    pub fn keep<T: Default>(&self, value: Result<T, BadInput>) -> T {
        if self.failed.get().is_some() {
            return T::default();
        }
        value.unwrap_or_else(|e| {
            self.failed.set(Some(e));
            T::default()
        })
    }

    /// The first refusal, if any read had one.
    pub fn finish(&self) -> Result<(), BadInput> {
        match self.failed.take() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

/// The objects' point records (`scene::point_records`, 0x1000 + (n & 0xFFF) for a point or
/// normal number `n`; its place, three words), the drawing part's.
pub struct PointRecords<'a> {
    part: &'a crate::scene::workers::PartScratch,
}

impl<'a> PointRecords<'a> {
    /// `part`'s records; refused when the records are not taken.
    pub fn new(
        lists: &crate::race::course_lists_store::CourseLists,
        part: &'a crate::scene::workers::PartScratch,
    ) -> Result<Self, BadInput> {
        if !crate::scene::point_records::held(lists) {
            return Err(BadInput("no records for the points"));
        }
        Ok(Self { part })
    }

    /// The place of point or normal `number`.
    pub fn place(&self, number: u32) -> Result<[u32; 3], BadInput> {
        use crate::scene::point_records::{get, of_object_point};
        let r = get(self.part, of_object_point(number))?;
        Ok([r[4], r[5], r[6]])
    }
}
