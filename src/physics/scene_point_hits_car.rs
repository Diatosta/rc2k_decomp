//! `physics::scene_point_hits_car` (0x45E7BF): `physics::point_hits_car` for the scene point
//! whose number is the record's second dword (`g_scenePoints` + 12 n), with no offset:
//! `scene_point_hits` is incremented and `object_offset` set to 0.
//!
//! Takes: the record (+4 the point's number), the object
//!
//! Caller: `physics::face_contact`.

use crate::errors::Refused;
use crate::physics::flow::fill_cells;
use crate::physics::point_hits_car::point_hits_car;
use crate::race::owner::Race;

/// For course face `face` (its number: the original's record and object are both the face):
/// (inside, the point's number).
pub fn scene_point_hits_car(race: &Race, face: u32) -> Result<(bool, u32), Refused> {
    let number = crate::race::course_places::face_field::<u32>(&race.course, face, 4)?;
    race.shared.physics_tables.scene_point_hits.set(
        race.shared
            .physics_tables
            .scene_point_hits
            .get()
            .wrapping_add(1),
    );
    fill_cells(&race.physics.contact.object_offset, 0);
    let place = crate::race::scene_points::point_number(&race.course, number)?;
    Ok(({ point_hits_car(race, place, face) }?, number))
}
