//! The stage's course data as one value (`CourseData`: the faces, the records, the scene points and
//! the course info, with the counter of the faces' first-point changes), held by the course owner
//! (`Course::stage`). `race::load_level` takes the parts from the course file and the info files,
//! and the stage's freeing gives them back; both happen on the game thread between frames, while no
//! drawing part reads, so the drawing threads always see one stage's data.
//!
//! The faces (0x40 bytes each) and records (0x20 bytes each) are reached by number
//! (`face_number`, `face_field`, `record_field`, ..; a field past its entry reads on into the
//! next, kept as is), the course info by offset (`info_bytes`), the points through
//! `race::scene_points`.

use crate::errors::BadInput;
use crate::memory::shared_table::SharedTable;
use crate::race::course_places_store::{CourseData, ROOM};
use crate::race::owner_course::Course;
use rc2k_core::race::faces::Face;

/// The place of face 0 in the numbering the scene points use for their reads of the face before
/// point 0 (see `race::scene_points`): the points' places lie `2 * ROOM` after it.
const PLACES_BASE: u32 = 0xD000_0000;

/// Scene point 0's place: the numbering `race::scene_points` reads points by.
pub(super) const fn points_base() -> u32 {
    PLACES_BASE + 2 * ROOM
}

/// The course info taken (`len` bytes, zeroed).
pub(super) fn take_info(data: &CourseData, len: u32) -> Result<(), BadInput> {
    if len > ROOM {
        return Err(BadInput("a course info larger than its room"));
    }
    data.info.set(&vec![0; len as usize])
}

/// Whether the course info is held.
pub fn info_held(data: &CourseData) -> bool {
    data.info.is_taken()
}

/// The whole course info set to `bytes` (the same length as the table taken).
pub(super) fn set_info(data: &CourseData, bytes: &[u8]) -> Result<(), BadInput> {
    data.info
        .write(0, bytes)?
        .ok_or(BadInput("an info larger than the course info"))
}

/// `len` bytes of the course info from byte `offset`.
pub fn info_bytes(data: &CourseData, offset: u32, len: usize) -> Result<Vec<u8>, BadInput> {
    data.info
        .bytes(offset as usize, len)?
        .ok_or(BadInput("a read outside the course info"))
}

/// How many times the faces' first points or the scene points have changed.
pub(super) fn first_points_changes(data: &CourseData) -> u64 {
    data.first_points_changes()
}

/// The faces' first points or the scene points have changed.
pub(super) fn first_points_changed(data: &CourseData) {
    data.first_points_changed();
}

/// Whether the faces and records are held.
pub fn held(data: &CourseData) -> bool {
    data.faces.is_taken()
}

/// The faces and records taken from the course file.
pub fn take(data: &CourseData, faces: &[u8], records: &[u8]) -> Result<(), BadInput> {
    data.faces.set(faces)?;
    data.records.set(records)?;
    first_points_changed(data);
    Ok(())
}

/// Given back with the stage's course list.
pub fn give_back(data: &CourseData) {
    data.faces.give_back();
    data.records.give_back();
    first_points_changed(data);
    data.info.give_back();
}

/// `f` given readers of the faces' and the scene points' dwords by byte (`None` past them).
pub(super) fn with_faces_and_points<R>(
    data: &CourseData,
    f: impl FnOnce(&dyn Fn(usize) -> Option<u32>, &dyn Fn(usize) -> Option<u32>) -> R,
) -> Result<R, BadInput> {
    let dword =
        |table: &SharedTable, at: usize| table.array(at).ok().flatten().map(u32::from_le_bytes);
    // Refused without them.
    data.faces.len()?;
    data.points.len()?;
    Ok(f(&|at| dword(&data.faces, at), &|at| {
        dword(&data.points, at)
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

/// Where byte `offset` of entry `n` (entries of `size` bytes) lies in its table; a field
/// reaching past the entry reads on into the next, kept as is.
#[inline]
fn index(size: u32, n: u32, offset: u32) -> Result<usize, BadInput> {
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

/// `bytes` written into `table` from `at` (a face's first point noted).
fn write_table(data: &CourseData, table: &Table, at: usize, bytes: &[u8]) -> Result<(), BadInput> {
    let faces = std::ptr::eq(table, &raw const data.faces);
    // A change to a face's first point is noted.
    let mut first_point = false;
    table
        .write_noting(at, bytes, |k| {
            first_point |= faces && (4..8).contains(&((at + k) % 0x40));
        })?
        .ok_or(BadInput("a write outside the course faces or records"))?;
    if first_point {
        first_points_changed(data);
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
fn face_index(n: u32, offset: u32) -> Result<usize, BadInput> {
    index(FACE, n, offset)
}

/// Byte `offset` of record `n` in the records' table.
#[inline]
fn record_index(n: u32, offset: u32) -> Result<usize, BadInput> {
    index(RECORD, n, offset)
}

/// Face `n`.
#[inline]
pub fn face_number(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
) -> Result<Face, BadInput> {
    whole_at::<[u8; 64]>(&course.stage.course.faces, face_index(n, 0)?).map(Face)
}

/// Field `offset` of face `n`.
#[inline]
pub fn face_field<T: Whole>(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
    offset: u32,
) -> Result<T, BadInput> {
    whole_at(&course.stage.course.faces, face_index(n, offset)?)
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
    write_table(
        &course.stage.course,
        &course.stage.course.faces,
        face_index(n, offset)?,
        &b[..size_of::<T>()],
    )
}

/// The first `count` faces' bytes.
pub fn faces_table(course: &Course, count: u32) -> Result<Vec<u8>, BadInput> {
    table_bytes(
        &course.stage.course.faces,
        face_index(0, 0)?,
        count as usize * FACE as usize,
    )
}

/// Face `n` set to `face`.
pub(super) fn write_face_number(course: &Course, n: u32, face: Face) -> Result<(), BadInput> {
    let data = &course.stage.course;
    write_table(data, &data.faces, face_index(n, 0)?, &face.0)
}

/// The first `count` records' bytes.
pub(super) fn records_table(
    course: crate::race::owner_course::CourseReads<'_>,
    count: u32,
) -> Result<Vec<u8>, BadInput> {
    table_bytes(
        &course.stage.course.records,
        record_index(0, 0)?,
        count as usize * RECORD as usize,
    )
}

/// Whether the first `count` records are held, refused as `records_table` refuses: their
/// fields can then be read one at a time (`record_field`) with no copy of the table.
pub(super) fn records_held(
    course: crate::race::owner_course::CourseReads<'_>,
    count: u32,
) -> Result<(), BadInput> {
    let end = record_index(count, 0)?;
    if end > course.stage.course.records.len()? {
        return Err(BadInput("a read outside the course faces or records"));
    }
    Ok(())
}

/// The records from record 0 set to `bytes`.
pub(super) fn write_records_table(course: &Course, bytes: &[u8]) -> Result<(), BadInput> {
    let data = &course.stage.course;
    write_table(data, &data.records, record_index(0, 0)?, bytes)
}

/// Field `offset` of record `n`.
#[inline]
pub fn record_field<T: Whole>(
    course: crate::race::owner_course::CourseReads<'_>,
    n: u32,
    offset: u32,
) -> Result<T, BadInput> {
    whole_at(&course.stage.course.records, record_index(n, offset)?)
}
