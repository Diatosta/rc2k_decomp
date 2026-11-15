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
    /// The x of the middle of a face's second and third points, which nothing reads (a local
    /// now).
    pub eax_mid_x: PlainFloat,
    /// Unused.
    pub unused_4: PlainDword,
    /// The z of that middle (unused likewise).
    pub eax_mid_z: PlainFloat,
    /// A scene point's x less car 0's, never read: unused.
    pub eax_point_x: PlainDword,
    /// Unused.
    pub unused_10: PlainDword,
    /// The z of the same, never read: unused.
    pub eax_point_z: PlainDword,
    /// A face weight's distance, never read: unused.
    pub eax_distance: PlainFloat,
    /// The numerator of the share `physics::contact_force` makes (a reading).
    pub contact_share_numerator: PlainFloat,
    /// The factor on the normal part `physics::contact_force` adds (a reading).
    pub contact_normal_factor: PlainFloat,
    /// 24 bytes, the lit faces' blink levels by step (`drive::animate_lit_faces`).
    pub blink_levels: [PlainDword; 6],
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
    /// The first four: the height before it is scaled (`physics::probe_ground`); the rest unused
    /// (a reading).
    pub wheel_height_raw: [PlainDword; 64],
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
    /// The probe y list kept (`physics::probe_and_keep`, a reading: copied with the other two).
    pub saved_probe_y: [PlainDword; 64],
    /// The probe z list kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_z: [PlainDword; 64],
    /// The probe points' faces kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_faces: [PlainDword; 64],
    /// The scale `physics::crash_contact` makes for the ground force (held at most
    /// 1).
    pub crash_ground_scale: PlainFloat,
    /// The contact force copied from the record (`physics::crash_contact`, `ground_contact`).
    pub ground_force: [PlainDword; 3],
    /// 0 then 1 in `physics::ground_contact` (a reading).
    pub ground_contact_started: PlainDword,
    /// `drive::animate_lit_faces`: the face's x offset saved before a kind-4 face is
    /// moved, put back when no ground is found under the new place.
    pub lit_face_saved_30: PlainDword,
    /// `drive::animate_lit_faces`: the face's z offset saved the same way.
    pub lit_face_saved_38: PlainDword,
    /// `physics::wall_contacts`: the faces' word +2 bits (& 0xC00) or-ed.
    pub wall_bits: PlainDword,
    /// `drive::animate_lit_faces`: the distance to a lit face, rounded to a whole number.
    pub lit_face_distance: PlainDword,
    /// `drive::animate_lit_faces`: the pace 1 .. 17 from the distance.
    pub lit_face_pace: PlainDword,
    /// `physics::probe_ground`: the noise bytes at the cell's four corners.
    pub ground_noise_corners: [PlainDword; 4],
    /// `physics::probe_ground`: the mixed steps.
    pub ground_noise_mix: [PlainDword; 3],
    /// `physics::probe_ground`: the cell's row and row offset.
    pub ground_cell_row: [PlainDword; 2],
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
    /// Turn speed crossed with the point (`physics::contact_force`).
    pub contact_turn_cross: [PlainFloat; 3],
    /// The point's speed (`physics::contact_force`).
    pub contact_point_speed: [PlainFloat; 3],
    /// The speed along the normal (`physics::contact_force`).
    pub contact_normal_speed: [PlainFloat; 3],
    /// The speed across the normal (`physics::contact_force`).
    pub contact_side_speed: [PlainFloat; 3],
    /// The slide limit as `contact_force` holds it (a reading).
    pub contact_slide_limit_held: PlainFloat,
    /// The slide limit the callers set (0 for none).
    pub contact_slide_limit: PlainFloat,
    /// The speed's length (`contact_force`, `wall_contacts`, `point_hits_car`,
    /// sound bands).
    pub contact_speed_length: PlainFloat,
    /// The across speed's length, into the car's contact value (`contact_force`,
    /// `crash_contact`).
    pub contact_value: PlainFloat,
    /// The normal push (`physics::contact_force`).
    pub contact_normal_push: [PlainFloat; 3],
    /// The slide against the motion (`physics::contact_force`).
    pub contact_slide: [PlainFloat; 3],
    /// The sum turned into the record's force (`physics::contact_force`).
    pub contact_total: [PlainFloat; 3],
    /// A part of the normal added (`physics::contact_force`).
    pub contact_extra: [PlainFloat; 3],
    /// `physics::wall_contacts`: the probe's face before (the wall).
    pub wall_face_before: PlainDword,
    /// `physics::wall_contacts`: the probe's face now.
    pub wall_face_now: PlainDword,
    /// `physics::wall_edge_contact`: the shared edge's first end.
    pub wall_edge_from: PlainDword,
    /// `physics::wall_edge_contact`: its second end.
    pub wall_edge_to: PlainDword,
    /// The face's four corners' x, z (`physics::crossed_edge_contact`).
    pub crossed_corners: [PlainDword; 8],
    /// The probe's before/now figures and face (`physics::crossed_edge_contact`).
    pub crossed_figures: [PlainDword; 5],
    /// The crossed edge's heading (`physics::crossed_edge_contact`).
    pub crossed_heading: PlainDword,
    /// `race::set_up_car_points`' count of the car's points.
    pub car_point_count: PlainDword,
    /// The car's up turned by the crash matrix (`physics::crash_contact`).
    pub crash_car_up: [PlainFloat; 3],
    /// Minus up dotted with the normal (`physics::crash_contact`).
    pub crash_up_dot: PlainFloat,
    /// The push the turn gives along the normal (`contact_force`).
    pub contact_turn_push: PlainFloat,
    /// The depth along the normal (`contact_force`).
    pub contact_depth: PlainFloat,
    /// The share (`contact_force`).
    pub contact_share: PlainFloat,
    /// Work vector, 16 bytes (`physics::contact_force`).
    pub contact_scratch_a: [PlainFloat; 4],
    /// Work vector, 16 bytes (`physics::contact_force`).
    pub contact_scratch_b: [PlainFloat; 4],
    /// A factor read as extended (`physics::contact_force`).
    pub contact_factor: PlainFloat,
    /// Unused.
    pub unused_1430: [PlainDword; 3],
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
    /// `physics::probe_ground`: the last probe x float (a reading).
    pub probe_turned_x_last: PlainFloat,
    /// `physics::probe_ground`: the last probe z float (a reading).
    pub probe_turned_z_last: PlainFloat,
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
    /// Scales the places' difference (`physics::bump_cars`).
    pub bump_push_scale: PlainFloat,
    /// The push apart (`physics::bump_cars`).
    pub bump_push: [PlainFloat; 3],
    /// One share of the two cars' speeds (`physics::bump_cars`).
    pub bump_share_a: PlainFloat,
    /// The other share of the two cars' speeds (`physics::bump_cars`).
    pub bump_share_b: PlainFloat,
    /// 1 when a corner is inside (`physics::car_collisions`, `corner_hits_car`).
    pub corner_inside: PlainDword,
    /// The other car's block (`car_collisions`, `bump_cars`).
    pub other_car_block: PlainDword,
    /// The box's size (`physics::corner_hits_car`).
    pub box_size: [PlainFloat; 3],
    /// The corner turned and scaled (`physics::corner_hits_car`, `bump_cars`).
    pub corner_turned: [PlainFloat; 3],
    /// The other car's corner being tried (`physics::car_collisions`, `corner_hits_car`).
    pub corner_in: [PlainFloat; 3],
    /// The eight corners' x (`physics::car_collisions`).
    pub corner_x: [PlainFloat; 8],
    /// The eight corners' y (`physics::car_collisions`).
    pub corner_y: [PlainFloat; 8],
    /// The eight corners' z (`physics::car_collisions`).
    pub corner_z: [PlainFloat; 8],
    /// The matrix between the cars (96 bytes; `car_collisions`, `corner_hits_car`).
    pub corner_matrix: [PlainFloat; 24],
    /// The note of the meeting (`physics::bump_cars`).
    pub bump_note: [PlainDword; 8],
    /// The length of the speed between the cars (`physics::bump_cars`, sound bands).
    pub bump_speed_length: PlainFloat,
    /// The number of the other car (`car_collisions`, `bump_cars`, `place_sound_e1_other`).
    pub other_car_number: PlainDword,
    /// The speed between the cars (`physics::bump_cars`).
    pub bump_relative_speed: [PlainFloat; 3],
    /// The scene batch's index count: nothing counts it up and no code reads or writes it.
    pub unused_1f14: PlainDword,
    /// The scene batch's vertex count, likewise; no code reads or writes it (the field's name
    /// nowhere in Rust), it only keeps the image's layout.
    pub unused_1f18: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ContactWork {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        use crate::memory::session::{PlainDword, PlainFloat};
        Self {
            contact_share_numerator: PlainFloat::new(1.1),
            contact_normal_factor: PlainFloat::new(0.1),
            blink_levels: {
                let mut v: [PlainDword; 6] = ZeroCells::ZERO;
                v[0] = PlainDword::new(0xFEFDFEFF);
                v[1] = PlainDword::new(0x20100FF);
                v[2] = PlainDword::new(0x10203);
                v[3] = PlainDword::new(0xFFFEFF);
                v[4] = PlainDword::new(0x10201);
                v[5] = PlainDword::new(0x100FF);
                v
            },
            bump_push_scale: PlainFloat::new(0.5),
            bump_share_a: PlainFloat::new(0.4),
            bump_share_b: PlainFloat::new(0.6),
            box_size: {
                let mut v: [PlainFloat; 3] = ZeroCells::ZERO;
                v[0] = PlainFloat::new(0.9);
                v[1] = PlainFloat::new(0.8);
                v[2] = PlainFloat::new(1.8);
                v
            },
            corner_x: {
                let mut v: [PlainFloat; 8] = ZeroCells::ZERO;
                v[0] = PlainFloat::new(-0.9);
                v[1] = PlainFloat::new(0.9);
                v[2] = PlainFloat::new(-0.9);
                v[3] = PlainFloat::new(0.9);
                v[4] = PlainFloat::new(-0.9);
                v[5] = PlainFloat::new(0.9);
                v[6] = PlainFloat::new(-0.9);
                v[7] = PlainFloat::new(0.9);
                v
            },
            corner_y: {
                let mut v: [PlainFloat; 8] = ZeroCells::ZERO;
                v[0] = PlainFloat::new(-0.8);
                v[1] = PlainFloat::new(-0.8);
                v[2] = PlainFloat::new(0.8);
                v[3] = PlainFloat::new(0.8);
                v[4] = PlainFloat::new(-0.8);
                v[5] = PlainFloat::new(-0.8);
                v[6] = PlainFloat::new(0.8);
                v[7] = PlainFloat::new(0.8);
                v
            },
            corner_z: {
                let mut v: [PlainFloat; 8] = ZeroCells::ZERO;
                v[0] = PlainFloat::new(-1.8);
                v[1] = PlainFloat::new(-1.8);
                v[2] = PlainFloat::new(-1.8);
                v[3] = PlainFloat::new(-1.8);
                v[4] = PlainFloat::new(1.8);
                v[5] = PlainFloat::new(1.8);
                v[6] = PlainFloat::new(1.8);
                v[7] = PlainFloat::new(1.8);
                v
            },
            ..ZeroCells::ZERO
        }
    }
}
