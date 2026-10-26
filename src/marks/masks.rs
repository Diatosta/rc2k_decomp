//! `sub_4C0F2E`'s `sub_4BDE9A` (IDA, 2026-10-17): the marks' masks. In a work block of
//! 0x80000 bytes it gathers the points of faces and makes shapes of them; each shape's
//! rows of spans are filled into a mask of 0x8000 bytes (rows of 0x100) with the shape's
//! number, 0xFF elsewhere. These are the leaves.

use crate::errors::{BadInput, Refused};
use crate::marks::mask_picks::PICKS;
use crate::marks::mask_vars::MASKS;
use crate::marks::shape_work::SHAPE_WORK;
use crate::pointers::GamePtr;
use crate::session::Dword;
use rc2k_core::marks::masks::{ShapeRows, Span};

pub const MASK_LEN: u32 = rc2k_core::marks::masks::MASK_LEN as u32;

/// The two point lists `collect_points_a` and `_b` fill: count, ids, faces.
pub struct PointList {
    pub count: &'static Dword,
    pub ids: &'static [Dword; 64],
    pub faces: &'static [Dword; 64],
}

pub static POINTS_A: PointList = PointList {
    count: &SHAPE_WORK.work.count_a, // g_marksPointsACount
    ids: &SHAPE_WORK.work.ids_a,     // g_marksPointsA
    faces: &SHAPE_WORK.work.faces_a, // g_marksPointsAFaces
};
pub static POINTS_B: PointList = PointList {
    count: &SHAPE_WORK.work.count_b, // g_marksPointsBCount
    ids: &SHAPE_WORK.work.ids_b,     // g_marksPointsB
    faces: &SHAPE_WORK.work.faces_b, // g_marksPointsBFaces
};
pub const MOST_POINTS: u32 = rc2k_core::marks::masks::MOST_POINTS;

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a game field")
}

impl PointList {}

/// `marks::collect_points_a` (0x4BD551) and `_b` (0x4BD59F): face `face`'s points (+0 the
/// count, from +0x10, 16 bytes apart) added to the list unless there already, up to 0x40,
/// each with the face. Refuses a list of more than 0x40. Converted:
/// `rc2k_core::marks::masks::PointList::add_face`.
pub fn collect_points(list: &PointList, face: u32) -> Result<(), Refused> {
    unsafe {
        let count: u32 = list.count.get();
        let most = MOST_POINTS as usize;
        let mut points = rc2k_core::marks::masks::PointList {
            count,
            ids: std::slice::from_raw_parts_mut(list.ids.as_ptr() as *mut u32, most),
            faces: std::slice::from_raw_parts_mut(list.faces.as_ptr() as *mut u32, most),
        };
        points.check()?;
        let face_points = get(face)?;
        let bad = crate::marks::Sticky::default();
        let added = points.add_face(face, face_points, |k| bad.take(get(face + 0x10 + 0x10 * k)));
        bad.done()?;
        added?;
        list.count.set(points.count);
    }
    Ok(())
}
/// `marks::reset_mask_cursor` (0x4BE0A3): `g_marksMaskCursor` the work block, the runs
/// from 0x831D2E, `g_marksMaskRunCount` 0. Every register kept. Converted:
/// `rc2k_core::marks::masks::MaskCursor::reset`.
pub fn reset_mask_cursor() {
    let c =
        rc2k_core::marks::masks::MaskCursor::reset(MASKS.work.get(), MASKS.run_table.addr() as u32);
    MASKS.cursor.set(c.cursor);
    MASKS.runs.set(c.runs);
    MASKS.run_count.set(c.run_count);
}

/// Entry `index` of a list of shapes' rows (`ebp` +8): 16 bytes each, +0 the spans (x,
/// length; 8 bytes each), +4 the first row, +8 the count (signed).
fn shape_rows(entries: u32, index: u32) -> Result<ShapeRows, Refused> {
    Ok({
        let entry = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, first_row, count) = (get(entry)?, get(entry + 4)?, get(entry + 8)?);
        ShapeRows {
            first_row,
            spans: (0..(count as i32).max(0) as u32)
                .map(|i| {
                    Ok(Span {
                        x: get(spans.wrapping_add(8 * i))?,
                        len: get(spans.wrapping_add(8 * i + 4))?,
                    })
                })
                .collect::<Result<_, BadInput>>()?,
        }
    })
}

