//! `sub_4C0F2E`'s `sub_4BDE9A` (IDA, 2026-10-17): the marks' masks. In a work block of
//! 0x80000 bytes it gathers the points of faces and makes shapes of them; each shape's
//! rows of spans are filled into a mask of 0x8000 bytes (rows of 0x100) with the shape's
//! number, 0xFF elsewhere. These are the leaves.

use crate::errors::{BadInput, Refused};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use rc2k_core::marks::masks::{ShapeRows, Span};

pub const WORK_RVA: Slot = Slot::at(0x82E386); // g_marksMaskWork: the 0x80000-byte block
pub const CURSOR_RVA: Slot = Slot::at(0x82FD0E); // g_marksMaskCursor
pub const RUNS_RVA: Slot = Slot::at(0x82FD12); // g_marksMaskRuns: where the next run goes
const RUNS_START_RVA: Slot = Slot::at(0x831D2E); // unk_831D2E
pub const RUN_COUNT_RVA: Slot = Slot::at(0x833D7E); // g_marksMaskRunCount
pub const MASK_RVA: Slot = Slot::at(0x844E14); // g_marksMask: the mask being drawn
pub const SHAPES_RVA: Slot = Slot::at(0x844E10); // g_marksMaskShapes: its shapes
pub const VALUE_RVA: Slot = Slot::at(0x833DBA); // g_marksMaskValue: the shape's number
pub const MASK_LEN: u32 = rc2k_core::marks::masks::MASK_LEN as u32;

/// The two point lists `collect_points_a` and `_b` fill: count, ids, faces.
pub struct PointList {
    pub count: Slot,
    pub ids: Slot,
    pub faces: Slot,
}

pub const POINTS_A: PointList = PointList {
    count: Slot::at(0x884FEC), // g_marksPointsACount
    ids: Slot::at(0x884FF4),   // g_marksPointsA
    faces: Slot::at(0x8851F4), // g_marksPointsAFaces
};
pub const POINTS_B: PointList = PointList {
    count: Slot::at(0x884FF0), // g_marksPointsBCount
    ids: Slot::at(0x8850F4),   // g_marksPointsB
    faces: Slot::at(0x8852F4), // g_marksPointsBFaces
};
pub const MOST_POINTS: u32 = rc2k_core::marks::masks::MOST_POINTS;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

impl PointList {}

/// `marks::collect_points_a` (0x4BD551) and `_b` (0x4BD59F): face `face`'s points (+0 the
/// count, from +0x10, 16 bytes apart) added to the list unless there already, up to 0x40,
/// each with the face. Refuses a list of more than 0x40. Converted:
/// `rc2k_core::marks::masks::PointList::add_face`.
pub unsafe fn collect_points(list: &PointList, face: u32) -> Result<(), Refused> {
    unsafe {
        let count: u32 = game_read(list.count);
        let most = MOST_POINTS as usize;
        let mut points = rc2k_core::marks::masks::PointList {
            count,
            ids: std::slice::from_raw_parts_mut(game_addr(list.ids) as *mut u32, most),
            faces: std::slice::from_raw_parts_mut(game_addr(list.faces) as *mut u32, most),
        };
        points.check()?;
        let face_points = get(face);
        points.add_face(face, face_points, |k| get(face + 0x10 + 0x10 * k))?;
        game_write(list.count, points.count);
    }
    Ok(())
}
/// `marks::reset_mask_cursor` (0x4BE0A3): `g_marksMaskCursor` the work block, the runs
/// from 0x831D2E, `g_marksMaskRunCount` 0. Every register kept. Converted:
/// `rc2k_core::marks::masks::MaskCursor::reset`.
pub unsafe fn reset_mask_cursor() {
    let c = rc2k_core::marks::masks::MaskCursor::reset(
        unsafe { game_read(WORK_RVA) },
        game_addr(RUNS_START_RVA) as u32,
    );
    unsafe {
        game_write(CURSOR_RVA, c.cursor);
        game_write(RUNS_RVA, c.runs);
        game_write(RUN_COUNT_RVA, c.run_count);
    }
}

