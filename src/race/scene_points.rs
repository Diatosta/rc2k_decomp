//! The scene's points (`g_scenePoints`, a part of the course memory: twelve bytes each,
//! three whole numbers): Rust's since phase 13 (2026-10-27). `race::load_level` moves them
//! out of the course memory once its data is read, with the room after them up to the
//! course info (where the game puts new points past the count: the face corners' copies,
//! the shadow's), and `race::free_course_memory` gives them back.
//!
//! A point is reached by its number (`point_number`, `point_word`, `put_point_word`,
//! `copy_point`). `COURSE.scene_points` is `base()` while they are held (0 otherwise; a number
//! in the range `race::course_places` reserves, which no host memory backs): the number's
//! byte is 12 n from it, and any other value of `COURSE.scene_points` reads nothing.

use crate::errors::BadInput;
use crate::race::course_store::COURSE;

/// `COURSE.scene_points` while the points are held: point 0's place.
pub fn base() -> u32 {
    crate::race::course_places::points_base()
}

crate::memory::owned::thread_owned!(
    pub static POINTS: Vec<u8> = "no scene points";
);

/// Whether the points are held.
pub fn held() -> bool {
    POINTS.is_taken()
}

/// The points moved out of the course memory: `len` bytes at game address `from`.
pub fn take_from(from: u32, len: u32) -> Result<(), BadInput> {
    crate::race::course_places::reserve()?;
    if len > crate::race::course_places::room() {
        return Err(BadInput("more scene points than their places hold"));
    }
    let bytes = crate::race::level_init::course_memory_at(from)?
        .read_bytes(len as usize, "the scene points")?;
    POINTS.set(bytes);
    crate::race::course_places::first_points_changed();
    Ok(())
}

/// Given back with the course memory.
pub fn give_back() {
    POINTS.give_back();
    crate::race::course_places::first_points_changed();
}

fn range(at: u32, len: usize) -> Option<std::ops::Range<usize>> {
    let start = at.checked_sub(base())? as usize;
    Some(start..start.checked_add(len)?)
}

/// A refused access logged with its place, the points' and the caller (the refusal's
/// text alone does not say which reader it was).
fn outside(
    what: &'static str,
    at: u32,
    len: usize,
    caller: &std::panic::Location<'static>,
) -> BadInput {
    let held = POINTS.with(|p| p.len()).ok();
    crate::framework::log(&format!(
        "scene points: {what} of {len} bytes at {at:#x}; the points at {:#x}, {held:?} bytes; \
         from {}",
        base(),
        caller
    ));
    BadInput(what)
}

/// `len` bytes at place `at`; refused outside the points, but for a read wholly in the
/// twelve bytes before them (point -1, which `race::face_texture_coordinates` reads for a
/// triangle's missing fourth corner: `notes/bugs.md`). The original's course memory had
/// the faces right before the points (`CourseLayout`: records, faces, points), so those
/// bytes are the last face's tail, read from the faces.
#[track_caller]
fn bytes(at: u32, len: usize) -> Result<Vec<u8>, BadInput> {
    let caller = std::panic::Location::caller();
    let before = base().wrapping_sub(at) as usize;
    if at < base() && len <= before && before <= 12 {
        let faces_len = crate::race::course_places::FACES.with(|f| f.len())?;
        let from = faces_len
            .checked_sub(before)
            .ok_or_else(|| outside("a read before the scene points", at, len, caller))?;
        return crate::race::course_places::FACES
            .with(|f| f.get(from..from + len).map(<[u8]>::to_vec))?
            .ok_or_else(|| outside("a read before the scene points", at, len, caller));
    }
    POINTS
        .with(|p| p.get(range(at, len)?).map(<[u8]>::to_vec))?
        .ok_or_else(|| outside("a read outside the scene points", at, len, caller))
}

