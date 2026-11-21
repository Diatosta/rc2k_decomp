//! The contact and probe work of the physics step: the 64 probe points' lists (whole numbers,
//! turned floats, heights, faces, shares) and the contact vector and object-hit chain of
//! `physics::contact_force`. Names are readings unless taken from the game's own.

/// The contact work of the physics step (`Physics::contact`, borrowed for one routine at a time):
/// the deepest probe point, the contact vector `physics::contact_force` takes (normal, point, car
/// speed, turn speed, slide limit) and gives (speed length, contact value), and the chain of the
/// object hit by a face. Floats are held as their bits, as the recorded vectors feed them: a
/// whole number kept in a float's place (`contact_point` after `point_hits_car`) stays exact.
#[derive(Clone, Default)]
pub struct ContactWork {
    /// 0 then 1 in `physics::ground_contact`; shown on the developers' page.
    pub ground_contact_started: u32,
    /// The deepest point's depth (`physics::lowest_body_point`, `crash_contact`,
    /// `ground_contact`, `find_clear_place`).
    pub lowest_depth: u32,
    /// The deepest point's number, -1 for none.
    pub lowest_point: u32,
    /// A share held to 0 .. 1 (`lowest_body_point`, `set_wheel_weights`), a float's bits.
    pub wheel_share: u32,
    /// The surface's normal, three floats' bits (`physics::contact_force`, `crash_contact`,
    /// `wall_edge_contact`, `wall_push_back`).
    pub contact_normal: [u32; 3],
    /// The contact's point in the car frame, three floats' bits (three whole numbers between
    /// `point_hits_car`'s steps).
    pub contact_point: [u32; 3],
    /// The car's speed copied, three floats' bits.
    pub contact_car_speed: [u32; 3],
    /// The car's turn speed, three floats' bits.
    pub contact_turn_speed: [u32; 3],
    /// The slide limit the callers set (0 for none), a float's bits.
    pub contact_slide_limit: u32,
    /// The speed's length (`contact_force`, `wall_contacts`, `point_hits_car`, sound bands), a
    /// float's bits.
    pub contact_speed_length: u32,
    /// The across speed's length, into the car's contact value, a float's bits.
    pub contact_value: u32,
    /// `race::set_up_car_points`' count of the car's points, kept between its calls.
    pub car_point_count: u32,
    /// The course face tried (`physics::face_contact`, `touch_face`).
    pub face_number: u32,
    /// The object the face names (`physics::face_contact`, `object_hits_car`,
    /// `object_turn_matrix`, `place_sound_object`).
    pub face_object: u32,
    /// The contact kind being looked at (`physics::face_contact`, `point_hits_car`).
    pub contact_kind: u32,
    /// The object's offset, whole numbers.
    pub object_offset: [u32; 3],
    /// The point relative to the car, whole numbers.
    pub object_point_offset: [u32; 3],
    /// The point / 65536 turned by the heading, three floats' bits.
    pub object_point_turned: [u32; 3],
    /// The contact's angle byte (`physics::face_contact`, `touch_face`).
    pub face_angle: u32,
    /// The car's speed x 65536 (`physics::face_contact`, `touch_face`).
    pub face_speed: u32,
    /// The point over the scale, three floats' bits.
    pub object_point_scaled: [u32; 3],
}

impl ContactWork {
    /// Every field 0.
    pub const fn new() -> Self {
        ContactWork {
            ground_contact_started: 0,
            lowest_depth: 0,
            lowest_point: 0,
            wheel_share: 0,
            contact_normal: [0; 3],
            contact_point: [0; 3],
            contact_car_speed: [0; 3],
            contact_turn_speed: [0; 3],
            contact_slide_limit: 0,
            contact_speed_length: 0,
            contact_value: 0,
            car_point_count: 0,
            face_number: 0,
            face_object: 0,
            contact_kind: 0,
            object_offset: [0; 3],
            object_point_offset: [0; 3],
            object_point_turned: [0; 3],
            face_angle: 0,
            face_speed: 0,
            object_point_scaled: [0; 3],
        }
    }
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
