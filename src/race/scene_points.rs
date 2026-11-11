//! The scene's points (`g_scenePoints`, a part of the course memory: twelve bytes each,
//! three whole numbers): Rust's. `race::load_level` moves them
//! out of the course memory once its data is read, with the room after them up to the
//! course info (where the game puts new points past the count: the face corners' copies,
//! the shadow's), and `race::free_course_memory` gives them back.
//!
//! A point is reached by its number (`point_number`, `point_word`, `put_point_word`,
//! `copy_point`). `COURSE.scene_points` is `base()` while they are held (0 otherwise; a handle of `race::course_places`, which no host memory backs): the number's
//! byte is 12 n from it, and any other value of `COURSE.scene_points` reads nothing.
//!
//! The points past the count are scratch: each drawing thread (`workers`) has its own,
//! taken from the shared ones as they are when it first reaches them in a run (`TAIL`).

use crate::errors::BadInput;
use crate::race::owner::race;
use crate::race::owner_course::Course;

/// `COURSE.scene_points` while the points are held: point 0's place.
pub fn base() -> u32 {
    crate::race::course_places::points_base()
}

/// The points' most bytes: their part of the course's places.
pub static POINTS: crate::memory::shared_table::SharedTable =
    crate::memory::shared_table::SharedTable::new(
        crate::race::course_places::room() as usize,
        "no scene points",
    );

