//! The course's faces (`g_courseFaces`, 0x40 bytes each) and records (`dword_5CCCB8`,
//! 0x20 bytes each): parts of the course memory, Rust's since phase 13 (2026-10-27).
//! `race::load_level` moves them out once the course is read (with the scene points,
//! `race::scene_points`), and `race::free_course_memory` gives them back.
//!
//! While held, `COURSE.course_faces` and `COURSE.course_records` are places in a range of
//! addresses reserved with no access (taken at the first course, kept for the program's
//! life), so the original's arithmetic (`faces + 0x40 n + 7`) names a place, read here: no
//! host memory lies there and the address map hands none of it out. A reader that still
//! takes a place for an address is refused by the checked paths with the number named.
//!
//! `read` / `write` take any of the course's places or a game address: the game's helpers
//! that read a face, a record, a scene point or a frame's word alike go through them.

use crate::errors::BadInput;
use crate::memory::owned::Owned;
use crate::pointers::GamePtr;
use rc2k_core::race::faces::{Face, Record};
use std::sync::atomic::{AtomicU32, Ordering};

/// Each part's room: the course memory's first part (`level_init::FIRST_PART`) holds both.
const ROOM: u32 = 0x100_0000;
/// Faces, records, scene points, course info.
const PARTS: u32 = 4;

/// The reserved range's game address (faces from it, records `ROOM` on, the scene points
/// `2 ROOM` on, `race::scene_points`, the course info `3 ROOM` on); 0 until taken.
static WINDOW: AtomicU32 = AtomicU32::new(0);

/// The reserved range, taken the first time.
pub fn reserve() -> Result<u32, BadInput> {
    let at = WINDOW.load(Ordering::Acquire);
    if at != 0 {
        return Ok(at);
    }
    let len = PARTS as usize * ROOM as usize;
    let reserved =
        crate::os::reserve(len).ok_or(BadInput("no addresses for the course's places"))?;
    let at = crate::address_map::map(reserved, len);
    WINDOW.store(at, Ordering::Release);
    Ok(at)
}

/// Face 0's place (`COURSE.course_faces` while held).
pub fn faces_base() -> u32 {
    WINDOW.load(Ordering::Acquire)
}

/// Record 0's place (`COURSE.course_records` while held).
pub fn records_base() -> u32 {
    faces_base() + ROOM
}

/// Scene point 0's place (`COURSE.scene_points` while held).
pub fn points_base() -> u32 {
    faces_base() + 2 * ROOM
}

/// The course info's place (`COURSE_LISTS.course_info` while held).
pub fn info_base() -> u32 {
    faces_base() + 3 * ROOM
}

/// The most bytes of each part.
pub const fn room() -> u32 {
    ROOM
}

pub static FACES: Owned<Vec<u8>> = Owned::new("no course faces");
pub static RECORDS: Owned<Vec<u8>> = Owned::new("no course records");
/// The course info (`g_courseInfo`, 0x400000 bytes: the texture infos `files\a\textinfo`
/// and `files\b\textinfo`, the mark corners `files\m\markinfo` at +0x1C2000): taken with
/// the course memory (`level_init::allocate_course_memory`), filled by
/// `level_init::load_texture_info`.
pub static INFO: Owned<Vec<u8>> = Owned::new("no course info");

/// The course info taken (`len` bytes, zeroed): its place.
pub fn take_info(len: u32) -> Result<u32, BadInput> {
    if len > ROOM {
        return Err(BadInput("a course info larger than its places"));
    }
    reserve()?;
    INFO.set(vec![0; len as usize]);
    Ok(info_base())
}

/// The file at game address `path` read whole into the course info from `at` (`count`
/// bytes at most).
pub fn read_file_into_info(path: u32, at: u32, count: u32) -> Result<(), BadInput> {
    let path = crate::pointers::GamePtr::new(path);
    INFO.with(|b| {
        let start = at.checked_sub(info_base())? as usize;
        let into = b.get_mut(start..start.checked_add(count as usize)?)?;
        Some(crate::files::read_whole_file::read_whole_file_into(
            path, into,
        ))
    })?
    .ok_or(BadInput("a file read outside the course info"))?
}

/// Whether the faces and records are held.
pub fn held() -> bool {
    FACES.is_taken()
}

/// The faces (`faces` bytes at game address `faces_at`) and records moved out of the
/// course memory.
pub fn take_from(faces_at: u32, faces: u32, records_at: u32, records: u32) -> Result<(), BadInput> {
    if faces > ROOM || records > ROOM {
        return Err(BadInput(
            "more course faces or records than their places hold",
        ));
    }
    reserve()?;
    let f = GamePtr::new(faces_at).read_bytes(faces as usize, "the course faces")?;
    let r = GamePtr::new(records_at).read_bytes(records as usize, "the course records")?;
    FACES.set(f);
    RECORDS.set(r);
    Ok(())
}

