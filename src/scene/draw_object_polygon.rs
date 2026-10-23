//! `scene::draw_object_polygon` (0x451F69): draws a polygon of an object, as it is if
//! it lies on the screen and as triangles if it has to be clipped.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//! Out: the original leaves every register changed; the Rust version keeps them
//!
//! The pool of records, the list (0, 1, 2 .. and a 0) and the next free entry are set
//! as `scene::draw_clipped_polygon` sets them; `scene::project_polygon`; then
//! `sub_451EED` says whether a polygon of three vertices or more has one that is
//! nearer than `flt_5C5D90`, or left of or above that same number, or at or past the
//! screen's right or bottom edge. If none, or with exactly three vertices:
//! `dword_639960` + 1 and `scene::draw_clipped_polygon`. Else `dword_639964` + 1, the
//! entries are kept at `unk_976610`, and the polygon is drawn as a fan of triangles:
//! the first three entries as they are, then for n = 3 .. the entries 0, n - 1 and n
//! from the kept ones (`sub_45204A` copies one), each with the number of vertices at
//! 3, where it stays.
//! Three call sites, all in `sub_450BDA`.
//! Refuses a polygon of no vertices or of more than 64 (the kept entries' room).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls of the projection and the drawing note
//! what they are given.
//!
//! Converted 2026-10-21: the test, the list, the choice and the fan through
//! `rc2k_core::scene::polygons` (compares of stored floats: exact).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::scene::clip::{NEXT_ENTRY_RVA, NEXT_RECORD_RVA, POOL_RVA};
use crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game;
use crate::scene::polygon::{ENTRIES_RVA, ENTRY_LEN, LIST_RVA, dword};
use crate::scene::project_polygon::project_polygon;

// ---- game data ----

const KEPT_RVA: usize = rva(0x976610); // unk_976610
const LEAST_RVA: usize = rva(0x5C5D90); // flt_5C5D90
const RIGHT_RVA: usize = rva(0x6241F0); // g_clipScreenRight
const BOTTOM_RVA: usize = rva(0x6241E8); // g_clipScreenBottom
const WHOLE_COUNT_RVA: usize = rva(0x639960); // dword_639960
const SPLIT_COUNT_RVA: usize = rva(0x639964); // dword_639964

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
        let number = |at: usize| f32::from_bits(game_read(at));
        let edges = Edges {
            least: number(LEAST_RVA),
            right: number(RIGHT_RVA),
            bottom: number(BOTTOM_RVA),
        };
        rc2k_core::scene::polygons::needs_clipping(count, &edges, |n| {
            let record = dword(game_addr(ENTRIES_RVA) + n * ENTRY_LEN + 4) as usize;
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
        let count: u32 = game_read(COUNT_RVA);
        if !fits(count) {
            return Err(BadInput("a polygon of no vertices, or of more than 64").into());
        }
        game_write(NEXT_RECORD_RVA, game_addr(POOL_RVA) as u32);
        for (n, number) in first_list(count).into_iter().enumerate() {
            game_write(LIST_RVA + n * 4, number);
        }
        game_write(NEXT_ENTRY_RVA, count);
        step(Step::Project)?;
        if drawn_whole(count, needs_clipping(count)?) {
            game_write(
                WHOLE_COUNT_RVA,
                game_read::<u32>(WHOLE_COUNT_RVA).wrapping_add(1),
            );
            return step(Step::Draw);
        }
        game_write(
            SPLIT_COUNT_RVA,
            game_read::<u32>(SPLIT_COUNT_RVA).wrapping_add(1),
        );
        let entries = game_addr(ENTRIES_RVA);
        let kept = game_addr(KEPT_RVA);
        std::ptr::copy(
            entries as *const u8,
            kept as *mut u8,
            count as usize * ENTRY_LEN,
        );
        game_write(COUNT_RVA, 3u32);
        step(Step::Draw)?;
        game_write(COUNT_RVA, count);
        for triangle in fan(count) {
            for (to, from) in triangle.into_iter().enumerate() {
                std::ptr::copy(
                    (kept + from * ENTRY_LEN) as *const u8,
                    (entries + to * ENTRY_LEN) as *mut u8,
                    ENTRY_LEN,
                );
            }
            game_write(COUNT_RVA, 3u32);
            step(Step::Draw)?;
        }
        Ok(())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let done = unsafe {
        draw_object_polygon(&mut |step| match step {
            Step::Project => project_polygon().map(|_| ()),
            Step::Draw => draw_clipped_polygon_in_game(&entry),
        })
    };
    done?;
    Ok(())
}
