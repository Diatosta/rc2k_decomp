//! The contact and probe work of the physics step: the face weights' scratch, the 64 probe points'
//! lists (whole numbers, turned floats, heights, faces, shares; stride 0x100 bytes), the contact
//! vectors of `physics::contact_force`, the wall and edge contacts, the lit faces and the draw batch's counts. The probe lists are 0x40 dwords each (the saved x, y,
//! z lists are one 0xC0-dword run). Names are readings unless taken from the game's
//! own.

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ContactWork {
    /// The contact force copied from the record (`physics::crash_contact`, `ground_contact`).
    pub ground_force: [PlainDword; 3],
    /// 0 then 1 in `physics::ground_contact`; shown on the developers' page.
    pub ground_contact_started: PlainDword,
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
    /// `race::set_up_car_points`' count of the car's points.
    pub car_point_count: PlainDword,
    /// The course face tried (`physics::face_contact`, `touch_face`).
    pub face_number: PlainDword,
    /// The object the face names (`physics::face_contact`, `object_hits_car`,
    /// `object_turn_matrix`, `place_sound_object`).
    pub face_object: PlainDword,
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
}

/// The probe points' lists, plain values on the game thread (`Physics::probes`, borrowed
/// for one routine at a time): the 64 points' whole-number places, turned floats, heights, faces and
/// shares, and the point that crossed an edge. Names are readings unless taken from the game's own.
#[derive(Clone)]
pub struct ProbeLists {
    /// The probe points' x as whole numbers x 65536 plus the car's place
    /// (`physics::probe_ground`); `crash_contact`, `crossed_edge_contact`; copied to
    /// `saved_probe_x` (`probe_and_keep`, one 0xC0-dword run with y and z lists).
    pub probe_x: [u32; 64],
    /// The probe points' y as whole numbers (`physics::probe_ground`); the first four are the
    /// wheels, lowered by `race::car_starts`.
    pub probe_y: [u32; 64],
    /// The probe points' z as whole numbers (`physics::probe_ground`, `crash_contact`).
    pub probe_z: [u32; 64],
    /// The ground height under each probe point plus the bump, 0 when none found
    /// (`physics::probe_ground`, `crash_contact`).
    pub probe_ground_height: [u32; 64],
    /// Each probe point's x turned by the crash matrix, as a float (`physics::probe_ground`,
    /// `crash_contact`, `wall_edge_contact`).
    pub probe_turned_x: [f32; 64],
    /// Each probe point's y turned (`physics::probe_ground`).
    pub probe_turned_y: [f32; 64],
    /// Each probe point's z turned (`physics::probe_ground`).
    pub probe_turned_z: [f32; 64],
    /// Each probe point's y less the ground (`physics::probe_ground`); the first four are the
    /// wheels' heights (`wheel_springs`, `lowest_body_point`, `race::car_starts`).
    pub probe_height: [u32; 64],
    /// Whether the ground was found under each probe point (`physics::probe_ground`).
    pub probe_found: [u32; 64],
    /// The probe points' faces saved (`physics::save_car_state`, `restore_saved_car`,
    /// `wall_contacts`).
    pub probe_saved_faces: [u32; 64],
    /// The found face's half flag per probe point (`physics::probe_ground`, `wheel_surfaces`,
    /// `crash_contact`).
    pub probe_face_flip: [u32; 64],
    /// The point's share in the face's cell, first (`physics::probe_ground`, `wheel_surfaces`).
    pub probe_share_u: [f32; 64],
    /// The point's share in the face's cell, second (`physics::probe_ground`, `wheel_surfaces`).
    pub probe_share_v: [f32; 64],
    /// The probe x list kept before the step (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_x: [u32; 64],
    /// The probe z list kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_z: [u32; 64],
    /// The probe points' faces kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_faces: [u32; 64],
    /// The probe point that crossed an edge, -1 for none (`probe_ground`,
    /// `crossed_edge`).
    pub crossed_probe: u32,
}

impl ProbeLists {
    pub const fn new() -> Self {
        ProbeLists {
            probe_x: [0; 64],
            probe_y: [0; 64],
            probe_z: [0; 64],
            probe_ground_height: [0; 64],
            probe_turned_x: [0.0; 64],
            probe_turned_y: [0.0; 64],
            probe_turned_z: [0.0; 64],
            probe_height: [0; 64],
            probe_found: [0; 64],
            probe_saved_faces: [0; 64],
            probe_face_flip: [0; 64],
            probe_share_u: [0.0; 64],
            probe_share_v: [0.0; 64],
            saved_probe_x: [0; 64],
            saved_probe_z: [0; 64],
            saved_probe_faces: [0; 64],
            crossed_probe: 0,
        }
    }
}

impl Default for ProbeLists {
    fn default() -> Self {
        Self::new()
    }
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
