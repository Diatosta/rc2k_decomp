//! The scene's points (`g_scenePoints`, a part of the course memory: twelve bytes each,
//! three whole numbers): Rust's since phase 13 (2026-10-27). `race::load_level` moves them
//! out of the course memory once its data is read, with the room after them up to the
//! course info (where the game puts new points past the count: the face corners' copies,
//! the shadow's), and `race::free_course_memory` gives them back.
//!
//! `COURSE.scene_points` is `base()` while they are held (0 otherwise; a place in the range
//! `race::course_places` reserves), so a point is named by `base()` + 12 n where the original held its address: the original's arithmetic is kept,
//! and a place is read here.

use crate::errors::BadInput;
use crate::memory::owned::Owned;
use crate::race::course_store::COURSE;

/// `COURSE.scene_points` while the points are held: point 0's place.
pub fn base() -> u32 {
    crate::race::course_places::points_base()
}

pub static POINTS: Owned<Vec<u8>> = Owned::new("no scene points");

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
    let bytes = crate::pointers::GamePtr::new(from).read_bytes(len as usize, "the scene points")?;
    POINTS.set(bytes);
    Ok(())
}

/// Given back with the course memory.
pub fn give_back() {
    POINTS.give_back();
}

fn range(at: u32, len: usize) -> Option<std::ops::Range<usize>> {
    let start = at.checked_sub(base())? as usize;
    Some(start..start.checked_add(len)?)
}

/// `len` bytes at place `at`; refused outside the points.
pub fn bytes(at: u32, len: usize) -> Result<Vec<u8>, BadInput> {
    POINTS
        .with(|p| p.get(range(at, len)?).map(<[u8]>::to_vec))?
        .ok_or(BadInput("a read outside the scene points"))
}

/// The dword at place `at`.
pub fn word(at: u32) -> Result<u32, BadInput> {
    let b = bytes(at, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// The dword at place `at`; outside the points the game ends (for the readers that
/// cannot carry a refusal, as their reads of game memory did).
pub fn word_or_end(at: u32) -> u32 {
    word(at).unwrap_or_else(|e| crate::fatal::raise::end("a scene point", e))
}

/// The three dwords of the point at place `at`.
pub fn point(at: u32) -> Result<[u32; 3], BadInput> {
    let b = bytes(at, 12)?;
    Ok(std::array::from_fn(|k| {
        u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
    }))
}

/// Point `n`'s place.
pub fn place(n: u32) -> u32 {
    n.wrapping_mul(12).wrapping_add(COURSE.scene_points.get())
}

/// `data` written at place `at`; refused outside the points.
pub fn write(at: u32, data: &[u8]) -> Result<(), BadInput> {
    POINTS
        .with(|p| {
            p.get_mut(range(at, data.len())?)
                .map(|to| to.copy_from_slice(data))
        })?
        .ok_or(BadInput("a write outside the scene points"))
}

/// The dword at place `at` set to `v`.
pub fn put(at: u32, v: u32) -> Result<(), BadInput> {
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
    COURSE.scene_points.set(base());
    let out = f();
    match kept {
        Some(p) => POINTS.set(p),
        None => POINTS.give_back(),
    }
    COURSE.scene_points.set(kept_at);
    out
}
