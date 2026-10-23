//! `sub_4C0F2E`'s `sub_4BDE9A` (IDA, 2026-10-17): the marks' masks. In a work block of
//! 0x80000 bytes it gathers the points of faces and makes shapes of them; each shape's
//! rows of spans are filled into a mask of 0x8000 bytes (rows of 0x100) with the shape's
//! number, 0xFF elsewhere. These are the leaves.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use rc2k_core::marks::masks::{ShapeRows, Span};

pub const WORK_RVA: usize = rva(0x82E386); // g_marksMaskWork: the 0x80000-byte block
pub const CURSOR_RVA: usize = rva(0x82FD0E); // g_marksMaskCursor
pub const RUNS_RVA: usize = rva(0x82FD12); // g_marksMaskRuns: where the next run goes
const RUNS_START_RVA: usize = rva(0x831D2E); // unk_831D2E
pub const RUN_COUNT_RVA: usize = rva(0x833D7E); // g_marksMaskRunCount
pub const MASK_RVA: usize = rva(0x844E14); // g_marksMask: the mask being drawn
pub const SHAPES_RVA: usize = rva(0x844E10); // g_marksMaskShapes: its shapes
pub const VALUE_RVA: usize = rva(0x833DBA); // g_marksMaskValue: the shape's number
pub const MASK_LEN: u32 = rc2k_core::marks::masks::MASK_LEN as u32;

/// The two point lists `collect_points_a` and `_b` fill: count, ids, faces.
pub struct PointList {
    pub count: usize,
    pub ids: usize,
    pub faces: usize,
}

pub const POINTS_A: PointList = PointList {
    count: rva(0x884FEC), // g_marksPointsACount
    ids: rva(0x884FF4),   // g_marksPointsA
    faces: rva(0x8851F4), // g_marksPointsAFaces
};
pub const POINTS_B: PointList = PointList {
    count: rva(0x884FF0), // g_marksPointsBCount
    ids: rva(0x8850F4),   // g_marksPointsB
    faces: rva(0x8852F4), // g_marksPointsBFaces
};
pub const MOST_POINTS: u32 = rc2k_core::marks::masks::MOST_POINTS;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

impl PointList {}

/// `marks::collect_points_a` (0x4BD551) and `_b` (0x4BD59F): face edi's points (+0 the
/// count, from +0x10, 16 bytes apart) added to the list unless there already, up to 0x40,
/// each with the face. Leaves eax the count, ebx the last point read (the face's count
/// with none), ecx where the last search stopped, ebp past the last point taken in.
/// Refuses a list of more than 0x40. Converted:
/// `rc2k_core::marks::masks::PointList::add_face`.
pub unsafe fn collect_points(entry: &Regs, list: &PointList) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let count: u32 = game_read(list.count);
        let most = MOST_POINTS as usize;
        let mut points = rc2k_core::marks::masks::PointList {
            count,
            ids: std::slice::from_raw_parts_mut(game_addr(list.ids) as *mut u32, most),
            faces: std::slice::from_raw_parts_mut(game_addr(list.faces) as *mut u32, most),
        };
        points.check()?;
        let face_points = get(g.edi);
        let c = points.add_face(g.edi, face_points, |k| get(g.edi + 0x10 + 0x10 * k))?;
        g.eax = points.count;
        g.ebx = c.last_point.unwrap_or(face_points);
        g.ecx = c.stopped.unwrap_or(g.ecx);
        g.ebp = g.edi.wrapping_add(0x10).wrapping_add(0x10 * c.stepped);
        game_write(list.count, g.eax);
    }
    Ok(g)
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

/// `marks::fill_mask_spans` (0x4BE712): entry esi of the list at edi, each of its spans
/// filled with `g_marksMaskValue`'s low byte into `g_marksMask`, a row (0x100) further each
/// time. Every register kept. Refuses a span outside the mask, before writing. Converted:
/// `rc2k_core::marks::masks::ShapeRows::fill`.
pub unsafe fn fill_mask_spans(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let rows = shape_rows(entry.edi, entry.esi);
        rows.fit()?;
        let value = game_read::<u32>(VALUE_RVA) as u8;
        rows.fill(mask(game_read(MASK_RVA)), value)?;
    }
    Ok(*entry)
}

/// `marks::draw_mask_shape` (0x4BE6FB): shape esi of the shapes at ebp: `g_marksMaskValue`
/// esi, its two entries (2 esi, 2 esi + 1) of the rows at +8 filled. Leaves esi 2 esi + 1,
/// edi +8's list. Refuses a span outside the mask, before writing. Converted:
/// `rc2k_core::marks::masks::draw_shape`.
pub unsafe fn draw_mask_shape(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let entries = get(g.ebp.wrapping_add(8));
        let first = g.esi.wrapping_add(g.esi);
        let rows = [
            shape_rows(entries, first),
            shape_rows(entries, first.wrapping_add(1)),
        ];
        rows[0].fit()?;
        rows[1].fit()?;
        rc2k_core::marks::masks::draw_shape(
            mask(game_read(MASK_RVA)),
            [&rows[0], &rows[1]],
            g.esi,
        )?;
        game_write(VALUE_RVA, g.esi);
        g.edi = entries;
        g.esi = first.wrapping_add(1);
    }
    Ok(g)
}

