//! `physics::ground_search` (0x441E2F): the course face under the point (`dword_639824`,
//! `dword_63982C`): `physics::try_last_face`, else `dword_5CCC74` = -1 and
//! `physics::search_cell` on the point's cell and its eight neighbours (`ground::CELLS`,
//! in the original's order) until one has it. `g_groundFound` (0x639830) 1 when found,
//! else 0. All inside `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; CF the found (no caller reads it: they read
//!      `g_groundFound`, `dword_5CCC74`, `dword_639828`)
//!
//! See `physics/ground.rs`. Caller: `physics::find_ground` (0x441E29), its only one.
//! Refuses (before anything) what `find_course_record` or `search_record_faces` would
//! refuse on any of the nine cells (also on cells past the one found).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::ground::*;
use crate::race::owner::Race;

/// The Rust version: the carry (the registers are kept).
pub fn ground_search(race: &Race, scene: &crate::scene::owner::Scene) -> Result<bool, Refused> {
    ground_search_at(race, scene)
}
