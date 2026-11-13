//! The scene's point records (0x40000 bytes, taken by `scene::allocate_point_records`),
//! counted on the list as the game's block was, given back by `race::free_point_records`. 0x2000 records of 0x20 bytes: +0 the point's number, +0xC
//! the x on the screen, +0x10 .. +0x18 the point in the view, +0x1C the y on the screen. A
//! scene point n has record n & 0xFFF, an object's point or normal n record
//! 0x1000 + (n & 0xFFF).
//!
//! A record is named by its number + 1 (1 .. 0x2000), where the original held its
//! address: in the polygon entries' +4. The two other kinds of record the entries name
//! follow: the four quad vertices (`PolygonWork::quad_vertices`, the view's own, `of_quad`)
//! and the clipper's pool (the part's `PolygonWork::record_pool`, `of_pool`; `g_polygonNextRecord`
//! holds the next one's number where the original held its address).
//! `COURSE_LISTS.point_records` is `HELD` while the records are taken, 0 otherwise.

use crate::errors::{BadInput, Refused};
use crate::session::Dword;
use std::sync::atomic::{AtomicBool, Ordering};

/// The records.
pub const COUNT: u32 = 0x2000;
/// The block's bytes, as counted.
pub const BYTES: u32 = 0x4_0000;
/// `COURSE_LISTS.point_records` while the records are taken.
pub const HELD: u32 = 1;

pub type Record = [u32; 8];

/// The records of one part, as cells: read and written many times a polygon.
pub type Records = [[Dword; 8]; COUNT as usize];

/// A part's records, all zero (`workers::PartScratch::records`): a record is the part's own
/// projection, made again by another part that needs it. Every reader below is handed the
/// part's scratch.
pub const fn new_records() -> Records {
    [const { [const { Dword::new(0) }; 8] }; COUNT as usize]
}

/// Whether the records are taken.
static TAKEN: AtomicBool = AtomicBool::new(false);

/// The records made, zeroed, counted on `list`.
pub fn take(
    course: &crate::race::owner_course::Course,
    list: &dyn crate::cells::Store,
) -> Result<(), Refused> {
    crate::memory::owned::count_on(list, BYTES)?;
    crate::workers::all_parts_on_game_thread()
        .flat_map(|part| part.records.iter().flatten())
        .for_each(|cell| cell.set(0));
    TAKEN.store(true, Ordering::Relaxed);
    course.lists.get().point_records.set(HELD);
    Ok(())
}

/// Given back (the list's `free_list` counts them down).
pub fn give_back() {
    TAKEN.store(false, Ordering::Relaxed);
}

/// The first `n` records' cells in every drawing thread's copy (copy 0 first): a clear
/// of the records reaches them all, so no copy keeps a record of an earlier frame. While
/// views are drawn beside each other (`workers::draw_beside`) only `part`'s copy (the
/// drawing part clearing them): the other view's are its own, being drawn.
pub fn first_records_everywhere<'a>(
    part: &'a crate::workers::PartScratch,
    n: usize,
) -> Result<impl Iterator<Item = &'a [Dword; 8]>, BadInput> {
    if !TAKEN.load(Ordering::Relaxed) {
        return Err(BadInput("no point records"));
    }
    if n > COUNT as usize {
        return Err(BadInput("a point record past the block"));
    }
    // Beside each other: the part's own. Otherwise the game's thread draws alone and reaches
    // every part's (no scope is open: no run holds one).
    let copies: [Option<&'a crate::workers::PartScratch>; crate::workers::COUNT] =
        if crate::workers::beside() {
            std::array::from_fn(|k| (k == 0).then_some(part))
        } else {
            let mut all = crate::workers::all_parts_on_game_thread();
            std::array::from_fn(|_| all.next())
        };
    Ok(copies
        .into_iter()
        .flatten()
        .flat_map(move |copy| &copy.records[..n]))
}

/// Whether the records are taken.
pub fn held(lists: &crate::race::course_lists_store::CourseLists) -> bool {
    lists.point_records.get() != 0
}

/// The record at byte `offset` of the block (the original's `records + offset`).
pub fn at_offset(offset: u32) -> u32 {
    (offset >> 5) + 1
}