/// The `N` bytes at place `at`, refused as `bytes`, without a heap copy.
#[track_caller]
#[inline]
fn array<const N: usize>(at: u32) -> Result<[u8; N], BadInput> {
    let caller = std::panic::Location::caller();
    if at < base() {
        let mut to = [0; N];
        to.copy_from_slice(&bytes(at, N)?);
        return Ok(to);
    }
    let start = (at - base()) as usize;
    POINTS
        .with(|p| p.get(start..)?.first_chunk::<N>().copied())?
        .ok_or_else(|| outside("a read outside the scene points", at, N, caller))
}

/// The dword at place `at`.
#[track_caller]
fn word(at: u32) -> Result<u32, BadInput> {
    array(at).map(u32::from_le_bytes)
}

/// The three dwords of the point at place `at`.
#[track_caller]
fn point(at: u32) -> Result<[u32; 3], BadInput> {
    let b: [u8; 12] = array(at)?;
    Ok(std::array::from_fn(|k| {
        u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
    }))
}

/// Point `n`'s place.
fn place(n: u32) -> u32 {
    n.wrapping_mul(12).wrapping_add(COURSE.scene_points.get())
}

/// Point `n`'s x, y and z (point -1 as `bytes` reads it).
#[track_caller]
pub fn point_number(n: u32) -> Result<[u32; 3], BadInput> {
    point(place(n))
}

/// Point `n`'s x and z, read as one point.
#[track_caller]
pub fn point_xz(n: u32) -> Result<(u32, u32), BadInput> {
    point_number(n).map(|[x, _, z]| (x, z))
}

/// Point `from` copied over point `to` (as `bytes` reads `from`).
#[track_caller]
pub fn copy_point(from: u32, to: u32) -> Result<(), BadInput> {
    write(place(to), &bytes(place(from), 12)?)
}

/// Coordinate `k` (0 x, 1 y, 2 z) of point `n`.
#[track_caller]
pub fn point_word(n: u32, k: u32) -> Result<u32, BadInput> {
    word(place(n).wrapping_add(4 * k))
}

/// Coordinate `k` (0 x, 1 y, 2 z) of point `n` set to `v`.
#[track_caller]
pub fn put_point_word(n: u32, k: u32, v: u32) -> Result<(), BadInput> {
    put(place(n).wrapping_add(4 * k), v)
}

/// `data` written at place `at`; refused outside the points.
#[track_caller]
fn write(at: u32, data: &[u8]) -> Result<(), BadInput> {
    let caller = std::panic::Location::caller();
    // The points below the count are the course's own (the ones past it are copies made
    // while drawing): a change to them is noted.
    let counted = (COURSE.scene_point_count.get() as usize).saturating_mul(12);
    POINTS
        .with(|p| {
            let range = range(at, data.len())?;
            let start = range.start;
            let to = p.get_mut(range)?;
            if start < counted && to != data {
                crate::race::course_places::first_points_changed();
            }
            to.copy_from_slice(data);
            Some(())
        })?
        .ok_or_else(|| outside("a write outside the scene points", at, data.len(), caller))
}

/// The dword at place `at` set to `v`.
#[track_caller]
fn put(at: u32, v: u32) -> Result<(), BadInput> {
    write(at, &v.to_le_bytes())
}

/// `f` with the scene points `words` held (`COURSE.scene_points` at `base()`), the points and
/// `COURSE.scene_points` as they were afterwards (the differential tests: the reference
/// reads the same words at their address).
#[cfg(all(test, target_arch = "x86"))]
pub fn with_test_points<R>(words: &[u32], f: impl FnOnce() -> R) -> R {
    let kept = POINTS.with(std::mem::take).ok();
    let kept_at = COURSE.scene_points.get();
    crate::race::course_places::reserve().expect("the course's places");
    POINTS.set(words.iter().flat_map(|w| w.to_le_bytes()).collect());
    crate::race::course_places::first_points_changed();
    COURSE.scene_points.set(base());
    let out = f();
    match kept {
        Some(p) => POINTS.set(p),
        None => POINTS.give_back(),
    }
    crate::race::course_places::first_points_changed();
    COURSE.scene_points.set(kept_at);
    out
}
