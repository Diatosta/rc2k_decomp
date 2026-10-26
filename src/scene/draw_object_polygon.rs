//! `scene::draw_object_polygon` (0x451F69): draws a polygon of an object, as it is if
//! it lies on the screen and as triangles if it has to be clipped.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//! Out: the original leaves every register changed; the Rust version keeps them
//!
//! The pool of records, the list (0, 1, 2 .. and a 0) and the next free entry are set as
//! `scene::draw_clipped_polygon` sets them; `scene::project_polygon`; then `sub_451EED` says
//! whether a polygon of three vertices or more has one that is nearer than `flt_5C5D90`, or
//! left of or above that same number, or at or past the screen's right or bottom edge. If none,
//! or with exactly three vertices: `dword_639960` + 1 and `scene::draw_clipped_polygon`. Else
//! `dword_639964` + 1, the entries are kept at `unk_976610`, and the polygon is drawn as a fan
//! of triangles: the first three entries as they are, then for n = 3 .. the entries 0, n - 1
//! and n from the kept ones (`sub_45204A` copies one), each with the number of vertices at 3,
//! where it stays. Three call sites, all in `sub_450BDA`. Refuses a polygon of no vertices or
//! of more than 64 (the kept entries' room).
//!
//! Converted 2026-10-21: the test, the list, the choice and the fan through
//! `rc2k_core::scene::polygons` (compares of stored floats: exact).

use crate::errors::{BadInput, Refused};
use crate::physics::ground_store::GROUND_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::polygon::{ENTRY_LEN, dword};

// ---- game data ----

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Project,
    Draw,
}

/// `sub_451EED` on the work table's records.
unsafe fn needs_clipping(count: u32) -> Result<bool, BadInput> {
    use rc2k_core::scene::polygons::{Edges, Vertex};
    unsafe {
        if (count as i32).wrapping_sub(3) < 0 {
            return Ok(false);
        }
        let edges = Edges {
            least: NUMBERS.zero.get(),
            right: DRAW.clip_screen[3].get(),
            bottom: DRAW.clip_screen[1].get(),
        };
        rc2k_core::scene::polygons::needs_clipping(count, &edges, |n| {
            let record = dword(crate::scene::polygon::entries_addr() + n * ENTRY_LEN + 4) as usize;
            if record == 0 {
                return Err(BadInput("an entry without its record"));
            }
            let value = |at: usize| f32::from_bits(dword(record + at));
            Ok(Vertex {
                x: value(0xC),
                y: value(0x1C),
                depth: value(0x18),
            })
        })
    }
}

// Converted: `rc2k_core::scene::polygons`
pub unsafe fn draw_object_polygon(
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    use rc2k_core::scene::polygons::{drawn_whole, fan, first_list, fits};
    unsafe {
        let count: u32 = DRAW.polygon_vertex_count.bits();
        if !fits(count) {
            return Err(BadInput("a polygon of no vertices, or of more than 64").into());
        }
        DRAW.polygon_next_record
            .set_bits(crate::scene::polygon::pool_addr() as u32);
        for (n, number) in first_list(count).into_iter().enumerate() {
            crate::scene::polygon::list_cell(n).set(number);
        }
        DRAW.polygon_next_entry.set_bits(count);
        step(Step::Project)?;
        if drawn_whole(count, needs_clipping(count)?) {
            GROUND_WORK
                .whole_polygon_count
                .set_bits(GROUND_WORK.whole_polygon_count.bits().wrapping_add(1));
            return step(Step::Draw);
        }
        GROUND_WORK
            .split_polygon_count
            .set_bits(GROUND_WORK.split_polygon_count.bits().wrapping_add(1));
        let entries = crate::scene::polygon::entries_addr();
        let kept = crate::scene::polygon::entries_addr() + 64 * ENTRY_LEN;
        crate::pointers::GamePtr::new(entries as u32).copy_to(
            crate::pointers::GamePtr::new(kept as u32),
            count as usize * ENTRY_LEN,
            "the polygon's entries",
        )?;
        DRAW.polygon_vertex_count.set_bits(3u32);
        step(Step::Draw)?;
        DRAW.polygon_vertex_count.set_bits(count);
        for triangle in fan(count) {
            for (to, from) in triangle.into_iter().enumerate() {
                crate::pointers::GamePtr::new((kept + from * ENTRY_LEN) as u32).copy_to(
                    crate::pointers::GamePtr::new((entries + to * ENTRY_LEN) as u32),
                    ENTRY_LEN,
                    "the polygon's entries",
                )?;
            }
            DRAW.polygon_vertex_count.set_bits(3u32);
            step(Step::Draw)?;
        }
        Ok(())
    }
}