/// The mask at `at`, checked to lie in one region.
fn mask(at: u32) -> Result<&'static mut [u8], BadInput> {
    crate::race::trap::span(at, MASK_LEN as usize, "a mark mask")?;
    // SAFETY: the span lies in one live region (checked above); nothing else holds it
    // while the mask is drawn.
    Ok(unsafe { std::slice::from_raw_parts_mut(crate::pointers::host(at), MASK_LEN as usize) })
}

/// `marks::draw_mask` (0x4BE6C2): the mask at `at` (into `g_marksMask`) all 0xFF, then
/// each of the +4 shapes at `shapes` (`g_marksMaskShapes`) drawn (`draw_mask_shape`).
/// Refuses a count of 0 or less (the original's loop tests after counting and would run
/// round) or a span outside the mask, before writing. Converted:
/// `rc2k_core::marks::masks::draw_mask`.
pub fn draw_mask(at: u32, shapes: u32) -> Result<(), Refused> {
    {
        let count = get(shapes.wrapping_add(4))? as i32;
        if count <= 0 {
            return Err(BadInput("a mask of no shapes: the loop would run round").into());
        }
        let entries = get(shapes.wrapping_add(8))?;
        let mut rows = Vec::new();
        for k in 0..2 * count as u32 {
            let r = shape_rows(entries, k)?;
            r.fit()?;
            rows.push(r);
        }
        rc2k_core::marks::masks::draw_mask(mask(at)?, &rows)?;
        PICKS.mask.set(at);
        PICKS.shapes.set(shapes);
        MASKS.mask_value.set(count as u32 - 1);
    }
    Ok(())
}

const WORK_LEN: u32 = 0x8_0000;

/// `marks::make_masks` (0x4BDE9A), in `sub_4C0F2E`: the 0x80000-byte work block taken on
/// its list (`memory::allocate`); `make_shape_sets`, `make_corners`, `reset_mask_cursor`;
/// each shape's ring (`span_ring`, its corner table in `g_marksRingCorners`), mask
/// (`draw_mask`) and pixels (`map_shapes`); the list freed (`memory::free_list`). Refuses
/// as its steps do, mid-way (the work block is on the list, which the original's run
/// frees with its own).
pub fn make_masks() -> Result<(), Refused> {
    // The two shapes' ring-angle tables, masks, pixel tables and corner tables.
    let shapes = [
        crate::pointers::game_address(core::ptr::addr_of!(MASKS.shapes[0])),
        crate::pointers::game_address(core::ptr::addr_of!(MASKS.shapes[1])),
    ];
    let corners = [MASKS.corners_a.addr() as u32, MASKS.corners_b.addr() as u32];
    let ring_angles = [
        MASKS.ring_angles_a.addr() as u32,
        MASKS.ring_angles_b.addr() as u32,
    ];
    let masks = [MASKS.mask_a.addr() as u32, MASKS.mask_b.addr() as u32];
    let pixel_tables = [
        PICKS.pixel_table_a.addr() as u32,
        PICKS.pixel_table_b.addr() as u32,
    ];
    let list = crate::pointers::host_at(MASKS.list.addr()) as *mut crate::memory::MemoryList;
    let work = crate::pointers::game_address(crate::memory::allocate::allocate(list, WORK_LEN)?);
    MASKS.work.set(work);
    crate::marks::shape_points::make_shape_sets()?;
    crate::marks::corners::make_corners()?;
    reset_mask_cursor();
    for n in 0..2 {
        MASKS.ring_corners.set(corners[n]);
        crate::marks::rings::span_ring(shapes[n], ring_angles[n])?;
    }
    for n in 0..2 {
        draw_mask(masks[n], shapes[n])?;
    }
    for n in 0..2 {
        crate::marks::pixels::map_shapes(shapes[n], pixel_tables[n], corners[n])?;
    }
    crate::memory::free_list::free_list(list)?;
    Ok(())
}
