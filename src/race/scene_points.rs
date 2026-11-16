//! The scene's points (`scene_points`: twelve bytes each, three whole numbers). `race::load_level`
//! takes them from the course file, with the room after them up to the file's room (where new
//! points past the count go: the face corners' copies, the shadow's), and the stage's freeing gives
//! them back.
//!
//! A point is reached by its number (`point_number`, `point_word`, `put_point_word`, `copy_point`):
//! the number's byte is 12 n. The points are a part of the stage's `CourseData`.
//!
//! The points past the count are scratch: each drawing part (`scene::workers::PartScratch`) has its
//! own while it is drawn in a run of a list drawn by several parts, taken from the shared ones
//! as they are when it first reaches them in the run (`OwnPoints`). A reader names the own
//! points it reads through; outside the parts that is `no_own()`, the shared points throughout.

use crate::errors::BadInput;
use crate::race::course_places::CourseData;
use crate::race::owner_course::CourseReads;
use std::cell::{Cell, RefCell};

/// Point 0's place: the numbering the readers reach points by (a point's place is `base()` +
/// 12 n; the twelve bytes before it are the last face's tail).
fn base() -> u32 {
    crate::race::course_places::points_base()
}

/// The scene points' table.
fn table(tables: &CourseData) -> &crate::memory::shared_table::SharedTable {
    &tables.points
}

/// A drawing part's own points past the count (`scene::workers::PartScratch::points`): kept from the
/// count on while the part is drawn in a run of a list drawn by several parts (the game thread's
/// first part too), none otherwise (see the header). Plain cells: only the thread that draws the
/// part reaches it (`memory::part_cell::PartCell`).
pub struct OwnPoints {
    /// The byte from which the points are these own ones while the part is being drawn in a
    /// run (the count's), `usize::MAX` outside one.
    from: Cell<usize>,
    /// Its points from the count on, as far as it has reached them in the run.
    tail: RefCell<Vec<u8>>,
}

impl OwnPoints {
    pub const fn new() -> Self {
        Self {
            from: Cell::new(usize::MAX),
            tail: RefCell::new(Vec::new()),
        }
    }

    /// A run begins over `point_count` points: the part's own points forgotten, its first
    /// reach takes them from the shared ones.
    pub fn begin_run(&self, point_count: u32) {
        self.tail.borrow_mut().clear();
        self.from.set((point_count as usize).saturating_mul(12));
    }

    /// The run is over: the shared points read again.
    pub fn end_run(&self) {
        self.from.set(usize::MAX);
    }

    /// The byte from which the points are these own ones: the count's in a run, none
    /// otherwise.
    fn own_from(&self) -> usize {
        self.from.get()
    }
}

impl Default for OwnPoints {
    fn default() -> Self {
        Self::new()
    }
}

/// No own points: the shared ones throughout, what everything outside the drawing parts
/// reads (the physics, the map, the best line, the debug pages). An `OwnPoints` that never
/// begins a run (so reads only the shared points); each reader makes its own, which holds no
/// allocation until something is written through it.
pub const fn no_own() -> OwnPoints {
    OwnPoints::new()
}

/// `f` on `points`' own bytes `from .. from + len` (`own` being their `own_from`, `from` at
/// least `own`); `None` past the points.
fn with_own<R>(
    tables: &CourseData,
    points: &OwnPoints,
    own: usize,
    from: usize,
    len: usize,
    f: impl FnOnce(&mut [u8]) -> R,
) -> Result<Option<R>, BadInput> {
    let all = table(tables).len()?;
    let Some(end) = from.checked_add(len).filter(|&end| end <= all) else {
        return Ok(None);
    };
    let (start, end) = (from - own, end - own);
    let mut tail = points.tail.borrow_mut();
    if tail.len() < end {
        let had = tail.len();
        tail.resize(end, 0);
        if table(tables).read(own + had, &mut tail[had..])?.is_none() {
            return Ok(None);
        }
    }
    Ok(Some(f(&mut tail[start..end])))
}

