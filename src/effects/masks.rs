//! `effects::make_masks` (called by `effects::load_edges`): the marks' masks. In a work block of
//! 0x80000 bytes it gathers the points of faces and makes shapes of them; each shape's rows
//! of spans are filled into a mask of 0x8000 bytes (rows of 0x100) with the shape's number,
//! 0xFF elsewhere. These are the leaves.

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::race::owner_course::Course;
use crate::session::PlainDword;
use rc2k_core::effects::masks::{ShapeRows, Span};

pub const MASK_LEN: u32 = rc2k_core::effects::masks::MASK_LEN as u32;

/// The two point lists `collect_points` fills: count, ids, faces.
#[derive(Clone, Copy)]
pub struct PointList<'a> {
    pub count: &'a PlainDword,
    pub ids: &'a [PlainDword; 64],
    pub faces: &'a [PlainDword; 64],
}

impl PointList<'_> {
    /// List A.
    pub fn a(drops: &WindscreenDrops) -> PointList<'_> {
        PointList {
            count: &drops.shape_work.work.count_a,
            ids: &drops.shape_work.work.ids_a,
            faces: &drops.shape_work.work.faces_a,
        }
    }

    /// List B.
    pub fn b(drops: &WindscreenDrops) -> PointList<'_> {
        PointList {
            count: &drops.shape_work.work.count_b,
            ids: &drops.shape_work.work.ids_b,
            faces: &drops.shape_work.work.faces_b,
        }
    }
}
pub const MOST_POINTS: u32 = rc2k_core::effects::masks::MOST_POINTS;

/// `collect_points`: face `face`'s points (+0 the count, from +0x10, 16 bytes
/// apart) added to the list unless there already, up to 0x40, each with the face. Refuses a list of
/// more than 0x40. Converted: `rc2k_core::effects::masks::PointList::add_face`.
pub fn collect_points(
    object_faces: &crate::scene::object_faces::ObjectFaces,
    list: &PointList<'_>,
    face: u32,
) -> Result<(), Refused> {
    let most = MOST_POINTS as usize;
    let mut ids: Vec<u32> = (0..most).map(|n| list.ids[n].get()).collect();
    let mut faces: Vec<u32> = (0..most).map(|n| list.faces[n].get()).collect();
    let mut points = rc2k_core::effects::masks::PointList {
        count: list.count.get(),
        ids: &mut ids,
        faces: &mut faces,
    };
    points.check()?;
    let face_points = object_faces.word(face)?;
    let bad = crate::effects::Sticky::default();
    let added = points.add_face(face, face_points, |k| {
        bad.take(object_faces.word(face + 0x10 + 0x10 * k))
    });
    bad.done()?;
    added?;
    list.count.set(points.count);
    for n in 0..most {
        list.ids[n].set(ids[n]);
        list.faces[n].set(faces[n]);
    }
    Ok(())
}
/// `effects::reset_mask_cursor`: the cursor the work block, the runs from the start of
/// `run_table` (the runs are offsets in it), the run count 0. Converted:
/// `rc2k_core::effects::masks::MaskCursor::reset`.
pub fn reset_mask_cursor(masks: &Masks) {
    // The work's first byte: offset 0.
    let c = rc2k_core::effects::masks::MaskCursor::reset(0, 0);
    masks.vars.cursor.set(c.cursor);
    masks.vars.runs.set(c.runs);
    masks.vars.run_count.set(c.run_count);
}

/// Entry `index` of a list of shapes' rows (`entries` is the offset of the list in `run_table`): 16
/// bytes each, +0 the spans (x, length; 8 bytes each), +4 the first row, +8 the count (signed).
fn shape_rows(masks: &Masks, entries: u32, index: u32) -> Result<ShapeRows, Refused> {
    Ok({
        let entry = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, first_row, count) = (
            crate::effects::spans::run_get(masks, entry)?,
            crate::effects::spans::run_get(masks, entry + 4)?,
            crate::effects::spans::run_get(masks, entry + 8)?,
        );
        ShapeRows {
            first_row,
            spans: (0..(count as i32).max(0) as u32)
                .map(|i| {
                    Ok(Span {
                        x: work_get(masks, spans.wrapping_add(8 * i))?,
                        len: work_get(masks, spans.wrapping_add(8 * i + 4))?,
                    })
                })
                .collect::<Result<_, BadInput>>()?,
        }
    })
}

