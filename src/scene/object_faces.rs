//! The objects' faces (`dword_5C6FA8`, the faces' part of `scene::allocate_object_memory`'s
//! block on `stru_5C6DD0`): Rust's since phase 13 (2026-10-27), counted on the list as the
//! game's bytes were, given back by `race::free_object_points`. The faces are written one
//! after another by `scene::add_object_shape` (+0 the number of vertices, +4 the texture
//! word, +6 the kind, +7, +0xB bytes, +0xC the normal, then 0x10 bytes a vertex).
//!
//! `COURSE_LISTS.face_memory` is `BASE` while the faces are taken (0 otherwise), so a face
//! is named by `BASE` + its offset, where the original held its address (`g_faces`' cells,
//! the marks' point lists): the original's arithmetic (`face + 7`, the faces' memory plus
//! the cursor) is kept, and a place is read here.

use crate::errors::{BadInput, Refused};
use crate::race::course_lists_store::COURSE_LISTS;

/// `COURSE_LISTS.face_memory` while the faces are taken: the first face's place.
pub const BASE: u32 = 1;

crate::memory::owned::thread_owned!(
    pub static FACES: Vec<u8> = "no objects' faces";
);

/// `bytes` of faces, zeroed, counted on `list`.
pub fn take(list: crate::pointers::GamePtr, bytes: u32) -> Result<(), Refused> {
    FACES.take(list, bytes, vec![0; bytes as usize])?;
    COURSE_LISTS.face_memory.set(BASE);
    Ok(())
}

/// Given back (the list's `free_list` counts them down).
pub fn give_back() {
    FACES.give_back();
}

fn range(at: u32, len: usize) -> Option<std::ops::Range<usize>> {
    let start = at.checked_sub(BASE)? as usize;
    Some(start..start.checked_add(len)?)
}

/// `len` bytes at place `at`; refused outside the faces.
pub fn bytes(at: u32, len: usize) -> Result<Vec<u8>, BadInput> {
    FACES
        .with(|f| f.get(range(at, len)?).map(<[u8]>::to_vec))?
        .ok_or(BadInput("a read outside the objects' faces"))
}

/// The byte at place `at`.
pub fn byte(at: u32) -> Result<u8, BadInput> {
    Ok(bytes(at, 1)?[0])
}

/// The dword at place `at`.
pub fn word(at: u32) -> Result<u32, BadInput> {
    let b = bytes(at, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// `data` written at place `at`; refused outside the faces.
pub fn write(at: u32, data: &[u8]) -> Result<(), BadInput> {
    FACES
        .with(|f| {
            f.get_mut(range(at, data.len())?)
                .map(|to| to.copy_from_slice(data))
        })?
        .ok_or(BadInput("a write outside the objects' faces"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_are_read_by_place() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept = COURSE_LISTS.face_memory.get();
        let mut memory = vec![0u8; size_of::<crate::memory::MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        take(
            crate::pointers::GamePtr::new(crate::pointers::game_address(memory.as_mut_ptr())),
            0x40,
        )
        .unwrap();
        let face = COURSE_LISTS.face_memory.get() + 0x10;
        write(face, &[3, 0, 0, 0, 9, 9, 0x1C, 5]).unwrap();
        assert_eq!((word(face).unwrap(), byte(face + 6).unwrap()), (3, 0x1C));
        assert!(bytes(BASE + 0x3E, 4).is_err() && bytes(0, 1).is_err());
        give_back();
        COURSE_LISTS.face_memory.set(kept);
        crate::memory::with_figures(|f| *f = figures);
    }
}