/// Given back with the course memory.
pub fn give_back() {
    FACES.give_back();
    RECORDS.give_back();
    INFO.give_back();
}

/// Whether `at` is one of the course's places (faces, records, scene points).
#[inline]
pub fn is_place(at: u32) -> bool {
    let w = faces_base();
    w != 0 && at.wrapping_sub(w) < PARTS * ROOM
}

/// The store and offset of place `at`, if it is one of the faces' or records'.
fn store(at: u32) -> Option<(&'static Owned<Vec<u8>>, usize)> {
    let w = faces_base();
    if w == 0 {
        return None;
    }
    match at.wrapping_sub(w) {
        off if off < ROOM => Some((&FACES, off as usize)),
        off if off < 2 * ROOM => Some((&RECORDS, (off - ROOM) as usize)),
        off if (3 * ROOM..4 * ROOM).contains(&off) => Some((&INFO, (off - 3 * ROOM) as usize)),
        _ => None,
    }
}

/// Whether `at` is a place of the scene points.
fn scene_point(at: u32) -> bool {
    faces_base() != 0 && at.wrapping_sub(points_base()) < ROOM
}

/// `len` bytes at `at`: a face's, a record's or a scene point's place, or a game address.
pub fn bytes(at: u32, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
    if let Some((s, off)) = store(at) {
        return s
            .with(|b| b.get(off..off.checked_add(len)?).map(<[u8]>::to_vec))?
            .ok_or(BadInput("a read outside the course faces or records"));
    }
    if scene_point(at) {
        return crate::race::scene_points::bytes(at, len);
    }
    GamePtr::new(at).read_bytes(len, what)
}

/// `data` written at `at` (as `bytes` names it).
pub fn write_bytes(at: u32, data: &[u8], what: &str) -> Result<(), BadInput> {
    if let Some((s, off)) = store(at) {
        return s
            .with(|b| {
                b.get_mut(off..off.checked_add(data.len())?)
                    .map(|to| to.copy_from_slice(data))
            })?
            .ok_or(BadInput("a write outside the course faces or records"));
    }
    if scene_point(at) {
        return crate::race::scene_points::write(at, data);
    }
    GamePtr::new(at).write_bytes(data, what)
}

/// A `T` (a whole number) at `at`.
pub fn read<T: Copy>(at: u32, what: &str) -> Result<T, BadInput> {
    let b = bytes(at, size_of::<T>(), what)?;
    // SAFETY: `b` holds `size_of::<T>()` bytes; `T` is a whole number (any bits).
    Ok(unsafe { b.as_ptr().cast::<T>().read_unaligned() })
}

/// As `read`; a refusal ends the game (for the readers that cannot carry one).
pub fn read_or_end<T: Copy>(at: u32, what: &str) -> T {
    read(at, what).unwrap_or_else(|e| crate::fatal::raise::end(what, e))
}

/// `v` (a whole number) written at `at`.
pub fn write<T: Copy>(at: u32, v: T, what: &str) -> Result<(), BadInput> {
    // SAFETY: `v` is a whole number: its bytes are all initialised.
    let b = unsafe { std::slice::from_raw_parts((&raw const v).cast::<u8>(), size_of::<T>()) };
    write_bytes(at, b, what)
}

/// The `N` bytes at `at` (as `bytes` names it), a store's without a heap copy.
fn array<const N: usize>(at: u32, what: &str) -> Result<[u8; N], BadInput> {
    let mut a = [0; N];
    if let Some((s, off)) = store(at) {
        s.with(|b| {
            b.get(off..off.checked_add(N)?)
                .map(|from| a.copy_from_slice(from))
        })?
        .ok_or(BadInput("a read outside the course faces or records"))?;
        return Ok(a);
    }
    a.copy_from_slice(&bytes(at, N, what)?);
    Ok(a)
}

/// The face at `at`.
pub fn face(at: u32, what: &str) -> Result<Face, BadInput> {
    array(at, what).map(Face)
}

/// The face at `at` changed by `change`.
pub fn update_face(at: u32, what: &str, change: impl FnOnce(&mut Face)) -> Result<(), BadInput> {
    let mut f = face(at, what)?;
    change(&mut f);
    write_bytes(at, &f.0, what)
}

/// The record at `at`.
pub fn record(at: u32, what: &str) -> Result<Record, BadInput> {
    array(at, what).map(Record)
}
