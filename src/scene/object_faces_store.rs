//! The objects' faces (the faces' part of `scene::allocate_object_memory`'s block), counted on
//! the list, given back by `give_back`. The faces are written one after another
//! by `scene::add_object_shape` (the number of vertices, the texture word, the kind, the
//! group, flags, shade and unlit bytes, the normal, then 0x10 bytes a vertex).
//!
//! `face_memory` is `BASE` while the faces are taken (0 otherwise), so a face is
//! named by `BASE` + its offset (`face + 7`, the faces' memory plus the cursor), and a place
//! is read here.

use crate::errors::{BadInput, Refused};
use crate::memory::shared_table::SharedTable;

/// `face_memory` while the faces are taken: the first face's place.
pub const BASE: u32 = 1;

/// The objects' faces: at most `scene::allocate_object_memory`'s room for them out of a race.
pub struct ObjectFaces(SharedTable);

impl ObjectFaces {
    pub const fn new() -> Self {
        ObjectFaces(SharedTable::new(0x40_0000, "no objects' faces"))
    }

    /// `bytes` of faces, zeroed, counted on `list`.
    pub fn take(
        &self,
        course: &crate::race::owner_course::Course,
        list: &crate::memory::ListCells,
        bytes: u32,
    ) -> Result<(), Refused> {
        crate::memory::owned::count_on(list, bytes)?;
        self.0.set(&vec![0; bytes as usize])?;
        course.lists.borrow_mut().face_memory = BASE;
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

    /// The `N` bytes at place `at`; refused outside the faces.
    pub fn array<const N: usize>(&self, at: u32) -> Result<[u8; N], BadInput> {
        let range = range(at, N).ok_or(BadInput("a read outside the objects' faces"))?;
        self.0
            .array(range.start)?
            .ok_or(BadInput("a read outside the objects' faces"))
    }

    /// `into`'s length of bytes at place `at`; refused outside the faces.
    pub fn read_into(&self, at: u32, into: &mut [u8]) -> Result<(), BadInput> {
        let range = range(at, into.len()).ok_or(BadInput("a read outside the objects' faces"))?;
        self.0
            .read(range.start, into)?
            .ok_or(BadInput("a read outside the objects' faces"))
    }

    /// The byte at place `at`.
    pub fn byte(&self, at: u32) -> Result<u8, BadInput> {
        Ok(self.array::<1>(at)?[0])
    }

    /// The dword at place `at`.
    pub fn word(&self, at: u32) -> Result<u32, BadInput> {
        Ok(u32::from_le_bytes(self.array(at)?))
    }

    /// `data` written at place `at`; refused outside the faces.
    pub fn write(&self, at: u32, data: &[u8]) -> Result<(), BadInput> {
        let range = range(at, data.len()).ok_or(BadInput("a write outside the objects' faces"))?;
        self.0
            .write(range.start, data)?
            .ok_or(BadInput("a write outside the objects' faces"))
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
    fn faces_are_read_by_place() {
        let course = &crate::test_support::own::race().course;
        let faces = &course.stage.object_faces;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = course.lists.borrow().face_memory;
        let memory = crate::memory::ListCells::empty();
        faces.take(course, &memory, 0x40).unwrap();
        let face = course.lists.borrow().face_memory + 0x10;
        faces.write(face, &[3, 0, 0, 0, 9, 9, 0x1C, 5]).unwrap();
        assert_eq!(
            (faces.word(face).unwrap(), faces.byte(face + 6).unwrap()),
            (3, 0x1C)
        );
        assert!(faces.array::<4>(BASE + 0x3E).is_err() && faces.array::<1>(0).is_err());
        faces.give_back();
        course.lists.borrow_mut().face_memory = kept;
        crate::memory::with_figures(|f| *f = figures);
    }
}
