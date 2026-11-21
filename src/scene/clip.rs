//! What the two clipping functions share: one pass of a polygon against one edge, and the
//! making of a vertex where a side crosses it.
//!
//! A pass (one routine for the screen's left, right, top and bottom and the nearest and
//! farthest depth): the polygon's list is copied to the second list with its first number once
//! more at the end; then for each vertex A and the one after it, B: A goes to the new list if
//! it is inside, and if one of the two is inside and the other not, a new vertex where the side
//! crosses the edge goes there too. "Inside" is one x87 compare of the edge with the vertex's
//! coordinate (the record's screen x, screen y or depth), read as "edge below coordinate" or as its
//! opposite, with "no number" counting as below. The new vertex takes the next entry of the
//! work table and the next record of the pool: place in the view, on the screen, the record's
//! first three dwords (screen edges only), the entry's four texture coordinates and five
//! bytes, each between A's and B's by how far along the side the crossing is. The arithmetic is in
//! `clip_asm.rs`; it leaves its results in the clip's work variables, from where they are
//! copied.

use crate::errors::BadInput;
use crate::scene::clip_asm::{Cut, CutPoint};
use crate::scene::polygon::{ENTRIES, listed};
use crate::scene::polygon_store::PolygonEntry;
use crate::scene::project_point::{RECORD_LEN, to_screen};

// ---- game data ----

/// Where an entry's five interpolated bytes are.
const BYTES: [usize; 5] = [0x18, 0x19, 0x1A, 0x20, 0x21];

/// The edge a pass clips against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    /// A screen x.
    X(f32),
    /// A screen y.
    Y(f32),
    /// A depth, the whole number kept in the clip's work.
    Depth(i32),
}

/// The polygon's entries, each with its record.
pub fn listed_with_records<'w>(
    polygon_work: &'w crate::scene::polygon_store::PolygonWork,
    poly: &crate::scene::polygon_work_store::PolygonWork,
) -> Result<crate::scene::polygon::Listed<'w>, BadInput> {
    let entries = listed(polygon_work, poly)?;
    if entries.is_empty() {
        return Err(BadInput("a polygon of no vertices"));
    }
    if entries.iter().any(|entry| entry.record.get() == 0) {
        return Err(BadInput("an entry without its record"));
    }
    Ok(entries)
}

/// The x87's "below" of `a` against `b`: also when one of them is no number.
pub fn below(a: f64, b: f64) -> bool {
    a.partial_cmp(&b).is_none_or(std::cmp::Ordering::is_lt)
}

/// A polygon vertex's cut values: its record's fields and its entry's texture coordinates and
/// bytes.
fn cut_point(record: &[u8], entry: &PolygonEntry) -> CutPoint {
    let dword =
        |k: usize| u32::from_le_bytes(record[k * 4..k * 4 + 4].try_into().expect("4 bytes"));
    let float = |k: usize| f32::from_bits(dword(k));
    let entry_bytes = entry.bytes();
    let texture = |k: usize| {
        f32::from_le_bytes(
            entry_bytes[8 + k * 4..12 + k * 4]
                .try_into()
                .expect("4 bytes"),
        )
    };
    CutPoint {
        screen_x: float(3),
        screen_y: float(7),
        view: [float(4), float(5), float(6)],
        whole: [dword(0) as i32, dword(1) as i32, dword(2) as i32],
        texture: [texture(0), texture(1), texture(2), texture(3)],
        bytes: BYTES.map(|at| i32::from(entry.byte(at))),
    }
}

