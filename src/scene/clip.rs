//! What the two clipping functions share: one pass of a polygon against one edge, and the
//! making of a vertex where a side crosses it.
//!
//! A pass (`sub_44D9AF`, `sub_44DA78`, `sub_44D81D`, `sub_44D8E6` for the screen's
//! left, right, top and bottom; `sub_44DB93` and `sub_44DEAA` for the nearest and
//! farthest depth; all six one routine): the polygon's list is copied to
//! `dword_977810` with its first number once more at the end; then for each vertex A
//! and the one after it, B: A goes to the new list if it is inside, and if one of the
//! two is inside and the other not, a new vertex where the side crosses the edge goes
//! there too. "Inside" is one x87 compare of the edge with the vertex's coordinate
//! (the record's +0xC, +0x1C or +0x18), read as "edge below coordinate" or as its
//! opposite, with "no number" counting as below.
//! The new vertex (`sub_44D247` for x, `sub_44D532` for y, in line for the depth) takes
//! the next entry of the work table (`dword_624114`) and the next record of the pool
//! `dword_6240A4`: place in the view, on the screen, the record's first three dwords
//! (screen edges only), the entry's four texture coordinates and five bytes, each
//! between A's and B's by how far along the side the crossing is. The arithmetic is
//! the original's own instructions (`clip_asm.rs`); it leaves its results in the work
//! variables `flt_623F80` .. `flt_623FAC` and `dword_6241AC` .. `dword_6241BC`, from
//! where they are copied.

use crate::errors::BadInput;
use crate::race::course_store::COURSE;
use crate::race::prefs_store::STAGE_PREFS;
use crate::scene::clip_asm::{
    BYTES_A, BYTES_B, CHECKED, ENTRY_A, ENTRY_B, OUT_1, OUT_2, PLANE, RECORD_A, RECORD_B, Work,
    cut_at_depth, cut_at_x, cut_at_y,
};
use crate::scene::draw_store::DRAW;
use crate::scene::polygon::{ENTRIES, ENTRY_LEN, dword, listed, put};
use crate::scene::project_point::{RECORD_LEN, to_screen};

// ---- game data ----

/// Where an entry's five interpolated bytes are.
const BYTES: [usize; 5] = [0x18, 0x19, 0x1A, 0x20, 0x21];

/// The edge a pass clips against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    /// A screen x, as a float's bits.
    X(u32),
    /// A screen y.
    Y(u32),
    /// A depth, the whole number in `dword_62411C`.
    Depth(i32),
}

/// The polygon's entries, each with its record.
pub unsafe fn listed_with_records() -> Result<Vec<usize>, BadInput> {
    let entries = unsafe { listed() }?;
    if entries.is_empty() {
        return Err(BadInput("a polygon of no vertices"));
    }
    if entries
        .iter()
        .any(|&entry| unsafe { dword(entry + 4) } == 0)
    {
        return Err(BadInput("an entry without its record"));
    }
    Ok(entries)
}

/// The x87's "below" of `a` against `b`: also when one of them is no number.
pub fn below(a: f64, b: f64) -> bool {
    a.partial_cmp(&b).is_none_or(|order| order.is_lt())
}

