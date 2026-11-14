//! The course's faces (`course_faces`, 0x40 bytes each) and records (`course_records`, 0x20 bytes
//! each): `race::load_level` takes them from the course file (with the scene points,
//! `race::scene_points`), and `give_back` gives them back.
//!
//! While held, the faces and records are entries of their tables, reached by number
//! (`face_number`, `face_field`, `record_field`, ..; a field past its entry reads on into the
//! next, kept as is) and the course info by offset (`info_bytes`, `info_field`).
//! `COURSE.course_faces` and `COURSE.course_records` hold the tables' bases, handles
//! (`PLACES_TAG`): nothing reads through them.

use crate::errors::BadInput;
use crate::race::owner_course::Course;
use rc2k_core::race::faces::Face;
use std::sync::atomic::{AtomicU64, Ordering};

/// Each part's room; a course file's room (`rc2k_core::race::loading::FIRST_PART`) is smaller.
const ROOM: u32 = 0x100_0000;

/// The handle of face 0 (`COURSE.course_faces` while held); the records' come `ROOM` on, the
/// scene points' `2 ROOM` on (`race::scene_points`), the course info's `3 ROOM` on. A number
/// like the other `*_TAG` handles, above every game address and every other tag.
pub const PLACES_TAG: u32 = 0xD000_0000;

/// Face 0's place (`COURSE.course_faces` while held).
pub const fn faces_base() -> u32 {
    PLACES_TAG
}

/// Record 0's place (`COURSE.course_records` while held).
pub const fn records_base() -> u32 {
    faces_base() + ROOM
}

/// Scene point 0's place (`COURSE.scene_points` while held).
pub const fn points_base() -> u32 {
    faces_base() + 2 * ROOM
}

/// The course info's place (`COURSE_LISTS.course_info` while held).
pub const fn info_base() -> u32 {
    faces_base() + 3 * ROOM
}

/// The most bytes of each part.
pub const fn room() -> u32 {
    ROOM
}

use crate::memory::shared_table::SharedTable;

pub static FACES: SharedTable = SharedTable::new(ROOM as usize, "no course faces");
pub static RECORDS: SharedTable = SharedTable::new(ROOM as usize, "no course records");
/// The course info (`course_info`, 0x400000 bytes: the texture infos `files\a\textinfo`
/// and `files\b\textinfo`, the mark corners `files\m\markinfo` at +0x1C2000): taken with
/// the course memory's info part (`level_init::allocate_course_memory`), filled by
/// `level_init::load_texture_info`.
pub static INFO: SharedTable = SharedTable::new(ROOM as usize, "no course info");

/// The course info taken (`len` bytes, zeroed): its place.
pub fn take_info(len: u32) -> Result<u32, BadInput> {
    if len > ROOM {
        return Err(BadInput("a course info larger than its places"));
    }
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

/// The faces and records taken from the course file.
pub fn take(faces: &[u8], records: &[u8]) -> Result<(), BadInput> {
    FACES.set(faces)?;
    RECORDS.set(records)?;
    first_points_changed();
    Ok(())
}

/// Given back with the stage's course list.
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

const FACE: u32 = 0x40;
const RECORD: u32 = 0x20;

type Table = SharedTable;

/// Where byte `offset` of entry `n` (entries of `size` bytes) lies in its table, the base held
/// being `base`; a field reaching past the entry reads on into the next, kept as is.
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
fn table_bytes(table: &Table, at: usize, len: usize) -> Result<Vec<u8>, BadInput> {
    table
        .bytes(at, len)?
        .ok_or(BadInput("a read outside the course faces or records"))
}

/// `data` written into `table` from `at` (a face's first point noted).
fn write_table(table: &Table, at: usize, data: &[u8]) -> Result<(), BadInput> {
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
fn whole_at<T: Whole>(table: &Table, at: usize) -> Result<T, BadInput> {
    let mut bytes = [0u8; 64];
    let len = size_of::<T>();
    table
        .read(at, &mut bytes[..len])?
        .ok_or(BadInput("a read outside the course faces or records"))?;
    Ok(T::from_bytes(&bytes))
}

/// Byte `offset` of face `n` in the faces' table.
#[inline]
fn face_index(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
    offset: u32,
) -> Result<usize, BadInput> {
    let base = course.course.course_faces.get();
    index(faces_base(), base, FACE, n, offset)
}

/// Byte `offset` of record `n` in the records' table.
#[inline]
fn record_index(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
    offset: u32,
) -> Result<usize, BadInput> {
    let base = course.course.course_records.get();
    index(records_base(), base, RECORD, n, offset)
}

/// Face `n`.
#[inline]
pub fn face_number(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
) -> Result<Face, BadInput> {
    whole_at::<[u8; 64]>(&FACES, face_index(course, n, 0)?).map(Face)
}

/// Field `offset` of face `n`.
#[inline]
pub fn face_field<T: Whole>(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
    offset: u32,
) -> Result<T, BadInput> {
    whole_at(&FACES, face_index(course, n, offset)?)
}

/// Face `n` changed by `change`.
pub fn update_face_number(
    course: &Course,
    n: u32,
    change: impl FnOnce(&mut Face),
) -> Result<(), BadInput> {
    let mut f = face_number(course.reads(), n)?;
    change(&mut f);
    write_face_number(course, n, f)
}

/// `v` written at field `offset` of face `n`.
pub fn write_face_field<T: Whole>(
    course: crate::race::owner_course::CourseReads<'_>,
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
        face_index(course.reads(), 0, 0)?,
        count as usize * FACE as usize,
    )
}

/// Face `n` set to `face`.
pub fn write_face_number(course: &Course, n: u32, face: Face) -> Result<(), BadInput> {
    write_table(&FACES, face_index(course.reads(), n, 0)?, &face.0)
}

/// The first `count` records' bytes.
pub fn records_table(
    course: crate::race::owner_course::CourseReads<'_>,
    count: u32,
) -> Result<Vec<u8>, BadInput> {
    table_bytes(
        &RECORDS,
        record_index(course, 0, 0)?,
        count as usize * RECORD as usize,
    )
}

/// The records from record 0 set to `bytes`.
pub fn write_records_table(course: &Course, bytes: &[u8]) -> Result<(), BadInput> {
    write_table(&RECORDS, record_index(course.reads(), 0, 0)?, bytes)
}

/// Field `offset` of record `n`.
#[inline]
pub fn record_field<T: Whole>(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
    offset: u32,
) -> Result<T, BadInput> {
    whole_at(&RECORDS, record_index(course, n, offset)?)
}