/// `into`'s length of the points' bytes from byte `start` (through `points`); `None` past
/// the points.
fn read_at(
    tables: &CourseData,
    points: &OwnPoints,
    start: usize,
    into: &mut [u8],
) -> Result<Option<()>, BadInput> {
    let own = points.own_from();
    let end = start.saturating_add(into.len());
    if end <= own {
        return table(tables).read(start, into);
    }
    let split = own.saturating_sub(start).min(into.len());
    let (shared, mine) = into.split_at_mut(split);
    if table(tables).read(start, shared)?.is_none() {
        return Ok(None);
    }
    with_own(tables, points, own, start + split, mine.len(), |t| {
        mine.copy_from_slice(t);
    })
}

/// `data` written into the points from byte `start` (through `points`); `None` (nothing
/// written) past them.
fn write_at(
    tables: &CourseData,
    points: &OwnPoints,
    start: usize,
    data: &[u8],
) -> Result<Option<()>, BadInput> {
    let own = points.own_from();
    let end = start.saturating_add(data.len());
    if end <= own {
        return table(tables).write(start, data);
    }
    if end > table(tables).len()? {
        return Ok(None);
    }
    let split = own.saturating_sub(start).min(data.len());
    let (shared, mine) = data.split_at(split);
    if table(tables).write(start, shared)?.is_none() {
        return Ok(None);
    }
    with_own(tables, points, own, start + split, mine.len(), |t| {
        t.copy_from_slice(mine);
    })
}

/// Whether the points are held.
pub fn held(tables: &CourseData) -> bool {
    table(tables).is_taken()
}

/// The points taken from the course file: `bytes` (the points and the room after them).
pub fn take(tables: &CourseData, bytes: &[u8]) -> Result<(), BadInput> {
    table(tables).set(bytes)?;
    crate::race::course_places::first_points_changed(tables);
    Ok(())
}

/// Given back with the stage's course list.
pub fn give_back(tables: &CourseData) {
    table(tables).give_back();
    crate::race::course_places::first_points_changed(tables);
}

fn range(at: u32, len: usize) -> Option<std::ops::Range<usize>> {
    let start = at.checked_sub(base())? as usize;
    Some(start..start.checked_add(len)?)
}

/// A refused access logged with its place, the points' and the caller (the refusal's
/// text alone does not say which reader it was).
fn outside(
    tables: &CourseData,
    what: &'static str,
    at: u32,
    len: usize,
    caller: &std::panic::Location<'static>,
) -> BadInput {
    let held = table(tables).len().ok();
    crate::logfile::program_log::log(&format!(
        "scene points: {what} of {len} bytes at {at:#x}; the points at {:#x}, {held:?} bytes; \
         from {}",
        base(),
        caller
    ));
    BadInput(what)
}

/// `len` bytes at place `at`; refused outside the points, but for a read wholly in the twelve bytes
/// before them (point -1, which `race::face_texture_coordinates` reads for a triangle's missing
/// fourth corner). The faces lie right before the points in the course memory (`CourseLayout`:
/// records, faces, points), so those bytes are the last face's tail, read from the faces.
#[track_caller]
fn bytes(
    tables: &CourseData,
    points: &OwnPoints,
    at: u32,
    len: usize,
) -> Result<Vec<u8>, BadInput> {
    let caller = std::panic::Location::caller();
    let before = base().wrapping_sub(at) as usize;
    if at < base() && len <= before && before <= 12 {
        let faces_len = tables.faces.len()?;
        let from = faces_len
            .checked_sub(before)
            .ok_or_else(|| outside(tables, "a read before the scene points", at, len, caller))?;
        return tables
            .faces
            .bytes(from, len)?
            .ok_or_else(|| outside(tables, "a read before the scene points", at, len, caller));
    }
    let mut to = vec![0; len];
    range(at, len)
        .map(|r| read_at(tables, points, r.start, &mut to))
        .transpose()?
        .flatten()
        .map(|()| to)
        .ok_or_else(|| outside(tables, "a read outside the scene points", at, len, caller))
}

/// The `N` bytes at place `at`, refused as `bytes`, without a heap copy.
#[track_caller]
#[inline]
fn array<const N: usize>(
    tables: &CourseData,
    points: &OwnPoints,
    at: u32,
) -> Result<[u8; N], BadInput> {
    let caller = std::panic::Location::caller();
    if at < base() {
        let mut to = [0; N];
        to.copy_from_slice(&bytes(tables, points, at, N)?);
        return Ok(to);
    }
    let start = (at - base()) as usize;
    let mut to = [0; N];
    read_at(tables, points, start, &mut to)?
        .map(|()| to)
        .ok_or_else(|| outside(tables, "a read outside the scene points", at, N, caller))
}

