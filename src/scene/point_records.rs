//! The scene's point records (`g_pointRecords`, 0x40000 bytes of the game's heap on
//! `stru_5C6E50`, taken by `scene::allocate_point_records`): Rust's since phase 13
//! (2026-10-27), counted on the list as the game's block was, given back by
//! `race::free_point_records`. 0x2000 records of 0x20 bytes: +0 the point's number, +0xC
//! the x on the screen, +0x10 .. +0x18 the point in the view, +0x1C the y on the screen. A
//! scene point n has record n & 0xFFF, an object's point or normal n record
//! 0x1000 + (n & 0xFFF).
//!
//! A record is named by its number + 1 (1 .. 0x2000), where the original held its
//! address: in the polygon entries' +4. The two other kinds of record the entries name
//! follow (2026-10-29): the four quad vertices (`PolygonWork::quad_vertices`, the view's own, `of_quad`)
//! and the clipper's pool (`POLYGON_WORK.record_pool`, `of_pool`; `g_polygonNextRecord`
//! holds the next one's number where the original held its address).
//! `COURSE_LISTS.point_records` is `HELD` while the records are taken, 0 otherwise.

use crate::errors::{BadInput, Refused};
use crate::race::owner::race;
use crate::session::Dword;
use std::sync::atomic::{AtomicBool, Ordering};

/// The records.
pub const COUNT: u32 = 0x2000;
/// The block's bytes, as counted.
pub const BYTES: u32 = 0x4_0000;
/// `COURSE_LISTS.point_records` while the records are taken.
pub const HELD: u32 = 1;

pub type Record = [u32; 8];

/// The records, as cells: read and written many times a polygon.
/// One set per drawing thread (`workers`): a record is a thread's own projection, made again
/// by another thread that needs it.
static RECORDS: crate::workers::PerWorker<[[Dword; 8]; COUNT as usize]> =
    crate::workers::PerWorker::new(
        [const { [const { [const { Dword::new(0) }; 8] }; COUNT as usize] }; crate::workers::COUNT],
    );
/// The records' copies (`workers`).
pub fn copies() -> &'static dyn crate::workers::Copies {
    &RECORDS
}

/// Whether the records are taken.
static TAKEN: AtomicBool = AtomicBool::new(false);

/// The records made, zeroed, counted on `list`.
pub fn take(
    course: &crate::race::owner_course::Course,
    list: &dyn crate::pointers::Store,
) -> Result<(), Refused> {
    crate::memory::owned::count_on(list, BYTES)?;
    RECORDS
        .all()
        .iter()
        .flatten()
        .flatten()
        .for_each(|cell| cell.set(0));
    TAKEN.store(true, Ordering::Relaxed);
    course.lists.point_records.set(HELD);
    Ok(())
}

/// Given back (the list's `free_list` counts them down).
pub fn give_back() {
    TAKEN.store(false, Ordering::Relaxed);
}

/// The first `n` records' cells in every drawing thread's copy (copy 0 first): a clear
/// of the records reaches them all, so no copy keeps a record of an earlier frame. While
/// views are drawn beside each other (`workers::draw_beside`) only the running thread's
/// copy: the other view's are its own, being drawn.
pub fn first_records_everywhere(
    n: usize,
) -> Result<impl Iterator<Item = &'static [Dword; 8]>, BadInput> {
    if !TAKEN.load(Ordering::Relaxed) {
        return Err(BadInput("no point records"));
    }
    if n > COUNT as usize {
        return Err(BadInput("a point record past the block"));
    }
    let copies: &'static [_] = if crate::workers::beside() {
        std::slice::from_ref(&*RECORDS)
    } else {
        RECORDS.all()
    };
    Ok(copies.iter().flat_map(move |copy| &copy[..n]))
}

