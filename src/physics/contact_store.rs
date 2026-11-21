//! The contact and probe work of the physics step: the face weights' scratch, the 64 probe points'
//! lists (whole numbers, turned floats, heights, faces, shares; stride 0x100 bytes), the contact
//! vectors of `physics::contact_force`, the wall and edge contacts, the lit faces, the car-to-car
//! corner work and the draw batch's counts. The probe lists are 0x40 dwords each (the saved x, y,
//! z lists are one 0xC0-dword run). It stops before the draw batch's vertex buffer
//! (`DrawBatches.scene_vertices`, 0x4000 bytes). Names are readings unless taken from the game's
//! own.

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ContactWork {
    /// The probe points' x as whole numbers x 65536 plus the car's place
    /// (`physics::probe_ground`); `crash_contact`, `crossed_edge_contact`; copied to
    /// `saved_probe_x` (`probe_and_keep`, one 0xC0-dword run with y and z lists).
    pub probe_x: [PlainDword; 64],
    /// The probe points' y as whole numbers (`physics::probe_ground`); the first four are the
    /// wheels, lowered by `race::car_starts`.
    pub probe_y: [PlainDword; 64],
    /// The probe points' z as whole numbers (`physics::probe_ground`, `crash_contact`).
    pub probe_z: [PlainDword; 64],
    /// The ground height under each probe point plus the bump, 0 when none found
    /// (`physics::probe_ground`, `crash_contact`).
    pub probe_ground_height: [PlainDword; 64],
    /// Each probe point's x turned by the crash matrix, as a float (`physics::probe_ground`,
    /// `crash_contact`, `wall_edge_contact`).
    pub probe_turned_x: [PlainFloat; 64],
    /// Each probe point's y turned (`physics::probe_ground`).
    pub probe_turned_y: [PlainFloat; 64],
    /// Each probe point's z turned (`physics::probe_ground`).
    pub probe_turned_z: [PlainFloat; 64],
    /// Each probe point's y less the ground (`physics::probe_ground`); the first four are the
    /// wheels' heights (`wheel_springs`, `lowest_body_point`, `race::car_starts`).
    pub probe_height: [PlainDword; 64],
    /// Whether the ground was found under each probe point (`physics::probe_ground`).
    pub probe_found: [PlainDword; 64],
    /// The probe points' faces saved (`physics::save_car_state`, `restore_saved_car`,
    /// `wall_contacts`).
    pub probe_saved_faces: [PlainDword; 64],
    /// The found face's half flag per probe point (`physics::probe_ground`, `wheel_surfaces`,
    /// `crash_contact`).
    pub probe_face_flip: [PlainDword; 64],
    /// The point's share in the face's cell, first (`physics::probe_ground`, `wheel_surfaces`).
    pub probe_share_u: [PlainFloat; 64],
    /// The point's share in the face's cell, second (`physics::probe_ground`, `wheel_surfaces`).
    pub probe_share_v: [PlainFloat; 64],
    /// The probe x list kept before the step (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_x: [PlainDword; 64],
    /// The probe z list kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_z: [PlainDword; 64],
    /// The probe points' faces kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_faces: [PlainDword; 64],
    /// The contact force copied from the record (`physics::crash_contact`, `ground_contact`).
    pub ground_force: [PlainDword; 3],
    /// 0 then 1 in `physics::ground_contact`; shown on the developers' page.
    pub ground_contact_started: PlainDword,
    /// `drive::animate_lit_faces`: the face's x offset saved before a kind-4 face is
    /// moved, put back when no ground is found under the new place.
    pub lit_face_saved_30: PlainDword,
    /// `drive::animate_lit_faces`: the face's z offset saved the same way.
    pub lit_face_saved_38: PlainDword,
    /// The deepest point's depth (`physics::lowest_body_point`, `crash_contact`,
    /// `ground_contact`, `find_clear_place`).
    pub lowest_depth: PlainDword,
    /// The deepest point's number, -1 for none.
    pub lowest_point: PlainDword,
    /// A share held to 0 .. 1 (`lowest_body_point`, `set_wheel_weights`).
    pub wheel_share: PlainFloat,
    /// The surface's normal (`physics::contact_force`, `crash_contact`, `wall_edge_contact`,
    /// `wall_push_back`).
    pub contact_normal: [PlainFloat; 3],
    /// The contact's point in the car frame (`physics::contact_force`, `crash_contact`,
    /// `face_contact`, `bump_cars`).
    pub contact_point: [PlainFloat; 3],
    /// The car's speed copied (`physics::face_contact`, `wall_edge_contact`,
    /// `crossed_edge_contact`).
    pub contact_car_speed: [PlainFloat; 3],
    /// The car's turn speed (`physics::apply_torque`, `crash_contact`, `contact_force`).
    pub contact_turn_speed: [PlainFloat; 3],
    /// The slide limit the callers set (0 for none).
    pub contact_slide_limit: PlainFloat,
    /// The speed's length (`contact_force`, `wall_contacts`, `point_hits_car`,
    /// sound bands).
    pub contact_speed_length: PlainFloat,
    /// The across speed's length, into the car's contact value (`contact_force`,
    /// `crash_contact`).
    pub contact_value: PlainFloat,
    /// `physics::wall_contacts`: the probe's face before (the wall).
    pub wall_face_before: PlainDword,
    /// `physics::wall_contacts`: the probe's face now.
    pub wall_face_now: PlainDword,
    /// The face's four corners' x, z (`physics::crossed_edge_contact`).
    pub crossed_corners: [PlainDword; 8],
    /// The probe's before/now figures and face (`physics::crossed_edge_contact`).
    pub crossed_figures: [PlainDword; 5],
    /// `race::set_up_car_points`' count of the car's points.
    pub car_point_count: PlainDword,
    /// The probe point being handled (`wall_contacts`, `crossed_edge_contact`,
    /// `wall_edge_contact`).
    pub probe_index: PlainDword,
    /// The course face tried (`physics::face_contact`, `touch_face`).
    pub face_number: PlainDword,
    /// The object the face names (`physics::face_contact`, `object_hits_car`,
    /// `object_turn_matrix`, `place_sound_object`).
    pub face_object: PlainDword,
    /// The numbers of the course faces lit (`physics::touch_face`,
    /// `drive::animate_lit_faces`, `race::frame_draw`).
    pub lit_faces: [PlainDword; 256],
    /// How many faces are in the lit list.
    pub lit_face_count: PlainDword,
    /// The contact kind being looked at (`physics::face_contact`,
    /// `point_hits_car`).
    pub contact_kind: PlainDword,
    /// The object's offset, whole numbers (`physics::scene_point_hits_car`, `object_hits_car`,
    /// `point_hits_car`).
    pub object_offset: [PlainDword; 3],
    /// The point relative to the car, whole numbers (`physics::point_hits_car`,
    /// `place_sound_object`).
    pub object_point_offset: [PlainDword; 3],
    /// The point / 65536 turned by the heading (`physics::point_hits_car`, `face_contact`).
    pub object_point_turned: [PlainFloat; 3],
    /// The contact's angle byte (`physics::face_contact`, `touch_face`).
    pub face_angle: PlainDword,
    /// The car's speed x 65536 (`physics::face_contact`, `touch_face`).
    pub face_speed: PlainDword,
    /// The point over the scale (`physics::point_hits_car`, `face_contact`).
    pub object_point_scaled: [PlainFloat; 3],
    /// The probe point that crossed an edge, -1 for none (`probe_ground`,
    /// `crossed_edge`).
    pub crossed_probe: PlainDword,
    /// The other car's block (`car_collisions`, `bump_cars`).
    pub other_car_block: PlainDword,
    /// The corner turned and scaled (`physics::corner_hits_car`, `bump_cars`).
    pub corner_turned: [PlainFloat; 3],
    /// The other car's corner being tried (`physics::car_collisions`, `corner_hits_car`).
    pub corner_in: [PlainFloat; 3],
    /// The matrix between the cars (96 bytes; `car_collisions`, `corner_hits_car`).
    pub corner_matrix: [PlainFloat; 24],
    /// The length of the speed between the cars (`physics::bump_cars`, sound bands).
    pub bump_speed_length: PlainFloat,
    /// The number of the other car (`car_collisions`, `bump_cars`, `place_sound_e1_other`).
    pub other_car_number: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, every field 0.
impl ContactWork {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        <Self as ZeroCells>::ZERO
    }
}
