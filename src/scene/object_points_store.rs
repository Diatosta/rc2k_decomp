//! The objects' points and normals (the points' part of `scene::allocate_object_memory`'s block),
//! counted on the list, given back by `give_back`. Twelve bytes each (three whole
//! numbers, or a normal's three floats), an object's points then its normals
//! (`first_point`).
//!
//! `object_points` is `BASE` while they are taken (0 otherwise), so a point is named
//! by `BASE` + 12 n (the marks' lists, the contact walks), and a place is read here.

use crate::errors::{BadInput, Refused};
use crate::memory::shared_table::SharedTable;

/// `object_points` while the points are taken: point 0's place.
pub const BASE: u32 = 1;

/// The objects' points: at most `scene::allocate_object_memory`'s room for them out of a race.
pub struct ObjectPoints(SharedTable);

impl ObjectPoints {
    pub const fn new() -> Self {
        ObjectPoints(SharedTable::new(0x18_0000, "no objects' points"))
    }

    /// `bytes` of points, zeroed, counted on `list`.
    pub fn take(
        &self,
        course: &crate::race::owner_course::Course,
        list: &crate::memory::PlainList,
        bytes: u32,
    ) -> Result<(), Refused> {
        crate::memory::owned::count_on(list, bytes)?;
        self.0.set(&vec![0; bytes as usize])?;
        course.lists.borrow_mut().object_points = BASE;
        Ok(())
    }

    /// Given back (the list's `free_list` counts them down).
    pub fn give_back(&self) {
        self.0.give_back();
    }

    /// The table, for the stage's hash.
    #[cfg(feature = "remote")]
    pub fn table(&self) -> &SharedTable {
        &self.0
    }

    /// The `N` bytes at place `at`; refused outside the points.
    fn array<const N: usize>(&self, at: u32) -> Result<[u8; N], BadInput> {
        let range = range(at, N).ok_or(BadInput("a read outside the objects' points"))?;
        self.0
            .array(range.start)?
            .ok_or(BadInput("a read outside the objects' points"))
    }

    /// The dword at place `at`.
    pub fn word(&self, at: u32) -> Result<u32, BadInput> {
        Ok(u32::from_le_bytes(self.array(at)?))
    }

    /// The three dwords of the point at place `at`.
    pub fn point(&self, at: u32) -> Result<[u32; 3], BadInput> {
        let b: [u8; 12] = self.array(at)?;
        Ok(std::array::from_fn(|k| {
            u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
        }))
    }

    /// `data` written at place `at`; refused outside the points.
    pub fn write(&self, at: u32, data: &[u8]) -> Result<(), BadInput> {
        let range = range(at, data.len()).ok_or(BadInput("a write outside the objects' points"))?;
        self.0
            .write(range.start, data)?
            .ok_or(BadInput("a write outside the objects' points"))
    }

    /// The point at place `at` set to `value`.
    pub fn set_point(&self, at: u32, value: [u32; 3]) -> Result<(), BadInput> {
        let mut b = [0u8; 12];
        for (bytes, dword) in b.as_chunks_mut::<4>().0.iter_mut().zip(value) {
            bytes.copy_from_slice(&dword.to_le_bytes());
        }
        self.write(at, &b)
    }

    /// The first `bytes` zeroed (`scene::reset_objects`).
    pub fn clear(&self, bytes: usize) -> Result<(), BadInput> {
        self.0
            .write(0, &vec![0; bytes])?
            .ok_or(BadInput("a clear past the objects' points"))
    }
}

fn range(at: u32, len: usize) -> Option<std::ops::Range<usize>> {
    let start = at.checked_sub(BASE)? as usize;
    Some(start..start.checked_add(len)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_are_read_by_place() {
        let course = &crate::test_support::own::race().course;
        let points = &course.stage.object_points;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = course.lists.borrow().object_points;
        let memory = crate::memory::PlainList::empty();
        points.take(course, &memory, 36).unwrap();
        let p = course.lists.borrow().object_points + 12;
        points.set_point(p, [1, 2, 3]).unwrap();
        assert_eq!(
            (points.point(p).unwrap(), points.word(p + 8).unwrap()),
            ([1, 2, 3], 3)
        );
        assert!(points.point(BASE + 25).is_err() && points.word(0).is_err());
        points.clear(36).unwrap();
        assert_eq!(points.point(p).unwrap(), [0; 3]);
        points.give_back();
        course.lists.borrow_mut().object_points = kept;
        crate::memory::with_figures(|f| *f = figures);
    }
}