thread_local! {
    /// A drawing thread's own points past the count, from the count on (see the header).
    static TAIL: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The byte from which the running thread's points are its own: the count's in a run of a
/// list drawn by several threads (the game's thread's run too), none otherwise.
fn own_from() -> usize {
    if !crate::workers::in_run() {
        return usize::MAX;
    }
    (race().course.course.scene_point_count.get() as usize).saturating_mul(12)
}

/// A drawing thread's own points forgotten: its next run takes them from the shared ones.
pub fn forget_own() {
    TAIL.with_borrow_mut(Vec::clear);
}

/// `f` on the running thread's own bytes `from .. from + len` (`own` being `own_from`,
/// `from` at least `own`); `None` past the points.
fn with_own<R>(
    own: usize,
    from: usize,
    len: usize,
    f: impl FnOnce(&mut [u8]) -> R,
) -> Result<Option<R>, BadInput> {
    let all = POINTS.len()?;
    let Some(end) = from.checked_add(len).filter(|&end| end <= all) else {
        return Ok(None);
    };
    let (start, end) = (from - own, end - own);
    TAIL.with_borrow_mut(|tail| {
        if tail.len() < end {
            let had = tail.len();
            tail.resize(end, 0);
            if POINTS.read(own + had, &mut tail[had..])?.is_none() {
                return Ok(None);
            }
        }
        Ok(Some(f(&mut tail[start..end])))
    })
}

/// `into`'s length of the points' bytes from byte `start`; `None` past the points.
fn read_at(start: usize, into: &mut [u8]) -> Result<Option<()>, BadInput> {
    let own = own_from();
    let end = start.saturating_add(into.len());
    if end <= own {
        return POINTS.read(start, into);
    }
    let split = own.saturating_sub(start).min(into.len());
    let (shared, mine) = into.split_at_mut(split);
    if POINTS.read(start, shared)?.is_none() {
        return Ok(None);
    }
    with_own(own, start + split, mine.len(), |t| mine.copy_from_slice(t))
}

/// `data` written into the points from byte `start`; `None` (nothing written) past them.
fn write_at(start: usize, data: &[u8]) -> Result<Option<()>, BadInput> {
    let own = own_from();
    let end = start.saturating_add(data.len());
    if end <= own {
        return POINTS.write(start, data);
    }
    if end > POINTS.len()? {
        return Ok(None);
    }
    let split = own.saturating_sub(start).min(data.len());
    let (shared, mine) = data.split_at(split);
    if POINTS.write(start, shared)?.is_none() {
        return Ok(None);
    }
    with_own(own, start + split, mine.len(), |t| t.copy_from_slice(mine))
}

/// Whether the points are held.
pub fn held() -> bool {
    POINTS.is_taken()
}

/// The points moved out of the course memory: `len` bytes at game address `from`.
pub fn take_from(from: u32, len: u32) -> Result<(), BadInput> {
    if len > crate::race::course_places::room() {
        return Err(BadInput("more scene points than their places hold"));
    }
    let bytes =
        crate::race::level_init::read_course_memory(from, len as usize, "the scene points")?;
    POINTS.set(&bytes)?;
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
    let held = POINTS.len().ok();
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
        let faces_len = crate::race::course_places::FACES.len()?;
        let from = faces_len
            .checked_sub(before)
            .ok_or_else(|| outside("a read before the scene points", at, len, caller))?;
        return crate::race::course_places::FACES
            .bytes(from, len)?
            .ok_or_else(|| outside("a read before the scene points", at, len, caller));
    }
    let mut to = vec![0; len];
    range(at, len)
        .map(|r| read_at(r.start, &mut to))
        .transpose()?
        .flatten()
        .map(|()| to)
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
    let mut to = [0; N];
    read_at(start, &mut to)?
        .map(|()| to)
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
fn place(course: &Course, n: u32) -> u32 {
    n.wrapping_mul(12)
        .wrapping_add(course.course.scene_points.get())
}

/// Point `n`'s x, y and z (point -1 as `bytes` reads it).
#[track_caller]
pub fn point_number(course: &Course, n: u32) -> Result<[u32; 3], BadInput> {
    point(place(course, n))
}

/// Point `n`'s x and z, read as one point.
#[track_caller]
pub fn point_xz(course: &Course, n: u32) -> Result<(u32, u32), BadInput> {
    point_number(course, n).map(|[x, _, z]| (x, z))
}

/// Point `from` copied over point `to` (as `bytes` reads `from`).
#[track_caller]
pub fn copy_point(course: &Course, from: u32, to: u32) -> Result<(), BadInput> {
    write(course, place(course, to), &bytes(place(course, from), 12)?)
}

/// Coordinate `k` (0 x, 1 y, 2 z) of point `n`.
#[track_caller]
pub fn point_word(course: &Course, n: u32, k: u32) -> Result<u32, BadInput> {
    word(place(course, n).wrapping_add(4 * k))
}

/// Coordinate `k` (0 x, 1 y, 2 z) of point `n` set to `v`.
#[track_caller]
pub fn put_point_word(course: &Course, n: u32, k: u32, v: u32) -> Result<(), BadInput> {
    put(course, place(course, n).wrapping_add(4 * k), v)
}

/// `data` written at place `at`; refused outside the points.
#[track_caller]
fn write(course: &Course, at: u32, data: &[u8]) -> Result<(), BadInput> {
    let caller = std::panic::Location::caller();
    // The points below the count are the course's own (the ones past it are copies made
    // while drawing): a change to them is noted.
    let counted = (course.course.scene_point_count.get() as usize).saturating_mul(12);
    let refused = || outside("a write outside the scene points", at, data.len(), caller);
    let start = range(at, data.len()).ok_or_else(refused)?.start;
    if start < counted {
        let mut was = vec![0; data.len()];
        if read_at(start, &mut was)?.is_some() && was != data {
            crate::race::course_places::first_points_changed();
        }
    }
    write_at(start, data)?.ok_or_else(refused)
}

/// The dword at place `at` set to `v`.
#[track_caller]
fn put(course: &Course, at: u32, v: u32) -> Result<(), BadInput> {
    write(course, at, &v.to_le_bytes())
}

/// `f` with the scene points `words` held (`COURSE.scene_points` at `base()`), the points and
/// `COURSE.scene_points` as they were afterwards (the differential tests: the reference
/// reads the same words at their address).
#[cfg(all(test, target_arch = "x86"))]
pub fn with_test_points<R>(words: &[u32], f: impl FnOnce() -> R) -> R {
    let course = &race().course;
    let kept = POINTS.all().ok();
    let kept_at = course.course.scene_points.get();
    POINTS
        .set(
            &words
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect::<Vec<u8>>(),
        )
        .expect("test points in their room");
    crate::race::course_places::first_points_changed();
    course.course.scene_points.set(base());
    let out = f();
    match kept {
        Some(p) => POINTS.set(&p).expect("the points as they were"),
        None => POINTS.give_back(),
    }
    crate::race::course_places::first_points_changed();
    course.course.scene_points.set(kept_at);
    out
}