/// The record of scene point `n` (`n & 0xFFF`).
pub fn of_scene_point(n: u32) -> u32 {
    (n & 0xFFF) + 1
}

/// The record of an object's point or normal `n` (`0x1000 + (n & 0xFFF)`).
pub fn of_object_point(n: u32) -> u32 {
    (n & 0xFFF) + 0x1001
}

/// The quad vertices' first record.
pub const QUAD_FIRST: u32 = COUNT + 1;
/// The quad vertices (`PolygonWork::quad_vertices`, 8 dwords each).
pub const QUADS: u32 = 4;
/// The clipper's pool's first record.
pub const POOL_FIRST: u32 = QUAD_FIRST + QUADS;
/// The pool's records (the part's `PolygonWork::record_pool`, 0x2020 bytes).
pub const POOL: u32 = 0x101;

/// The record of quad vertex `k`.
pub fn of_quad(k: u32) -> u32 {
    QUAD_FIRST + k
}

/// The record of the pool's record `k`.
pub fn of_pool(k: u32) -> u32 {
    POOL_FIRST + k
}

/// The cells of record `r` in `part`'s records, of whichever kind; `quad` is the view's quad
/// vertices (`PolygonWork::quad_vertices`), `None` outside a view's drawing (a quad record is
/// then refused).
#[inline]
fn cells<'a>(
    part: &'a crate::workers::PartScratch,
    r: u32,
    quad: Option<&'a [Dword; 32]>,
) -> Result<&'a [Dword; 8], BadInput> {
    if r > COUNT {
        return far_cells(part, r, quad);
    }
    if !TAKEN.load(Ordering::Relaxed) {
        return Err(BadInput("no point records"));
    }
    (r as usize)
        .checked_sub(1)
        .and_then(|k| part.records.get(k))
        .ok_or(BadInput("a point record past the block"))
}

/// The cells of a quad vertex's or a pool record (`part`'s pool).
fn far_cells<'a>(
    part: &'a crate::workers::PartScratch,
    r: u32,
    quad: Option<&'a [Dword; 32]>,
) -> Result<&'a [Dword; 8], BadInput> {
    let (all, k): (&'a [Dword], u32) = match r {
        _ if (QUAD_FIRST..POOL_FIRST).contains(&r) => (
            quad.ok_or(BadInput("a quad vertex outside a view's drawing"))?,
            r - QUAD_FIRST,
        ),
        _ if (POOL_FIRST..POOL_FIRST + POOL).contains(&r) => {
            (&part.polygon.record_pool, r - POOL_FIRST)
        }
        _ => return Err(BadInput("a point record past the records")),
    };
    all.get(8 * k as usize..8 * k as usize + 8)
        .and_then(|cells| cells.try_into().ok())
        .ok_or(BadInput("a point record past the records"))
}

/// Dword `offset` of a record's cells; refused for an offset past the record.
#[inline]
fn cell(cells: &[Dword; 8], offset: u32) -> Result<&Dword, BadInput> {
    if offset.is_multiple_of(4)
        && let Some(cell) = cells.get(offset as usize / 4)
    {
        return Ok(cell);
    }
    Err(BadInput("a field past a point record"))
}

/// Record `r` of `part`'s records; refused for none or one past the records.
#[inline]
pub fn get(part: &crate::workers::PartScratch, r: u32) -> Result<Record, BadInput> {
    get_in(part, None, r)
}

/// `get` with the view's quad vertices (`PolygonWork::quad_vertices`).
#[inline]
pub fn get_in(
    part: &crate::workers::PartScratch,
    quad: Option<&[Dword; 32]>,
    r: u32,
) -> Result<Record, BadInput> {
    let cells = cells(part, r, quad)?;
    Ok(std::array::from_fn(|k| cells[k].get()))
}

/// Record `r` set to `record`; refused as `get`.
#[inline]
pub fn set(part: &crate::workers::PartScratch, r: u32, record: Record) -> Result<(), BadInput> {
    set_in(part, None, r, record)
}

/// `set` with the view's quad vertices.
#[inline]
pub fn set_in(
    part: &crate::workers::PartScratch,
    quad: Option<&[Dword; 32]>,
    r: u32,
    record: Record,
) -> Result<(), BadInput> {
    for (cell, v) in cells(part, r, quad)?.iter().zip(record) {
        cell.set(v);
    }
    Ok(())
}

