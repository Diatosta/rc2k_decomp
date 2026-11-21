//! The scene's point records (0x40000 bytes, taken by `scene::allocate_point_records`), counted on
//! the list, given back by `give_back`. 0x2000 records of eight dwords: the point's number, (after
//! three unused dwords) the x on the screen, the point in the view (three floats, the depth last),
//! the y on the screen. A scene point n has record n & 0xFFF, an object's point or normal n record
//! 0x1000 + (n & 0xFFF).
//!
//! A record is named by its number + 1 (1 .. 0x2000), in the polygon entries' record field. The two
//! other kinds of record the entries name follow: the four quad vertices
//! (`PolygonWork::quads`, the view's own, `of_quad`) and the clipper's pool (the part's
//! `PolygonWork::record_pool`, `of_pool`; `polygon_next_record` holds the next one's number).
//! `point_records` is `HELD` while the records are taken, 0 otherwise.

use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::scene::polygon_work_store::QuadSlot;

/// The records.
pub const COUNT: u32 = 0x2000;
/// The block's bytes, as counted.
pub const BYTES: u32 = 0x4_0000;
/// `point_records` while the records are taken.
pub const HELD: u32 = 1;

pub type Record = [u32; 8];

/// The records of one part, as cells: read and written many times a polygon.
pub type Records = [[PlainDword; 8]; COUNT as usize];

/// A part's records, all zero (`scene::scratch_store::PartScratch::records`): a record is the part's own
/// projection, made again by another part that needs it. Every reader below is handed the
/// part's scratch.
pub const fn new_records() -> Records {
    [const { [const { PlainDword::new(0) }; 8] }; COUNT as usize]
}

/// Which of a part's records have been written since `clear_written`, a bit each
/// (`scene::scratch_store::PartScratch::records_written`): what a piece drawn aside wrote is put into its
/// part's records (`put_written`).
pub type Written = [std::cell::Cell<u64>; COUNT as usize / 64];

/// No record written.
pub const fn new_written() -> Written {
    [const { std::cell::Cell::new(0) }; COUNT as usize / 64]
}

/// Record `r` (1 .. `COUNT`) written in `part`'s records.
#[inline]
fn note_written(part: &crate::scene::scratch_store::PartScratch, r: u32) {
    if let Some(k) = (r as usize).checked_sub(1)
        && let Some(bits) = part.records_written.get(k / 64)
    {
        bits.set(bits.get() | 1 << (k % 64));
    }
}

/// The records `part` has written (since `clear_written`), each by its index.
fn each_written(part: &crate::scene::scratch_store::PartScratch, mut f: impl FnMut(usize)) {
    for (n, bits) in part.records_written.iter().enumerate() {
        let mut left = bits.get();
        while left != 0 {
            f(n * 64 + left.trailing_zeros() as usize);
            left &= left - 1;
        }
    }
}

/// Each record `from` has written (since `clear_written`) put into `to`'s records, whole.
pub fn put_written(
    from: &crate::scene::scratch_store::PartScratch,
    to: &crate::scene::scratch_store::PartScratch,
) {
    each_written(from, |k| {
        for (a, b) in to.records[k].iter().zip(&from.records[k]) {
            a.set(b.get());
        }
    });
}

/// The records `part` has written (since the last time) set back to 0 and forgotten: a piece
/// drawn aside starts from records all 0 each time (`scene::workers::draw_aside`) without clearing
/// all 0x2000.
pub fn clear_written(part: &crate::scene::scratch_store::PartScratch) {
    each_written(part, |k| part.records[k].iter().for_each(|c| c.set(0)));
    for bits in &part.records_written {
        bits.set(0);
    }
}

/// The scene points' records of `part` (the first 0x1000) marked as holding no point, as a
/// view's start leaves them (`race::clear_point_records`): a scratch whose records were set
/// back to 0 would otherwise pass for holding scene point 0. Not counted as written.
pub fn mark_scene_records_unmade(part: &crate::scene::scratch_store::PartScratch) {
    for record in &part.records[..0x1000] {
        record[0].set(u32::MAX);
    }
}

/// The records made, zeroed, counted on `list`.
pub fn take(
    scratch: &crate::scene::scratch_store::DrawingScratch,
    course: &crate::race::owner_course::Course,
    list: &crate::memory::PlainList,
) -> Result<(), Refused> {
    crate::memory::owned::count_on(list, BYTES)?;
    for part in scratch.all_parts_on_game_thread() {
        part.records.iter().flatten().for_each(|cell| cell.set(0));
        part.records_taken.set(true);
    }
    course.lists.borrow_mut().point_records = HELD;
    Ok(())
}

/// Given back (the list's `free_list` counts them down).
pub fn give_back(scratch: &crate::scene::scratch_store::DrawingScratch) {
    for part in scratch.all_parts_on_game_thread() {
        part.records_taken.set(false);
    }
}