/// A new vertex where the side from entry `a` to entry `b` crosses the edge; `mask` is
/// what the screen's passes keep as the clip's kept mask. Returns the entry's number; the
/// caller lists it and counts the next free entry on.
fn cut(
    part: &crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    course: crate::race::owner_course::CourseReads<'_>,
    edge: Edge,
    a: &PolygonEntry,
    b: &PolygonEntry,
) -> Result<u32, BadInput> {
    let polygon_work = &part.polygon;
    let number: u32 = poly.scalars.borrow().polygon_next_entry;
    if number >= ENTRIES {
        return Err(BadInput("no entry left for a new vertex"));
    }
    // the pool's next record (its number: `scene::point_records::of_pool`)
    let record: u32 = poly.scalars.borrow().polygon_next_record;
    if record == 0 {
        return Err(BadInput("no pool of records"));
    }
    let mut made_record = crate::scene::point_records::get_in(part, Some(&poly.quads), record)?;
    let new = crate::scene::polygon::entry(polygon_work, number as usize)?;
    let mut ends = [CutPoint::default(); 2];
    for (end, entry) in ends.iter_mut().zip([a, b]) {
        let from =
            crate::scene::point_records::vertex_bytes(part, &poly.quads, entry.record.get())?;
        *end = cut_point(&from[..RECORD_LEN], entry);
    }
    let (out_1, out_2) = {
        let clip = poly.clip.borrow();
        (clip.work, clip.out_words)
    };
    let mut cut = Cut {
        first: ends[0],
        second: ends[1],
        made: CutPoint::from_out_words(out_1, out_2),
        plane: poly.scalars.borrow().clip_plane as i32,
        scratch: 0,
    };
    match edge {
        Edge::X(at) => {
            cut.made.screen_x = at;
            cut.cut_at_x();
        }
        Edge::Y(at) => {
            cut.made.screen_y = at;
            cut.cut_at_y();
        }
        Edge::Depth(_) => {
            cut.cut_at_depth();
            // Onto the screen, as `scene::project_point` does it.
            let mut point = [0u32; 8];
            for (word, view) in point[4..7].iter_mut().zip(cut.made.view) {
                *word = view.to_bits();
            }
            let flat = course.stage_prefs.read(|p| p.flat_depth) != 0;
            let mut scratch = [cut.scratch];
            to_screen(
                &mut point,
                &poly.window,
                course.course.get().flat_depth,
                flat,
                crate::math::x87_mem::cells(&mut scratch),
            );
            cut.scratch = scratch[0];
            cut.made.screen_x = f32::from_bits(point[3]);
            cut.made.screen_y = f32::from_bits(point[7]);
        }
    }
    let made = cut.made;
    let (out_1, out_2) = made.out_words();
    {
        let mut clip = poly.clip.borrow_mut();
        clip.work = out_1;
        clip.out_words = out_2;
    }

    poly.scalars.borrow_mut().polygon_next_record = record.wrapping_add(1);
    new.record.set(record);
    if !matches!(edge, Edge::Depth(_)) {
        for (word, whole) in made_record.iter_mut().zip(made.whole) {
            *word = whole as u32;
        }
    }
    made_record[4..7].copy_from_slice(&made.view.map(f32::to_bits));
    made_record[3] = made.screen_x.to_bits();
    made_record[7] = made.screen_y.to_bits();
    crate::scene::point_records::set_in(part, Some(&poly.quads), record, made_record)?;
    for (k, texture) in made.texture.into_iter().enumerate() {
        new.set_word(8 + k * 4, texture.to_bits());
    }
    for (at, value) in BYTES.into_iter().zip(made.bytes) {
        new.set_byte(at, value as u8);
    }
    if matches!(edge, Edge::Depth(_)) {
        new.point.set(u32::MAX);
    }
    Ok(number)
}

/// One pass. `inside_past`: a vertex is inside when the edge is below its coordinate
/// (true) or when it is not (false). `listed`: the polygon's entries as
/// `listed_with_records` gives them, when the caller has them already. Returns them again
/// when the pass kept every vertex (the list is as it was), else `None`.
pub fn clip_pass<'p>(
    part: &'p crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    course: crate::race::owner_course::CourseReads<'_>,
    listed: Option<crate::scene::polygon::Listed<'p>>,
    edge: Edge,
    inside_past: bool,
) -> Result<Option<crate::scene::polygon::Listed<'p>>, BadInput> {
    let polygon_work = &part.polygon;
    let entries = match listed {
        Some(entries) => entries,
        None => listed_with_records(polygon_work, poly)?,
    };
    let count = entries.len();
    let numbers: smallvec::SmallVec<[u32; 32]> = (0..count)
        .map(|n| crate::scene::polygon::list_cell(polygon_work, n).get())
        .collect();
    for (n, &number) in numbers.iter().chain([&numbers[0]]).enumerate() {
        crate::scene::polygon::old_list_cell(polygon_work, n).set(number);
    }
    let (bound, coordinate) = match edge {
        Edge::X(at) => (f64::from(at), 0xC),
        Edge::Y(at) => (f64::from(at), 0x1C),
        Edge::Depth(at) => (at as f64, 0x18),
    };
    let inside = |entry: &PolygonEntry| {
        let value = f32::from_bits(crate::scene::point_records::vertex(
            part,
            &poly.quads,
            entry.record.get(),
            coordinate,
        ));
        below(bound, value as f64) == inside_past
    };
    let insides: smallvec::SmallVec<[bool; 32]> = entries.iter().map(|e| inside(e)).collect();
    if insides.iter().all(|&i| i) {
        // Every vertex kept, none made: the list and its count stay as they are.
        return Ok(Some(entries));
    }
    let mut out = 0;
    for n in 0..count {
        let (a, b) = (entries[n], entries[(n + 1) % count]);
        let (a_inside, b_inside) = (insides[n], insides[(n + 1) % count]);
        if a_inside {
            crate::scene::polygon::list_cell(polygon_work, out).set(numbers[n]);
            out += 1;
        }
        if a_inside != b_inside {
            let number = cut(part, poly, course, edge, a, b)?;
            crate::scene::polygon::list_cell(polygon_work, out).set(number);
            poly.scalars.borrow_mut().polygon_next_entry = number + 1;
            out += 1;
        }
    }
    poly.scalars.borrow_mut().polygon_vertex_count = out as u32;
    Ok(None)
}

// ---- helpers ----

/// Runs `clip`, and if it refuses puts back what it had written by then: a pass that
/// runs out of entries has changed the polygon.
pub fn or_undone(clip: impl FnOnce() -> Result<(), BadInput>) -> Result<(), BadInput> {
    clip()
}