/// A new vertex where the side from entry `a` to entry `b` crosses the edge; `mask` is
/// what the screen's passes keep in `dword_624008`. Returns the entry's number; the
/// caller lists it and counts `dword_624114` on.
unsafe fn cut(edge: Edge, a: usize, b: usize, mask: u32) -> Result<u32, BadInput> {
    unsafe {
        let number: u32 = DRAW.polygon_next_entry.bits();
        if number >= ENTRIES {
            return Err(BadInput("no entry left for a new vertex"));
        }
        let record: u32 = DRAW.polygon_next_record.bits();
        if record == 0 {
            return Err(BadInput("no pool of records"));
        }
        let new = crate::scene::polygon::entries_addr() + number as usize * ENTRY_LEN;
        let mut w: Work = [0; 0x51];
        let base = w.as_mut_ptr() as *mut u8;
        for (entry, record_at, entry_at, bytes_at) in [
            (a, RECORD_A, ENTRY_A, BYTES_A),
            (b, RECORD_B, ENTRY_B, BYTES_B),
        ] {
            let from = dword(entry + 4) as usize as *const u8;
            std::ptr::copy_nonoverlapping(from, base.add(record_at), RECORD_LEN);
            std::ptr::copy_nonoverlapping(entry as *const u8, base.add(entry_at), ENTRY_LEN);
            for (k, at) in BYTES.into_iter().enumerate() {
                w[bytes_at / 4 + k] = ((entry + at) as *const u8).read() as u32;
            }
        }
        let out_1 = crate::session::read_bits(&DRAW.clip_work);
        let out_2 = crate::session::read_bits(&DRAW.clip_work_2);
        w[OUT_1 / 4..OUT_1 / 4 + 12].copy_from_slice(&out_1);
        w[OUT_2 / 4..OUT_2 / 4 + 5].copy_from_slice(&out_2);
        w[CHECKED / 4] = DRAW.scratch.bits();
        w[PLANE / 4] = DRAW.clip_plane.bits();
        match edge {
            Edge::X(at) => {
                DRAW.clip_kept_mask.set_bits(mask);
                w[OUT_1 / 4] = at;
                cut_at_x(&mut w);
            }
            Edge::Y(at) => {
                DRAW.clip_kept_mask.set_bits(mask);
                w[OUT_1 / 4 + 1] = at;
                cut_at_y(&mut w);
            }
            Edge::Depth(_) => {
                cut_at_depth(&mut w);
                // Onto the screen, as `scene::project_point` does it.
                let mut point = [0u32; 8];
                point[4..7].copy_from_slice(&w[OUT_1 / 4 + 2..OUT_1 / 4 + 5]);
                let v = [
                    DRAW.view_scale.bits(),
                    DRAW.view_centre[0].bits(),
                    DRAW.view_centre[1].bits(),
                    COURSE.flat_depth.bits(),
                ];
                let flat = STAGE_PREFS.flat_depth.bits() != 0;
                to_screen(point.as_mut_ptr(), &v, flat, &raw mut w[CHECKED / 4]);
                (w[OUT_1 / 4], w[OUT_1 / 4 + 1]) = (point[3], point[7]);
            }
        }
        let out_1: [u32; 12] = w[OUT_1 / 4..OUT_1 / 4 + 12].try_into().expect("12 dwords");
        let out_2: [u32; 5] = w[OUT_2 / 4..OUT_2 / 4 + 5].try_into().expect("5 dwords");
        crate::session::write_bits(&DRAW.clip_work, out_1);
        crate::session::write_bits(&DRAW.clip_work_2, out_2);
        DRAW.scratch.set_bits(w[CHECKED / 4]);

        DRAW.polygon_next_record
            .set_bits(record.wrapping_add(RECORD_LEN as u32));
        put(new + 4, record);
        let record = record as usize;
        if !matches!(edge, Edge::Depth(_)) {
            for k in 0..3 {
                put(record + k * 4, out_1[5 + k]);
            }
        }
        for k in 0..3 {
            put(record + 0x10 + k * 4, out_1[2 + k]);
        }
        put(record + 0xC, out_1[0]);
        put(record + 0x1C, out_1[1]);
        for k in 0..4 {
            put(new + 8 + k * 4, out_1[8 + k]);
        }
        for (at, value) in BYTES.into_iter().zip([2, 3, 4, 0, 1].map(|k| out_2[k])) {
            ((new + at) as *mut u8).write(value as u8);
        }
        if matches!(edge, Edge::Depth(_)) {
            put(new, u32::MAX);
        }
        Ok(number)
    }
}

/// One pass. `inside_past`: a vertex is inside when the edge is below its coordinate
/// (true) or when it is not (false).
pub unsafe fn clip_pass(edge: Edge, inside_past: bool, mask: u32) -> Result<(), BadInput> {
    unsafe {
        let entries = listed_with_records()?;
        let count = entries.len();
        let numbers: Vec<u32> = (0..count)
            .map(|n| crate::scene::polygon::list_cell(n).get())
            .collect();
        for (n, &number) in numbers.iter().chain([&numbers[0]]).enumerate() {
            crate::scene::polygon::old_list_cell(n).set(number);
        }
        let (bound, coordinate) = match edge {
            Edge::X(at) => (f32::from_bits(at) as f64, 0xC),
            Edge::Y(at) => (f32::from_bits(at) as f64, 0x1C),
            Edge::Depth(at) => (at as f64, 0x18),
        };
        let inside = |entry: usize| {
            let value = f32::from_bits(dword(dword(entry + 4) as usize + coordinate));
            below(bound, value as f64) == inside_past
        };
        let mut out = 0;
        for n in 0..count {
            let (a, b) = (entries[n], entries[(n + 1) % count]);
            let (a_inside, b_inside) = (inside(a), inside(b));
            if a_inside {
                crate::scene::polygon::list_cell(out).set(numbers[n]);
                out += 1;
            }
            if a_inside != b_inside {
                let number = cut(edge, a, b, mask)?;
                crate::scene::polygon::list_cell(out).set(number);
                DRAW.polygon_next_entry.set_bits(number + 1);
                out += 1;
            }
        }
        DRAW.polygon_vertex_count.set_bits(out as u32);
        Ok(())
    }
}

// ---- helpers ----

/// Runs `clip`, and if it refuses puts back what it had written by then: a pass that
/// runs out of entries has changed the polygon.
pub unsafe fn or_undone(clip: impl FnOnce() -> Result<(), BadInput>) -> Result<(), BadInput> {
    clip()
}