/// The first `n` records' cells in every drawing thread's copy (copy 0 first): a clear
/// of the records reaches them all, so no copy keeps a record of an earlier frame. While
/// views are drawn beside each other (`scene::workers::draw_beside`) only `part`'s copy (the
/// drawing part clearing them): the other view's are its own, being drawn.
pub fn first_records_everywhere<'a>(
    scratch: &'a crate::scene::scratch_store::DrawingScratch,
    part: &'a crate::scene::scratch_store::PartScratch,
    n: usize,
) -> Result<impl Iterator<Item = &'a [PlainDword; 8]>, BadInput> {
    if !part.records_taken.is_set() {
        return Err(BadInput("no point records"));
    }
    if n > COUNT as usize {
        return Err(BadInput("a point record past the block"));
    }
    // Beside each other: the part's own. Otherwise the game's thread draws alone and reaches
    // every part's (no scope is open: no run holds one).
    let copies: [Option<&'a crate::scene::scratch_store::PartScratch>;
        crate::memory::draw_order::COUNT] = if crate::scene::workers::beside() {
        std::array::from_fn(|k| (k == 0).then_some(part))
    } else {
        let mut all = scratch.all_parts_on_game_thread();
        std::array::from_fn(|_| all.next())
    };
    Ok(copies
        .into_iter()
        .flatten()
        .flat_map(move |copy| &copy.records[..n]))
}

/// Whether the records are taken.
pub fn held(lists: crate::race::course_lists_store::CourseListsRef<'_>) -> bool {
    lists.point_records() != 0
}

/// The record at byte `offset` of the block.
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
/// The quad vertices (`PolygonWork::quads`, 8 dwords each).
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

/// Where record `r` is, of whichever kind: cells of `part`'s records or pool, or one of the
/// view's quad vertices.
enum Place<'a> {
    Cells(&'a [PlainDword; 8]),
    Quad(&'a QuadSlot, usize),
}

impl Place<'_> {
    fn record(&self) -> Record {
        match self {
            Place::Cells(cells) => std::array::from_fn(|k| cells[k].get()),
            Place::Quad(quad, first) => {
                let quad = quad.borrow();
                std::array::from_fn(|k| quad.words[first + k])
            }
        }
    }

    fn put_record(&self, record: Record) {
        match self {
            Place::Cells(cells) => {
                for (cell, v) in cells.iter().zip(record) {
                    cell.set(v);
                }
            }
            Place::Quad(quad, first) => {
                quad.borrow_mut().words[*first..*first + 8].copy_from_slice(&record);
            }
        }
    }

    /// The dword at `offset`; refused for an offset past the record.
    fn field(&self, offset: u32) -> Result<u32, BadInput> {
        let k = word_of(offset)?;
        Ok(match self {
            Place::Cells(cells) => cells[k].get(),
            Place::Quad(quad, first) => quad.borrow().words[first + k],
        })
    }

    fn put_field(&self, offset: u32, v: u32) -> Result<(), BadInput> {
        let k = word_of(offset)?;
        match self {
            Place::Cells(cells) => cells[k].set(v),
            Place::Quad(quad, first) => quad.borrow_mut().words[first + k] = v,
        }
        Ok(())
    }
}

/// The index of the dword at byte `offset` of a record; refused for an offset past the record.
#[inline]
fn word_of(offset: u32) -> Result<usize, BadInput> {
    if offset.is_multiple_of(4) && offset < 32 {
        return Ok(offset as usize / 4);
    }
    Err(BadInput("a field past a point record"))
}

/// Record `r` in `part`'s records, of whichever kind; `quad` is the view's quad vertices,
/// `None` outside a view's drawing (a quad record is then refused).
#[inline]
fn place<'a>(
    part: &'a crate::scene::scratch_store::PartScratch,
    r: u32,
    quad: Option<&'a QuadSlot>,
) -> Result<Place<'a>, BadInput> {
    if r > COUNT {
        return far_place(part, r, quad);
    }
    if !part.records_taken.is_set() {
        return Err(BadInput("no point records"));
    }
    (r as usize)
        .checked_sub(1)
        .and_then(|k| part.records.get(k))
        .map(Place::Cells)
        .ok_or(BadInput("a point record past the block"))
}

