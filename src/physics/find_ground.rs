//! `physics::find_ground` (0x441E29): `call physics::ground_search; retn`.
//!
//! In:  the point (`dword_639824`, `dword_63982C`), the face mask `dword_620470`, the
//!      last face `dword_5CCC78`
//!
//! See `physics/ground.rs`. Callers (7): `physics::probe_ground` (the car's ground points),
//! `physics::move_slot`, `drive::animate_lit_faces`, `race::shadow_points`,
//! `physics::car_lighting`, `drive::simple_car_step`, `physics::count_ground_probe`. Refuses what
//! `ground_search` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::ground_search::ground_search;
use crate::race::owner::Race;

/// The carry (the registers are kept).
pub fn find_ground(race: &Race, scene: &crate::scene::owner::Scene) -> Result<bool, Refused> {
    ground_search(race, scene)
}

/// The same on the fields of `q` (a drawing's own: `race::car_shadow`).
pub fn find_ground_with(
    race: &Race,
    q: &crate::physics::ground_query::GroundQuery<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    crate::physics::ground::ground_search_at(race, q, scene)
}
