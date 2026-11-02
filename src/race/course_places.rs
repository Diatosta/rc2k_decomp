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
use crate::pointers::GamePtr;
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
    let reserved =
        crate::os::reserve(len).ok_or(BadInput("no addresses for the course's places"))?;
    let at = crate::address_map::map(reserved, len);
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

crate::memory::owned::thread_owned!(
    pub static FACES: Vec<u8> = "no course faces";
);
crate::memory::owned::thread_owned!(
    pub static RECORDS: Vec<u8> = "no course records";
);
crate::memory::owned::thread_owned!(
    /// The course info (`g_courseInfo`, 0x400000 bytes: the texture infos `files\a\textinfo`
    /// and `files\b\textinfo`, the mark corners `files\m\markinfo` at +0x1C2000): taken with
    /// the course memory (`level_init::allocate_course_memory`), filled by
    /// `level_init::load_texture_info`.
    pub static INFO: Vec<u8> = "no course info";
);

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

/// Whether writing `new` over `old` at byte `off` of the faces changes a face's first point
/// (+4 .. +8).
fn first_point_written(off: usize, old: &[u8], new: &[u8]) -> bool {
    old.iter()
        .zip(new)
        .enumerate()
        .any(|(k, (a, b))| a != b && (4..8).contains(&((off + k) % 0x40)))
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

/// `f` on the faces' and the scene points' bytes.
pub fn with_faces_and_points<R>(f: impl FnOnce(&[u8], &[u8]) -> R) -> Result<R, BadInput> {
    FACES.with(|faces| crate::race::scene_points::POINTS.with(|points| f(faces, points)))?
}

/// Whether `at` is one of the course's places (faces, records, scene points).
#[inline]
pub fn is_place(at: u32) -> bool {
    let w = faces_base();
    w != 0 && at.wrapping_sub(w) < PARTS * ROOM
}

/// The store and offset of place `at`, if it is one of the faces' or records'.
fn store(at: u32) -> Option<(&'static crate::memory::owned::ThreadOwned<Vec<u8>>, usize)> {
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
        let faces = std::ptr::eq(s, std::ptr::from_ref(&FACES));
        return s
            .with(|b| {
                let to = b.get_mut(off..off.checked_add(data.len())?)?;
                if faces && first_point_written(off, to, data) {
                    first_points_changed();
                }
                to.copy_from_slice(data);
                Some(())
            })?
            .ok_or(BadInput("a write outside the course faces or records"));
    }
    if scene_point(at) {
        return crate::race::scene_points::write(at, data);
    }
    GamePtr::new(at).write_bytes(data, what)
}

/// A `T` (a whole number) at `at`.
pub fn read<T: Whole>(at: u32, what: &str) -> Result<T, BadInput> {
    T::read_from(at, what)
}

/// What `read` and `write` take: values of any bits, at most 64 bytes.
pub trait Whole: Copy {
    /// Read at `at`, through a buffer of just its size: these reads are the physics' every tick.
    fn read_from(at: u32, what: &str) -> Result<Self, BadInput>;
    fn to_bytes(self, to: &mut [u8; 64]);
    /// From the first bytes of `from` (little-endian).
    fn from_bytes(from: &[u8; 64]) -> Self;
}