/// A quad vertex's or a pool record's place (`part`'s pool).
fn far_place<'a>(
    part: &'a crate::scene::scratch_store::PartScratch,
    r: u32,
    quad: Option<&'a QuadSlot>,
) -> Result<Place<'a>, BadInput> {
    if (QUAD_FIRST..POOL_FIRST).contains(&r) {
        let quad = quad.ok_or(BadInput("a quad vertex outside a view's drawing"))?;
        return Ok(Place::Quad(quad, 8 * (r - QUAD_FIRST) as usize));
    }
    if (POOL_FIRST..POOL_FIRST + POOL).contains(&r) {
        let first = 8 * (r - POOL_FIRST) as usize;
        return part.polygon.record_pool[first..first + 8]
            .try_into()
            .map(Place::Cells)
            .map_err(|_| BadInput("a point record past the records"));
    }
    Err(BadInput("a point record past the records"))
}

/// Record `r` of `part`'s records; refused for none or one past the records.
#[inline]
pub fn get(part: &crate::scene::scratch_store::PartScratch, r: u32) -> Result<Record, BadInput> {
    get_in(part, None, r)
}

/// `get` with the view's quad vertices (`PolygonWork::quads`).
#[inline]
pub fn get_in(
    part: &crate::scene::scratch_store::PartScratch,
    quad: Option<&QuadSlot>,
    r: u32,
) -> Result<Record, BadInput> {
    Ok(place(part, r, quad)?.record())
}

/// Record `r` set to `record`; refused as `get`.
#[inline]
pub fn set(
    part: &crate::scene::scratch_store::PartScratch,
    r: u32,
    record: Record,
) -> Result<(), BadInput> {
    set_in(part, None, r, record)
}

/// `set` with the view's quad vertices.
#[inline]
pub fn set_in(
    part: &crate::scene::scratch_store::PartScratch,
    quad: Option<&QuadSlot>,
    r: u32,
    record: Record,
) -> Result<(), BadInput> {
    place(part, r, quad)?.put_record(record);
    note_written(part, r);
    Ok(())
}

/// The dword at `offset` of record `r`; refused for an offset past the record.
#[inline]
pub fn field(
    part: &crate::scene::scratch_store::PartScratch,
    r: u32,
    offset: u32,
) -> Result<u32, BadInput> {
    field_in(part, None, r, offset)
}

/// `field` with the view's quad vertices.
#[inline]
pub fn field_in(
    part: &crate::scene::scratch_store::PartScratch,
    quad: Option<&QuadSlot>,
    r: u32,
    offset: u32,
) -> Result<u32, BadInput> {
    place(part, r, quad)?.field(offset)
}

/// The dword at `offset` of record `r` set to `v`; refused as `field`.
#[inline]
pub fn put_field(
    part: &crate::scene::scratch_store::PartScratch,
    r: u32,
    offset: u32,
    v: u32,
) -> Result<(), BadInput> {
    put_field_in(part, None, r, offset, v)
}

/// `put_field` with the view's quad vertices.
#[inline]
pub fn put_field_in(
    part: &crate::scene::scratch_store::PartScratch,
    quad: Option<&QuadSlot>,
    r: u32,
    offset: u32,
    v: u32,
) -> Result<(), BadInput> {
    place(part, r, quad)?.put_field(offset, v)?;
    note_written(part, r);
    Ok(())
}

/// The dword at `offset` of a polygon vertex's record. A refusal ends the game (the polygon
/// code's reads cannot carry one).
#[inline]
pub fn vertex(
    part: &crate::scene::scratch_store::PartScratch,
    quad: &QuadSlot,
    v: u32,
    offset: u32,
) -> u32 {
    field_in(part, Some(quad), v, offset)
        .unwrap_or_else(|e| crate::fatal::raise::end("a point record", e))
}

/// A polygon vertex's whole record (as `vertex`), its bytes.
pub fn vertex_bytes(
    part: &crate::scene::scratch_store::PartScratch,
    quad: &QuadSlot,
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
        let course = &crate::test_support::own::race().course;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = course.lists.borrow().point_records;
        let memory = crate::memory::PlainList::empty();
        take(&crate::owners::Owners::the().scene.scratch, course, &memory).unwrap();
        assert_eq!(memory.get().total, BYTES);
        assert_eq!(at_offset(0x20 * 5), of_scene_point(0x1005));
        assert_eq!(of_object_point(3), at_offset(0x20 * 0x1003));
        let quad = QuadSlot::new(crate::scene::polygon_work_store::QuadVertices::new());
        let part = crate::owners::Owners::the().scene.scratch.game_scratch();
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
        assert_eq!((quad.borrow().words[31], pool[0x800].get()), (7, 9));
        set_in(part, Some(&quad), of_quad(3), kept_quad).unwrap();
        assert!(get(part, of_quad(0)).is_err());
        set(part, of_pool(POOL - 1), kept_pool).unwrap();
        assert!(get(part, of_pool(POOL)).is_err() && get(part, 0x1_0000).is_err());
        give_back(&crate::owners::Owners::the().scene.scratch);
        course.lists.borrow_mut().point_records = kept;
        crate::memory::with_figures(|f| *f = figures);
    }
}