/// The dword at `offset` of record `r`; refused for an offset past the record.
#[inline]
pub fn field(part: &crate::workers::PartScratch, r: u32, offset: u32) -> Result<u32, BadInput> {
    field_in(part, None, r, offset)
}

/// `field` with the view's quad vertices.
#[inline]
pub fn field_in(
    part: &crate::workers::PartScratch,
    quad: Option<&[Dword; 32]>,
    r: u32,
    offset: u32,
) -> Result<u32, BadInput> {
    Ok(cell(cells(part, r, quad)?, offset)?.get())
}

/// The dword at `offset` of record `r` set to `v`; refused as `field`.
#[inline]
pub fn put_field(
    part: &crate::workers::PartScratch,
    r: u32,
    offset: u32,
    v: u32,
) -> Result<(), BadInput> {
    put_field_in(part, None, r, offset, v)
}

/// `put_field` with the view's quad vertices.
#[inline]
pub fn put_field_in(
    part: &crate::workers::PartScratch,
    quad: Option<&[Dword; 32]>,
    r: u32,
    offset: u32,
    v: u32,
) -> Result<(), BadInput> {
    cell(cells(part, r, quad)?, offset)?.set(v);
    Ok(())
}

/// The dword at `offset` of a polygon vertex's record. A refusal ends the game, as the
/// original's read of a wrong address went wrong (the polygon code's reads cannot carry
/// one).
#[inline]
pub fn vertex(part: &crate::workers::PartScratch, quad: &[Dword; 32], v: u32, offset: u32) -> u32 {
    field_in(part, Some(quad), v, offset)
        .unwrap_or_else(|e| crate::fatal::raise::end("a point record", e))
}

/// A polygon vertex's whole record (as `vertex`), its bytes.
pub fn vertex_bytes(
    part: &crate::workers::PartScratch,
    quad: &[Dword; 32],
    v: u32,
) -> Result<[u8; 32], BadInput> {
    let record = get_in(part, Some(quad), v)?;
    Ok(std::array::from_fn(|n| record[n / 4].to_le_bytes()[n % 4]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_named_by_number() {
        let course = &crate::race::owner::race().course;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = course.lists.get().point_records.get();
        let memory = crate::memory::ListCells::empty();
        take(course, &memory).unwrap();
        assert_eq!(memory.get().total, BYTES);
        assert_eq!(at_offset(0x20 * 5), of_scene_point(0x1005));
        assert_eq!(of_object_point(3), at_offset(0x20 * 0x1003));
        let quad = [const { Dword::new(0) }; 32];
        let part = crate::workers::game_scratch();
        put_field(part, of_scene_point(7), 0x18, 42).unwrap();
        assert_eq!(vertex(part, &quad, of_scene_point(7), 0x18), 42);
        assert_eq!(get(part, 8).unwrap()[6], 42);
        assert!(get(part, 0).is_err());
        assert!(field(part, 1, 0x20).is_err() && field(part, 1, 2).is_err());
        // after the point records the quad vertices, then the clipper's pool, by their cells
        assert_eq!((of_quad(0), of_pool(0)), (COUNT + 1, COUNT + 1 + QUADS));
        let pool = &part.polygon.record_pool;
        let (kept_quad, kept_pool) = (
            get_in(part, Some(&quad), of_quad(3)).unwrap(),
            get(part, of_pool(POOL - 1)).unwrap(),
        );
        put_field_in(part, Some(&quad), of_quad(3), 0x1C, 7).unwrap();
        put_field(part, of_pool(POOL - 1), 0, 9).unwrap();
        assert_eq!((quad[31].get(), pool[0x800].get()), (7, 9));
        set_in(part, Some(&quad), of_quad(3), kept_quad).unwrap();
        assert!(get(part, of_quad(0)).is_err());
        set(part, of_pool(POOL - 1), kept_pool).unwrap();
        assert!(get(part, of_pool(POOL)).is_err() && get(part, 0x1_0000).is_err());
        give_back();
        course.lists.get().point_records.set(kept);
        crate::memory::with_figures(|f| *f = figures);
    }
}