/// Whether the records are taken.
pub fn held() -> bool {
    race().course.lists.point_records.get() != 0
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
/// The pool's records (`POLYGON_WORK.record_pool`, 0x2020 bytes).
pub const POOL: u32 = 0x101;

/// The record of quad vertex `k`.
pub fn of_quad(k: u32) -> u32 {
    QUAD_FIRST + k
}

/// The record of the pool's record `k`.
pub fn of_pool(k: u32) -> u32 {
    POOL_FIRST + k
}

/// The cells of record `r`, of whichever kind; `quad` is the view's quad vertices
/// (`PolygonWork::quad_vertices`), `None` outside a view's drawing (a quad record is then
/// refused).
#[inline]
fn cells(r: u32, quad: Option<&[Dword; 32]>) -> Result<&[Dword; 8], BadInput> {
    if r > COUNT {
        return far_cells(r, quad);
    }
    if !TAKEN.load(Ordering::Relaxed) {
        return Err(BadInput("no point records"));
    }
    (r as usize)
        .checked_sub(1)
        .and_then(|k| RECORDS.get(k))
        .ok_or(BadInput("a point record past the block"))
}

/// The cells of a quad vertex's or a pool record.
fn far_cells<'a>(r: u32, quad: Option<&'a [Dword; 32]>) -> Result<&'a [Dword; 8], BadInput> {
    let (all, k): (&'a [Dword], u32) = match r {
        _ if (QUAD_FIRST..POOL_FIRST).contains(&r) => (
            quad.ok_or(BadInput("a quad vertex outside a view's drawing"))?,
            r - QUAD_FIRST,
        ),
        _ if (POOL_FIRST..POOL_FIRST + POOL).contains(&r) => (
            &crate::scene::polygon_store::POLYGON_WORK.record_pool,
            r - POOL_FIRST,
        ),
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

/// Record `r`; refused for none or one past the records.
#[inline]
pub fn get(r: u32) -> Result<Record, BadInput> {
    get_in(None, r)
}

/// `get` with the view's quad vertices (`PolygonWork::quad_vertices`).
#[inline]
pub fn get_in(quad: Option<&[Dword; 32]>, r: u32) -> Result<Record, BadInput> {
    let cells = cells(r, quad)?;
    Ok(std::array::from_fn(|k| cells[k].get()))
}

/// Record `r` set to `record`; refused as `get`.
#[inline]
pub fn set(r: u32, record: Record) -> Result<(), BadInput> {
    set_in(None, r, record)
}

/// `set` with the view's quad vertices.
#[inline]
pub fn set_in(quad: Option<&[Dword; 32]>, r: u32, record: Record) -> Result<(), BadInput> {
    for (cell, v) in cells(r, quad)?.iter().zip(record) {
        cell.set(v);
    }
    Ok(())
}

/// The dword at `offset` of record `r`; refused for an offset past the record.
#[inline]
pub fn field(r: u32, offset: u32) -> Result<u32, BadInput> {
    field_in(None, r, offset)
}

/// `field` with the view's quad vertices.
#[inline]
pub fn field_in(quad: Option<&[Dword; 32]>, r: u32, offset: u32) -> Result<u32, BadInput> {
    Ok(cell(cells(r, quad)?, offset)?.get())
}

/// The dword at `offset` of record `r` set to `v`; refused as `field`.
#[inline]
pub fn put_field(r: u32, offset: u32, v: u32) -> Result<(), BadInput> {
    put_field_in(None, r, offset, v)
}

/// `put_field` with the view's quad vertices.
#[inline]
pub fn put_field_in(
    quad: Option<&[Dword; 32]>,
    r: u32,
    offset: u32,
    v: u32,
) -> Result<(), BadInput> {
    cell(cells(r, quad)?, offset)?.set(v);
    Ok(())
}

/// The dword at `offset` of a polygon vertex's record. A refusal ends the game, as the
/// original's read of a wrong address went wrong (the polygon code's reads cannot carry
/// one).
#[inline]
pub fn vertex(quad: &[Dword; 32], v: u32, offset: u32) -> u32 {
    field_in(Some(quad), v, offset)
        .unwrap_or_else(|e| crate::fatal::raise::end("a point record", e))
}

/// A polygon vertex's whole record (as `vertex`), its bytes.
pub fn vertex_bytes(quad: &[Dword; 32], v: u32) -> Result<[u8; 32], BadInput> {
    let record = get_in(Some(quad), v)?;
    Ok(std::array::from_fn(|n| record[n / 4].to_le_bytes()[n % 4]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_named_by_number() {
        let course = &race().course;
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept = course.lists.point_records.get();
        let memory = crate::memory::ListCells::empty();
        take(course, &memory).unwrap();
        assert_eq!(memory.get().total, BYTES);
        assert_eq!(at_offset(0x20 * 5), of_scene_point(0x1005));
        assert_eq!(of_object_point(3), at_offset(0x20 * 0x1003));
        let quad = [const { Dword::new(0) }; 32];
        put_field(of_scene_point(7), 0x18, 42).unwrap();
        assert_eq!(vertex(&quad, of_scene_point(7), 0x18), 42);
        assert_eq!(get(8).unwrap()[6], 42);
        assert!(get(0).is_err());
        assert!(field(1, 0x20).is_err() && field(1, 2).is_err());
        // after the point records the quad vertices, then the clipper's pool, by their cells
        assert_eq!((of_quad(0), of_pool(0)), (COUNT + 1, COUNT + 1 + QUADS));
        let pool = &crate::scene::polygon_store::POLYGON_WORK.record_pool;
        let (kept_quad, kept_pool) = (
            get_in(Some(&quad), of_quad(3)).unwrap(),
            get(of_pool(POOL - 1)).unwrap(),
        );
        put_field_in(Some(&quad), of_quad(3), 0x1C, 7).unwrap();
        put_field(of_pool(POOL - 1), 0, 9).unwrap();
        assert_eq!((quad[31].get(), pool[0x800].get()), (7, 9));
        set_in(Some(&quad), of_quad(3), kept_quad).unwrap();
        assert!(get(of_quad(0)).is_err());
        set(of_pool(POOL - 1), kept_pool).unwrap();
        assert!(get(of_pool(POOL)).is_err() && get(0x1_0000).is_err());
        give_back();
        course.lists.point_records.set(kept);
        crate::memory::with_figures(|f| *f = figures);
    }
}