macro_rules! whole {
    ($($t:ty),*) => {$(
        impl Whole for $t {
            #[inline]
            fn read_from(at: u32, what: &str) -> Result<Self, BadInput> {
                array(at, what).map(<$t>::from_le_bytes)
            }
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
    #[inline]
    fn read_from(at: u32, what: &str) -> Result<Self, BadInput> {
        array(at, what)
    }
    fn to_bytes(self, to: &mut [u8; 64]) {
        to[..N].copy_from_slice(&self);
    }
    #[inline]
    fn from_bytes(from: &[u8; 64]) -> Self {
        *from.first_chunk().expect("at most 64 bytes")
    }
}

/// As `read`; a refusal ends the game (for the readers that cannot carry one).
pub fn read_or_end<T: Whole>(at: u32, what: &str) -> T {
    read(at, what).unwrap_or_else(|e| crate::fatal::raise::end(what, e))
}

/// `v` (a whole number) written at `at`.
pub fn write<T: Whole>(at: u32, v: T, what: &str) -> Result<(), BadInput> {
    let mut b = [0; 64];
    v.to_bytes(&mut b);
    write_bytes(at, &b[..size_of::<T>()], what)
}

/// The `N` bytes at `at` (as `bytes` names it), with no heap copy and no copy of unknown
/// length where the place is a store's: these reads are the physics' every tick.
#[inline]
fn array<const N: usize>(at: u32, what: &str) -> Result<[u8; N], BadInput> {
    if let Some((s, off)) = store(at) {
        return s
            .with(|b| b.get(off..)?.first_chunk::<N>().copied())?
            .ok_or(BadInput("a read outside the course faces or records"));
    }
    if scene_point(at) {
        return crate::race::scene_points::array(at);
    }
    let mut to = [0; N];
    to.copy_from_slice(&GamePtr::new(at).read_bytes(N, what)?);
    Ok(to)
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

// ---- by number ----
//
// What the original reaches as `g_courseFaces + 0x40 n + offset` (and the records' `+ 0x20 n`):
// a face or record by its number. While the store holds the course (`COURSE.course_faces` is
// its base) they are its table's entries; before that, the course memory's bytes at the
// original's address.

const FACE: u32 = 0x40;
const RECORD: u32 = 0x20;

/// `T` at byte `offset` of entry `n` of `table` (entries of `size` bytes, the base the game
/// holds `base`); a field reaching past the entry reads on into the next, as the original's.
#[inline]
fn field_of<T: Whole>(
    table: &'static crate::memory::owned::ThreadOwned<Vec<u8>>,
    own_base: u32,
    base: u32,
    size: u32,
    n: u32,
    offset: u32,
    what: &str,
) -> Result<T, BadInput> {
    let place = base.wrapping_add(n.wrapping_mul(size)).wrapping_add(offset);
    if base != own_base || own_base == 0 {
        return read(place, what);
    }
    let at = (n as usize)
        .checked_mul(size as usize)
        .and_then(|at| at.checked_add(offset as usize));
    let mut bytes = [0u8; 64];
    let len = size_of::<T>();
    table
        .with(|b| {
            let from = b.get(at?..at? + len)?;
            bytes[..len].copy_from_slice(from);
            Some(())
        })?
        .ok_or(BadInput("a read outside the course faces or records"))?;
    Ok(T::from_bytes(&bytes))
}

/// Face `n`.
#[inline]
pub fn face_number(n: u32, what: &str) -> Result<Face, BadInput> {
    face_field::<[u8; 64]>(n, 0, what).map(Face)
}

/// Field `offset` of face `n`.
#[inline]
pub fn face_field<T: Whole>(n: u32, offset: u32, what: &str) -> Result<T, BadInput> {
    let base = crate::race::course_store::COURSE.course_faces.get();
    field_of(&FACES, faces_base(), base, FACE, n, offset, what)
}

/// Face `n` changed by `change`.
pub fn update_face_number(
    n: u32,
    what: &str,
    change: impl FnOnce(&mut Face),
) -> Result<(), BadInput> {
    update_face(face_place(n), what, change)
}

/// `v` written at field `offset` of face `n`.
pub fn write_face_field<T: Whole>(n: u32, offset: u32, v: T, what: &str) -> Result<(), BadInput> {
    write(face_place(n).wrapping_add(offset), v, what)
}

/// Face `n`'s place (`g_courseFaces + 0x40 n`), for the helpers that still take one.
pub fn face_place(n: u32) -> u32 {
    let base = crate::race::course_store::COURSE.course_faces.get();
    base.wrapping_add(n.wrapping_mul(FACE))
}

/// The first `count` faces' bytes.
pub fn faces_table(count: u32, what: &str) -> Result<Vec<u8>, BadInput> {
    bytes(face_place(0), count as usize * FACE as usize, what)
}

/// Face `n` set to `face`.
pub fn write_face_number(n: u32, face: Face, what: &str) -> Result<(), BadInput> {
    write_bytes(face_place(n), &face.0, what)
}

/// The first `count` records' bytes.
pub fn records_table(count: u32, what: &str) -> Result<Vec<u8>, BadInput> {
    bytes(record_place(0), count as usize * RECORD as usize, what)
}

/// The records from record 0 set to `bytes`.
pub fn write_records_table(bytes: &[u8], what: &str) -> Result<(), BadInput> {
    write_bytes(record_place(0), bytes, what)
}

/// Record `n`'s place (`dword_5CCCB8 + 0x20 n`).
fn record_place(n: u32) -> u32 {
    let base = crate::race::course_store::COURSE.course_records.get();
    base.wrapping_add(n.wrapping_mul(RECORD))
}

/// Field `offset` of record `n`.
#[inline]
pub fn record_field<T: Whole>(n: u32, offset: u32, what: &str) -> Result<T, BadInput> {
    let base = crate::race::course_store::COURSE.course_records.get();
    field_of(&RECORDS, records_base(), base, RECORD, n, offset, what)
}
