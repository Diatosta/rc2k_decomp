//! The course's faces (`g_courseFaces`, 0x40 bytes each) and records (`course_records`,
//! 0x20 bytes each): parts of the course memory, Rust's.
//! `race::load_level` moves them out once the course is read (with the scene points,
//! `race::scene_points`), and `race::free_course_memory` gives them back.
//!
//! While held, the faces and records are entries of their tables, reached by number
//! (`face_number`, `face_field`, `record_field`, ..; a field past its entry reads on into
//! the next, as the original's arithmetic did) and the course info by offset
//! (`info_bytes`, `info_field`). `COURSE.course_faces` and `COURSE.course_records` hold the
//! tables' bases, numbers in a range of addresses reserved with no access (taken at the
//! first course, kept for the program's life): nothing reads through them, and a reader
//! that still took one for an address is ended by `pointers::host` with the number named.

use crate::errors::BadInput;
use crate::race::owner_course::Course;
use rc2k_core::race::faces::Face;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

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
    let at = crate::address_map::reserve(len);
    WINDOW.store(at, Ordering::Release);
    Ok(at)
}

/// Face 0's place (`COURSE.course_faces` while held).
pub fn faces_base() -> u32 {
    // A number only: nothing is published through it.
    WINDOW.load(Ordering::Relaxed)
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

use crate::memory::shared_table::SharedTable;

pub static FACES: SharedTable = SharedTable::new(ROOM as usize, "no course faces");
pub static RECORDS: SharedTable = SharedTable::new(ROOM as usize, "no course records");
/// The course info (`g_courseInfo`, 0x400000 bytes: the texture infos `files\a\textinfo`
/// and `files\b\textinfo`, the mark corners `files\m\markinfo` at +0x1C2000): taken with
/// the course memory (`level_init::allocate_course_memory`), filled by
/// `level_init::load_texture_info`.
pub static INFO: SharedTable = SharedTable::new(ROOM as usize, "no course info");

/// The course info taken (`len` bytes, zeroed): its place.
pub fn take_info(len: u32) -> Result<u32, BadInput> {
    if len > ROOM {
        return Err(BadInput("a course info larger than its places"));
    }
    reserve()?;
    INFO.set(&vec![0; len as usize])?;
    Ok(info_base())
}

/// The file at game address `path` read whole into the course info from byte `offset`
/// (`count` bytes at most).
pub fn read_file_into_info(path: &[u8], offset: u32, count: u32) -> Result<(), BadInput> {
    let start = offset as usize;
    let mut into = INFO
        .bytes(start, count as usize)?
        .ok_or(BadInput("a file read outside the course info"))?;
    let read = crate::files::read_whole_file::read_whole_file_into(path, &mut into);
    INFO.write(start, &into)?;
    read
}

/// `len` bytes of the course info from byte `offset`.
pub fn info_bytes(offset: u32, len: usize) -> Result<Vec<u8>, BadInput> {
    INFO.bytes(offset as usize, len)?
        .ok_or(BadInput("a read outside the course info"))
}

/// `data` written into the course info from byte `offset`.
pub fn write_info(offset: u32, data: &[u8]) -> Result<(), BadInput> {
    INFO.write(offset as usize, data)?
        .ok_or(BadInput("a write outside the course info"))
}

/// The whole number at byte `offset` of the course info.
pub fn info_field<T: Whole>(offset: u32) -> Result<T, BadInput> {
    let mut bytes = [0u8; 64];
    let len = size_of::<T>();
    bytes[..len].copy_from_slice(&info_bytes(offset, len)?);
    Ok(T::from_bytes(&bytes))
}

/// Bumped when a face's first point (+4) or a scene point below the count may have
/// changed, or the faces or points were taken or given back: what is worked out from them
/// (`race::start_place`'s checkpoint places) is kept until then.
static FIRST_POINTS_CHANGED: AtomicU64 = AtomicU64::new(0);

/// How many times the faces' first points or the scene points have changed (see
/// `FIRST_POINTS_CHANGED`).
pub fn first_points_changes() -> u64 {
    FIRST_POINTS_CHANGED.load(Ordering::Relaxed)
}

/// The faces' first points or the scene points have changed.
pub fn first_points_changed() {
    FIRST_POINTS_CHANGED.fetch_add(1, Ordering::Relaxed);
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
    let f =
        crate::race::level_init::read_course_memory(faces_at, faces as usize, "the course faces")?;
    let r = crate::race::level_init::read_course_memory(
        records_at,
        records as usize,
        "the course records",
    )?;
    FACES.set(&f)?;
    RECORDS.set(&r)?;
    first_points_changed();
    Ok(())
}

/// Given back with the course memory.
pub fn give_back() {
    FACES.give_back();
    RECORDS.give_back();
    first_points_changed();
    INFO.give_back();
}

/// `f` given readers of the faces' and the scene points' dwords by byte (`None` past them).
pub fn with_faces_and_points<R>(
    f: impl FnOnce(&dyn Fn(usize) -> Option<u32>, &dyn Fn(usize) -> Option<u32>) -> R,
) -> Result<R, BadInput> {
    let dword =
        |table: &SharedTable, at: usize| table.array(at).ok().flatten().map(u32::from_le_bytes);
    // Refused without them, as before a read.
    FACES.len()?;
    crate::race::scene_points::POINTS.len()?;
    Ok(f(&|at| dword(&FACES, at), &|at| {
        dword(&crate::race::scene_points::POINTS, at)
    }))
}

/// Whether `at` is one of the course's places (faces, records, scene points).
#[inline]
pub fn is_place(at: u32) -> bool {
    let w = faces_base();
    w != 0 && at.wrapping_sub(w) < PARTS * ROOM
}

/// What the by-number readers and writers take: values of any bits, at most 64 bytes.
pub trait Whole: Copy {
    fn to_bytes(self, to: &mut [u8; 64]);
    /// From the first bytes of `from` (little-endian).
    fn from_bytes(from: &[u8; 64]) -> Self;
}

macro_rules! whole {
    ($($t:ty),*) => {$(
        impl Whole for $t {
            fn to_bytes(self, to: &mut [u8; 64]) {
                to[..size_of::<$t>()].copy_from_slice(&self.to_le_bytes());
            }
            #[inline]
            fn from_bytes(from: &[u8; 64]) -> Self {
                <$t>::from_le_bytes(*from.first_chunk().expect("64 bytes"))
            }
        }
    )*};
}
whole!(u8, u16, u32, i8, i16, i32, f32);

impl<const N: usize> Whole for [u8; N] {
    fn to_bytes(self, to: &mut [u8; 64]) {
        to[..N].copy_from_slice(&self);
    }
    #[inline]
    fn from_bytes(from: &[u8; 64]) -> Self {
        *from.first_chunk().expect("at most 64 bytes")
    }
}

// ---- by number ----
//
// What the original reaches as `g_courseFaces + 0x40 n + offset` (and the records' `+ 0x20 n`):
// a face or record by its number, an entry of the store's table. `COURSE.course_faces` and
// `COURSE.course_records` hold the tables' bases while the course is held; any other value
// (0 before a course, or after its free) has no faces or records, and a read is refused.

const FACE: u32 = 0x40;
const RECORD: u32 = 0x20;

type Table = SharedTable;

/// Where byte `offset` of entry `n` (entries of `size` bytes) lies in its table, the base the
/// game holds being `base`; a field reaching past the entry reads on into the next, as the
/// original's.
#[inline]
fn index(own_base: u32, base: u32, size: u32, n: u32, offset: u32) -> Result<usize, BadInput> {
    if base != own_base || own_base == 0 {
        return Err(BadInput("no course faces or records"));
    }
    (n as usize)
        .checked_mul(size as usize)
        .and_then(|at| at.checked_add(offset as usize))
        .ok_or(BadInput("a read outside the course faces or records"))
}

/// `len` bytes of `table` from `at`.
fn table_bytes(table: &'static Table, at: usize, len: usize) -> Result<Vec<u8>, BadInput> {
    table
        .bytes(at, len)?
        .ok_or(BadInput("a read outside the course faces or records"))
}

/// `data` written into `table` from `at` (a face's first point noted).
fn write_table(table: &'static Table, at: usize, data: &[u8]) -> Result<(), BadInput> {
    let faces = std::ptr::eq(table, std::ptr::from_ref(&FACES));
    // A face's first point (+4 .. +8) changed is noted.
    let mut first_point = false;
    table
        .write_noting(at, data, |k| {
            first_point |= faces && (4..8).contains(&((at + k) % 0x40));
        })?
        .ok_or(BadInput("a write outside the course faces or records"))?;
    if first_point {
        first_points_changed();
    }
    Ok(())
}

/// `T` at `at` of `table`.
#[inline]
fn whole_at<T: Whole>(table: &'static Table, at: usize) -> Result<T, BadInput> {
    let mut bytes = [0u8; 64];
    let len = size_of::<T>();
    table
        .read(at, &mut bytes[..len])?
        .ok_or(BadInput("a read outside the course faces or records"))?;
    Ok(T::from_bytes(&bytes))
}

/// Byte `offset` of face `n` in the faces' table.
#[inline]
fn face_index(course: &Course, n: u32, offset: u32) -> Result<usize, BadInput> {
    let base = course.course.course_faces.get();
    index(faces_base(), base, FACE, n, offset)
}

/// Byte `offset` of record `n` in the records' table.
#[inline]
fn record_index(course: &Course, n: u32, offset: u32) -> Result<usize, BadInput> {
    let base = course.course.course_records.get();
    index(records_base(), base, RECORD, n, offset)
}

/// Face `n`.
#[inline]
pub fn face_number(course: &Course, n: u32) -> Result<Face, BadInput> {
    whole_at::<[u8; 64]>(&FACES, face_index(course, n, 0)?).map(Face)
}

/// Field `offset` of face `n`.
#[inline]
pub fn face_field<T: Whole>(course: &Course, n: u32, offset: u32) -> Result<T, BadInput> {
    whole_at(&FACES, face_index(course, n, offset)?)
}

/// Face `n` changed by `change`.
pub fn update_face_number(
    course: &Course,
    n: u32,
    change: impl FnOnce(&mut Face),
) -> Result<(), BadInput> {
    let mut f = face_number(course, n)?;
    change(&mut f);
    write_face_number(course, n, f)
}

/// `v` written at field `offset` of face `n`.
pub fn write_face_field<T: Whole>(
    course: &Course,
    n: u32,
    offset: u32,
    v: T,
) -> Result<(), BadInput> {
    let mut b = [0; 64];
    v.to_bytes(&mut b);
    write_table(&FACES, face_index(course, n, offset)?, &b[..size_of::<T>()])
}

/// The first `count` faces' bytes.
pub fn faces_table(course: &Course, count: u32) -> Result<Vec<u8>, BadInput> {
    table_bytes(
        &FACES,
        face_index(course, 0, 0)?,
        count as usize * FACE as usize,
    )
}

/// Face `n` set to `face`.
pub fn write_face_number(course: &Course, n: u32, face: Face) -> Result<(), BadInput> {
    write_table(&FACES, face_index(course, n, 0)?, &face.0)
}

/// The first `count` records' bytes.
pub fn records_table(course: &Course, count: u32) -> Result<Vec<u8>, BadInput> {
    table_bytes(
        &RECORDS,
        record_index(course, 0, 0)?,
        count as usize * RECORD as usize,
    )
}

/// The records from record 0 set to `bytes`.
pub fn write_records_table(course: &Course, bytes: &[u8]) -> Result<(), BadInput> {
    write_table(&RECORDS, record_index(course, 0, 0)?, bytes)
}

/// Field `offset` of record `n`.
#[inline]
pub fn record_field<T: Whole>(course: &Course, n: u32, offset: u32) -> Result<T, BadInput> {
    whole_at(&RECORDS, record_index(course, n, offset)?)
}