/// Entry `index` of a list of shapes' rows (`ebp` +8): 16 bytes each, +0 the spans (x,
/// length; 8 bytes each), +4 the first row, +8 the count (signed).
unsafe fn shape_rows(entries: u32, index: u32) -> ShapeRows {
    unsafe {
        let entry = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, first_row, count) = (get(entry), get(entry + 4), get(entry + 8));
        ShapeRows {
            first_row,
            spans: (0..(count as i32).max(0) as u32)
                .map(|i| Span {
                    x: get(spans.wrapping_add(8 * i)),
                    len: get(spans.wrapping_add(8 * i + 4)),
                })
                .collect(),
        }
    }
}

/// The mask at `at`.
unsafe fn mask(at: u32) -> &'static mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(at as *mut u8, MASK_LEN as usize) }
}

/// `marks::draw_mask` (0x4BE6C2): the mask at `at` (into `g_marksMask`) all 0xFF, then
/// each of the +4 shapes at `shapes` (`g_marksMaskShapes`) drawn (`draw_mask_shape`).
/// Refuses a count of 0 or less (the original's loop tests after counting and would run
/// round) or a span outside the mask, before writing. Converted:
/// `rc2k_core::marks::masks::draw_mask`.
pub unsafe fn draw_mask(at: u32, shapes: u32) -> Result<(), Refused> {
    unsafe {
        let count = get(shapes.wrapping_add(4)) as i32;
        if count <= 0 {
            return Err(BadInput("a mask of no shapes: the loop would run round").into());
        }
        let entries = get(shapes.wrapping_add(8));
        let mut rows = Vec::new();
        for k in 0..2 * count as u32 {
            let r = shape_rows(entries, k);
            r.fit()?;
            rows.push(r);
        }
        rc2k_core::marks::masks::draw_mask(mask(at), &rows)?;
        game_write(MASK_RVA, at);
        game_write(SHAPES_RVA, shapes);
        game_write(VALUE_RVA, count as u32 - 1);
    }
    Ok(())
}

const LIST_RVA: Slot = Slot::at(0x82E346); // stru_82E346: the work block's list
const WORK_LEN: u32 = 0x8_0000;
/// The two shapes' ring-angle tables, masks and pixel tables.
const SHAPES: [Slot; 2] = [Slot::at(0x82FC56), Slot::at(0x82FC7A)];
const RING_ANGLES: [Slot; 2] = [Slot::at(0x843DC2), Slot::at(0x8441CA)];
const MASKS: [Slot; 2] = [Slot::at(0x833DC2), Slot::at(0x83BDC2)];
const PIXEL_TABLES: [Slot; 2] = [Slot::at(0x844E58), Slot::at(0x864E58)];

/// `marks::make_masks` (0x4BDE9A), in `sub_4C0F2E`: the 0x80000-byte work block taken on
/// its list (`memory::allocate`); `make_shape_sets`, `make_corners`, `reset_mask_cursor`;
/// each shape's ring (`span_ring`, its corner table in `g_marksRingCorners`), mask
/// (`draw_mask`) and pixels (`map_shapes`); the list freed (`memory::free_list`). Refuses
/// as its steps do, mid-way (the work block is on the list, which the original's run
/// frees with its own).
pub unsafe fn make_masks() -> Result<(), Refused> {
    use crate::marks::corners::CORNERS_RVA;
    use crate::marks::rings::CORNERS_RVA as RING_CORNERS_RVA;
    let a = |r: Slot| game_addr(r) as u32;
    unsafe {
        let list = a(LIST_RVA) as *mut crate::memory::MemoryList;
        let work = crate::memory::allocate::allocate(list, WORK_LEN)? as u32;
        game_write(WORK_RVA, work);
        crate::marks::shape_points::make_shape_sets()?;
        crate::marks::corners::make_corners()?;
        reset_mask_cursor();
        for n in 0..2 {
            game_write(RING_CORNERS_RVA, a(CORNERS_RVA[n]));
            crate::marks::rings::span_ring(a(SHAPES[n]), a(RING_ANGLES[n]))?;
        }
        for n in 0..2 {
            draw_mask(a(MASKS[n]), a(SHAPES[n]))?;
        }
        for n in 0..2 {
            crate::marks::pixels::map_shapes(a(SHAPES[n]), a(PIXEL_TABLES[n]), a(CORNERS_RVA[n]))?;
        }
        crate::memory::free_list::free_list(list)?;
    }
    Ok(())
}