/// `effects::draw_mask`: shape `n`'s mask (`MASKS.mask_a`, `_b`) all 0xFF, then each of the
/// shape's +4 quads drawn (`rc2k_core::effects::masks::draw_shape`); the shape's number is held
/// in the picks.
/// Refuses a count of 0 or less or a span outside the mask, before writing. Converted:
/// `rc2k_core::effects::masks::draw_mask`.
pub fn draw_mask(masks: &Masks, drops: &WindscreenDrops, n: u32) -> Result<(), Refused> {
    {
        let shapes = masks
            .vars
            .shapes
            .get(n as usize)
            .ok_or(BadInput("a shape's number that names none"))?;
        let place = [&masks.vars.mask_a, &masks.vars.mask_b][n as usize];
        let count = shapes.count.get() as i32;
        if count <= 0 {
            return Err(BadInput("a mask of no shapes: the loop would run round").into());
        }
        let entries = shapes.rows.get();
        let mut rows = Vec::new();
        for k in 0..2 * count as u32 {
            let r = shape_rows(masks, entries, k)?;
            r.fit()?;
            rows.push(r);
        }
        let mut mask = place.read_at(0, MASK_LEN as usize, "a mark mask")?;
        rc2k_core::effects::masks::draw_mask(&mut mask, &rows)?;
        place.write_at(0, &mask, "a mark mask")?;
        drops.picks.mask.set(n);
        drops.picks.shapes.set(n);
        masks.vars.mask_value.set(count as u32 - 1);
    }
    Ok(())
}

pub const WORK_LEN: u32 = 0x8_0000;

// The masks' work (the 0x80000-byte block) is `Masks::work`: the rows of spans
// `span_triangle` makes, reached by byte offsets (the cursor, the runs' +0). Taken by
// `make_masks` counted on its list, given back with it.

/// The work's dword at byte offset `at`.
pub fn work_get(masks: &Masks, at: u32) -> Result<u32, BadInput> {
    masks
        .work
        .with(|w| w.get(at as usize / 4).copied())?
        .filter(|_| at.is_multiple_of(4))
        .ok_or(BadInput("a mask span past the work block"))
}

/// The work's dword at byte offset `at` set to `v`.
pub fn work_put(masks: &Masks, at: u32, v: u32) -> Result<(), BadInput> {
    masks
        .work
        .with(|w| match w.get_mut(at as usize / 4) {
            Some(d) if at.is_multiple_of(4) => {
                *d = v;
                Some(())
            }
            _ => None,
        })?
        .ok_or(BadInput("a mask span past the work block"))
}

/// `effects::make_masks`, in `effects::load_edges`: the 0x80000-byte work block taken on its
/// list (`memory::allocate`); `make_shape_sets`, `make_corners`, `reset_mask_cursor`; each
/// shape's ring (`span_ring`, with its corner table), mask (`draw_mask`) and pixels
/// (`map_shapes`); the list freed (`memory::free_list`). Refuses as its steps do, mid-way
/// (the work block is on the list, which is freed with its own).
pub fn make_masks(
    course: &Course,
    faces_table: &crate::scene::faces_store::Faces,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    // The two shapes' ring-angle tables (each shape's mask, pixel and corner tables are
    // found by its number).
    let ring_angles = [&masks.vars.ring_angles_a, &masks.vars.ring_angles_b];
    let list = &masks.vars.list;
    masks
        .work
        .take(list, WORK_LEN, vec![0; WORK_LEN as usize / 4])?;
    crate::race::load_times::mark("masks: make_shape_sets");
    crate::effects::shape_points::make_shape_sets(course, faces_table, drops, scene, effects)?;
    crate::race::load_times::mark("masks: make_corners");
    crate::effects::corners::make_corners(masks, drops, effects)?;
    reset_mask_cursor(masks);
    crate::race::load_times::mark("masks: span_ring");
    for (n, (shape, angles)) in masks.vars.shapes.iter().zip(ring_angles).enumerate() {
        masks.vars.ring_corners.set(n as u32);
        crate::effects::rings::span_ring(masks, shape, angles, effects)?;
    }
    crate::race::load_times::mark("masks: draw_mask");
    for n in 0..2 {
        draw_mask(masks, drops, n as u32)?;
    }
    crate::race::load_times::mark("masks: map_shapes");
    for n in 0..2 {
        crate::effects::pixels::map_shapes(masks, drops, n as u32, effects)?;
    }
    #[cfg(feature = "remote")]
    crate::race::course_hash::log_mask_hashes(masks, drops, effects);
    crate::memory::free_list::free_list(list)?;
    masks.work.give_back();
    Ok(())
}