/// The dword at place `at`.
#[track_caller]
fn word(tables: &CourseData, points: &OwnPoints, at: u32) -> Result<u32, BadInput> {
    array(tables, points, at).map(u32::from_le_bytes)
}

/// The three dwords of the point at place `at`.
#[track_caller]
fn point(tables: &CourseData, points: &OwnPoints, at: u32) -> Result<[u32; 3], BadInput> {
    let b: [u8; 12] = array(tables, points, at)?;
    Ok(std::array::from_fn(|k| {
        u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
    }))
}

/// Point `n`'s place.
fn place(n: u32) -> u32 {
    n.wrapping_mul(12).wrapping_add(base())
}

/// Point `n`'s x, y and z (point -1 as `bytes` reads it), through `points`.
#[track_caller]
pub fn point_number(
    course: CourseReads<'_>,
    points: &OwnPoints,
    n: u32,
) -> Result<[u32; 3], BadInput> {
    point(&course.stage.course, points, place(n))
}

/// Point `n`'s x and z, read as one point.
#[track_caller]
pub fn point_xz(
    course: CourseReads<'_>,
    points: &OwnPoints,
    n: u32,
) -> Result<(u32, u32), BadInput> {
    point_number(course, points, n).map(|[x, _, z]| (x, z))
}

/// Point `from` copied over point `to` (as `bytes` reads `from`).
#[track_caller]
pub fn copy_point(
    course: CourseReads<'_>,
    points: &OwnPoints,
    from: u32,
    to: u32,
) -> Result<(), BadInput> {
    write(
        course,
        points,
        place(to),
        &bytes(&course.stage.course, points, place(from), 12)?,
    )
}

/// Coordinate `k` (0 x, 1 y, 2 z) of point `n`.
#[track_caller]
pub fn point_word(
    course: CourseReads<'_>,
    points: &OwnPoints,
    n: u32,
    k: u32,
) -> Result<u32, BadInput> {
    word(&course.stage.course, points, place(n).wrapping_add(4 * k))
}

/// Coordinate `k` (0 x, 1 y, 2 z) of point `n` set to `v`.
#[track_caller]
pub fn put_point_word(
    course: CourseReads<'_>,
    points: &OwnPoints,
    n: u32,
    k: u32,
    v: u32,
) -> Result<(), BadInput> {
    put(course, points, place(n).wrapping_add(4 * k), v)
}

/// `data` written at place `at`; refused outside the points.
#[track_caller]
fn write(
    course: CourseReads<'_>,
    points: &OwnPoints,
    at: u32,
    data: &[u8],
) -> Result<(), BadInput> {
    let caller = std::panic::Location::caller();
    // The points below the count are the course's own (the ones past it are copies made
    // while drawing): a change to them is noted.
    let counted = (course.course.scene_point_count.get() as usize).saturating_mul(12);
    let tables = &course.stage.course;
    let refused = || {
        outside(
            tables,
            "a write outside the scene points",
            at,
            data.len(),
            caller,
        )
    };
    let start = range(at, data.len()).ok_or_else(refused)?.start;
    if start < counted {
        let mut was = vec![0; data.len()];
        if read_at(tables, points, start, &mut was)?.is_some() && was != data {
            crate::race::course_places::first_points_changed(tables);
        }
    }
    write_at(tables, points, start, data)?.ok_or_else(refused)
}

/// The dword at place `at` set to `v`.
#[track_caller]
fn put(course: CourseReads<'_>, points: &OwnPoints, at: u32, v: u32) -> Result<(), BadInput> {
    write(course, points, at, &v.to_le_bytes())
}

/// `f` with the scene points `words` held, the points as they were afterwards (the differential
/// tests: the reference reads the same words at their address).
#[cfg(test)]
pub fn with_test_points<R>(tables: &CourseData, words: &[u32], f: impl FnOnce() -> R) -> R {
    let kept = table(tables).all().ok();
    table(tables)
        .set(
            &words
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect::<Vec<u8>>(),
        )
        .expect("test points in their room");
    crate::race::course_places::first_points_changed(tables);
    let out = f();
    match kept {
        Some(p) => table(tables).set(&p).expect("the points as they were"),
        None => table(tables).give_back(),
    }
    crate::race::course_places::first_points_changed(tables);
    out
}