/// `marks::draw_mask` (0x4BE6C2): the mask esi (into `g_marksMask`) all 0xFF, then each of
/// the +4 shapes at ebp (`g_marksMaskShapes`) drawn (`draw_mask_shape`). Leaves eax the
/// count, ecx 0, esi and edi as the last shape leaves them. Refuses a count of 0 or less
/// (the original's loop tests after counting and would run round) or a span outside the
/// mask, before writing. Converted: `rc2k_core::marks::masks::draw_mask`.
pub unsafe fn draw_mask(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let count = get(g.ebp.wrapping_add(4)) as i32;
        if count <= 0 {
            return Err(BadInput("a mask of no shapes: the loop would run round").into());
        }
        let entries = get(g.ebp.wrapping_add(8));
        let mut rows = Vec::new();
        for k in 0..2 * count as u32 {
            let r = shape_rows(entries, k);
            r.fit()?;
            rows.push(r);
        }
        rc2k_core::marks::masks::draw_mask(mask(g.esi), &rows)?;
        game_write(MASK_RVA, g.esi);
        game_write(SHAPES_RVA, g.ebp);
        game_write(VALUE_RVA, count as u32 - 1);
        g.eax = count as u32;
        g.ecx = 0;
        g.edi = entries;
        g.esi = 2 * (count as u32 - 1) + 1;
    }
    Ok(g)
}

// ---- for the self-tests ----

const LIST_RVA: usize = rva(0x82E346); // stru_82E346: the work block's list
const WORK_LEN: u32 = 0x8_0000;
/// The two shapes' ring-angle tables, masks and pixel tables.
const SHAPES: [usize; 2] = [rva(0x82FC56), rva(0x82FC7A)];
const RING_ANGLES: [usize; 2] = [rva(0x843DC2), rva(0x8441CA)];
const MASKS: [usize; 2] = [rva(0x833DC2), rva(0x83BDC2)];
const PIXEL_TABLES: [usize; 2] = [rva(0x844E58), rva(0x864E58)];

/// `marks::make_masks` (0x4BDE9A), in `sub_4C0F2E`: the 0x80000-byte work block taken on
/// its list (place 0 of the part); `make_shape_sets` (1), `make_corners` (2),
/// `reset_mask_cursor` (3); each shape's ring (`span_ring`, 4, its corner table in
/// `g_marksRingCorners`), mask (`draw_mask`, 5) and pixels (`map_shapes`, 6); the spans
/// made counted into eax and the list freed (`memory::free_list`, 7). Leaves eax that
/// count, ebx the list, the rest as the last step. Refuses as its steps do, mid-way (the
/// work block is on the list, which the original's run frees with its own).
pub unsafe fn make_masks(entry: &Regs) -> Result<Regs, Refused> {
    use crate::marks::corners::CORNERS_RVA;
    use crate::marks::rings::CORNERS_RVA as RING_CORNERS_RVA;
    let mut g = *entry;
    let a = |r: usize| game_addr(r) as u32;
    unsafe {
        g.eax = WORK_LEN;
        g.ebx = a(LIST_RVA);
        crate::marks::masks::game_part(0, &mut g)?;
        game_write(WORK_RVA, g.eax);
        crate::marks::masks::game_part(1, &mut g)?;
        crate::marks::masks::game_part(2, &mut g)?;
        crate::marks::masks::game_part(3, &mut g)?;
        for n in 0..2 {
            g.ebp = a(SHAPES[n]);
            g.esi = a(RING_ANGLES[n]);
            game_write(RING_CORNERS_RVA, a(CORNERS_RVA[n]));
            crate::marks::masks::game_part(4, &mut g)?;
        }
        for n in 0..2 {
            g.ebp = a(SHAPES[n]);
            g.esi = a(MASKS[n]);
            crate::marks::masks::game_part(5, &mut g)?;
        }
        for n in 0..2 {
            g.ebp = a(SHAPES[n]);
            g.esi = a(PIXEL_TABLES[n]);
            g.edi = a(CORNERS_RVA[n]);
            crate::marks::masks::game_part(6, &mut g)?;
        }
        g.eax = game_read::<u32>(CURSOR_RVA).wrapping_sub(game_read(WORK_RVA)) >> 3;
        g.ebx = a(LIST_RVA);
        crate::marks::masks::game_part(7, &mut g)?;
    }
    Ok(g)
}

/// In the game: the steps' Rust versions (place 0 `memory::allocate`, 7
/// `memory::free_list`).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let keep = |r: &mut Regs, left: Regs| {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        };
        match place {
            0 => return crate::race::level_init::game_part(0, r),
            1 => keep(r, crate::marks::shape_points::make_shape_sets(r)?),
            2 => keep(r, crate::marks::corners::make_corners(r)?),
            3 => reset_mask_cursor(),
            4 => keep(r, crate::marks::rings::span_ring(r)?),
            5 => keep(r, draw_mask(r)?),
            6 => keep(r, crate::marks::pixels::map_shapes(r)?),
            _ => crate::memory::free_list::free_list(r.ebx as *mut crate::memory::MemoryList)?,
        }
    }
    Ok(())
}
