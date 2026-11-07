//! The objects' points and normals (`g_objectPoints`, the points' part of
//! `scene::allocate_object_memory`'s block on `stru_5C6DD0`): Rust's since phase 13
//! (2026-10-27), counted on the list as the game's block was, given back by
//! `race::free_object_points`. Twelve bytes each (three whole numbers, or a normal's
//! three floats), an object's points then its normals (`OBJECT_SLOTS.first_point`).
//!
//! `COURSE_LISTS.object_points` is `BASE` while they are taken (0 otherwise), so a point is
//! named by `BASE` + 12 n where the original held its address (the marks' lists, the
//! contact walks): the original's arithmetic is kept, and a place is read here.

use crate::errors::{BadInput, Refused};
#[cfg(test)]
use crate::race::owner::race;

/// `COURSE_LISTS.object_points` while the points are taken: point 0's place.
pub const BASE: u32 = 1;

crate::memory::owned::thread_owned!(
    pub static POINTS: Vec<u8> = "no objects' points";
);

/// `bytes` of points, zeroed, counted on `list`.
pub fn take(
    course: &crate::race::owner_course::Course,
    list: &dyn crate::pointers::Store,
    bytes: u32,
) -> Result<(), Refused> {
    POINTS.take(list, bytes, vec![0; bytes as usize])?;
    course.lists.object_points.set(BASE);
    Ok(())
}

/// Given back (the list's `free_list` counts them down).
pub fn give_back() {
    POINTS.give_back();
}

fn range(at: u32, len: usize) -> Option<std::ops::Range<usize>> {
    let start = at.checked_sub(BASE)? as usize;
    Some(start..start.checked_add(len)?)
}

/// The `N` bytes at place `at`; refused outside the points.
fn array<const N: usize>(at: u32) -> Result<[u8; N], BadInput> {
    POINTS
        .with(|p| {
            let run = p.get(range(at, N)?)?;
            <[u8; N]>::try_from(run).ok()
        })?
        .ok_or(BadInput("a read outside the objects' points"))
}

/// The dword at place `at`.
pub fn word(at: u32) -> Result<u32, BadInput> {
    Ok(u32::from_le_bytes(array(at)?))
}

/// The three dwords of the point at place `at`.
pub fn point(at: u32) -> Result<[u32; 3], BadInput> {
    let b: [u8; 12] = array(at)?;
    Ok(std::array::from_fn(|k| {
        u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
    }))
}

/// `data` written at place `at`; refused outside the points.
pub fn write(at: u32, data: &[u8]) -> Result<(), BadInput> {
    POINTS
        .with(|p| {
            p.get_mut(range(at, data.len())?)
                .map(|to| to.copy_from_slice(data))
        })?
        .ok_or(BadInput("a write outside the objects' points"))
}

/// The point at place `at` set to `value`.
pub fn set_point(at: u32, value: [u32; 3]) -> Result<(), BadInput> {
    let b: Vec<u8> = value.iter().flat_map(|d| d.to_le_bytes()).collect();
    write(at, &b)
}

/// The first `bytes` zeroed (`scene::reset_objects`).
pub fn clear(bytes: usize) -> Result<(), BadInput> {
    POINTS
        .with(|p| p.get_mut(..bytes).map(|b| b.fill(0)))?
        .ok_or(BadInput("a clear past the objects' points"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_are_read_by_place() {
        let course = &race().course;
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept = course.lists.object_points.get();
        let memory = crate::memory::ListCells::empty();
        take(course, &memory, 36).unwrap();
        let p = course.lists.object_points.get() + 12;
        set_point(p, [1, 2, 3]).unwrap();
        assert_eq!((point(p).unwrap(), word(p + 8).unwrap()), ([1, 2, 3], 3));
        assert!(point(BASE + 25).is_err() && word(0).is_err());
        clear(36).unwrap();
        assert_eq!(point(p).unwrap(), [0; 3]);
        give_back();
        course.lists.object_points.set(kept);
        crate::memory::with_figures(|f| *f = figures);
    }
}
