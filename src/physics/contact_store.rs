//! The contact and probe work of the physics step, 0x6D1DD8 .. 0x6D3CF4: the EAX face weights' scratch, the 64 probe points' lists (whole numbers, turned floats, heights, faces, shares: stride 0x100 bytes), the contact vectors of `physics::contact_force`, the wall and edge contacts, the lit faces, the car-to-car corner work and the draw batch's counts.
//! The probe lists run from 0x6D1E14 to 0x6D3014 (0x40 dwords each; the saved x, y, z lists are one 0xC0-dword run). 0x6D1DFC is a byte.
//! Stops before the draw batch's vertex buffer (`DrawBatches.scene_vertices`, 0x4000 bytes to 0x6D7CF4).
//! Names are readings unless IDA's.

use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ContactWork {
    /// (0x6D1DD8): the original's `race::eax_face_weight` kept the x of the middle of a face's second and third points here, which nothing read (a local now; kept for the layout).
    pub eax_mid_x: PlainFloat,
    /// (0x6D1DDC): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_4: PlainDword,
    /// (0x6D1DE0): the z of that middle (unused likewise).
    pub eax_mid_z: PlainFloat,
    /// (0x6D1DE4): the original's `point_from_car` x (a scene point's x less car 0's), never read: unused.
    pub eax_point_x: PlainDword,
    /// (0x6D1DE8): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_10: PlainDword,
    /// (0x6D1DEC): the z of the same, never read: unused.
    pub eax_point_z: PlainDword,
    /// (0x6D1DF0): the original's `race::eax_face_weight` distance, never read: unused.
    pub eax_distance: PlainFloat,
    /// (0x6D1DF4): `flt_6D1DF4`, the numerator of the share `physics::contact_force` makes (a reading).
    pub contact_share_numerator: PlainFloat,
    /// (0x6D1DF8): `flt_6D1DF8`, the factor on the normal part `physics::contact_force` adds (a reading).
    pub contact_normal_factor: PlainFloat,
    /// (0x6D1DFC): `byte_6D1DFC`: 24 bytes, the lit faces' blink levels by step (`drive::animate_lit_faces`).
    pub blink_levels: [PlainDword; 6],
    /// (0x6D1E14): The probe points' x as whole numbers x 65536 plus the car's place (`physics::probe_ground`); `crash_contact`, `crossed_edge_contact`; copied to `saved_probe_x` (`probe_and_keep`, one 0xC0-dword run with y and z lists).
    pub probe_x: [PlainDword; 64],
    /// (0x6D1F14): The probe points' y as whole numbers (`physics::probe_ground`); the first four are the wheels, lowered by `race::car_starts`.
    pub probe_y: [PlainDword; 64],
    /// (0x6D2014): The probe points' z as whole numbers (`physics::probe_ground`, `crash_contact`).
    pub probe_z: [PlainDword; 64],
    /// (0x6D2114): The ground height under each probe point plus the bump, 0 when none found (`physics::probe_ground`, `crash_contact`).
    pub probe_ground_height: [PlainDword; 64],
    /// (0x6D2214): Each probe point's x turned by the crash matrix, as a float (`physics::probe_ground`, `crash_contact`, `wall_edge_contact`).
    pub probe_turned_x: [PlainFloat; 64],
    /// (0x6D2314): Each probe point's y turned (`physics::probe_ground`).
    pub probe_turned_y: [PlainFloat; 64],
    /// (0x6D2414): Each probe point's z turned (`physics::probe_ground`).
    pub probe_turned_z: [PlainFloat; 64],
    /// (0x6D2514): Each probe point's y less the ground (`physics::probe_ground`); the first four are the wheels' heights (`wheel_springs`, `lowest_body_point`, `race::car_starts`).
    pub probe_height: [PlainDword; 64],
    /// (0x6D2614): The first four: the height before it is scaled (`physics::probe_ground`); the rest unused (a reading).
    pub wheel_height_raw: [PlainDword; 64],
    /// (0x6D2714): Whether the ground was found under each probe point (`physics::probe_ground`).
    pub probe_found: [PlainDword; 64],
    /// (0x6D2814): The probe points' faces saved (`physics::save_car_state`, `restore_saved_car`, `wall_contacts`).
    pub probe_saved_faces: [PlainDword; 64],
    /// (0x6D2914): The found face's half flag per probe point (`physics::probe_ground`, `wheel_surfaces`, `crash_contact`).
    pub probe_face_flip: [PlainDword; 64],
    /// (0x6D2A14): The point's share in the face's cell, first (`physics::probe_ground`, `wheel_surfaces`).
    pub probe_share_u: [PlainFloat; 64],
    /// (0x6D2B14): The point's share in the face's cell, second (`physics::probe_ground`, `wheel_surfaces`).
    pub probe_share_v: [PlainFloat; 64],
    /// (0x6D2C14): The probe x list kept before the step (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_x: [PlainDword; 64],
    /// (0x6D2D14): The probe y list kept (`physics::probe_and_keep`, a reading: copied with the other two).
    pub saved_probe_y: [PlainDword; 64],
    /// (0x6D2E14): The probe z list kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_z: [PlainDword; 64],
    /// (0x6D2F14): The probe points' faces kept (`physics::probe_and_keep`, `crossed_edge_contact`).
    pub saved_probe_faces: [PlainDword; 64],
    /// (0x6D3014): `flt_6D3014`: the scale `physics::crash_contact` makes for the ground force (held at most 1).
    pub crash_ground_scale: PlainFloat,
    /// (0x6D3018): The contact force copied from the record (`physics::crash_contact`, `ground_contact`).
    pub ground_force: [PlainDword; 3],
    /// (0x6D3028): 0 then 1 in `physics::ground_contact` (a reading).
    pub ground_contact_started: PlainDword,
    /// (0x6D302C): `drive::animate_lit_faces`: the face's x offset (word +0x30) saved before a kind-4 face is moved, put back when no ground is found under the new place.
    pub lit_face_saved_30: PlainDword,
    /// (0x6D3030): `drive::animate_lit_faces`: the face's z offset (word +0x38) saved the same way.
    pub lit_face_saved_38: PlainDword,
    /// (0x6D3038): `physics::wall_contacts`: the faces' word +2 bits (& 0xC00) or-ed.
    pub wall_bits: PlainDword,
    /// (0x6D303C): `drive::animate_lit_faces`: the distance to a lit face (`fistp`).
    pub lit_face_distance: PlainDword,
    /// (0x6D3040): `drive::animate_lit_faces`: the pace 1 .. 17 from the distance.
    pub lit_face_pace: PlainDword,
    /// (0x6D3044): `physics::probe_ground`: the noise bytes at the cell's four corners (written in the order 3044, 3048, 3050, 304C).
    pub ground_noise_corners: [PlainDword; 4],
    /// (0x6D3054): `physics::probe_ground`: the mixed steps (written at 3054, 3058, 305C).
    pub ground_noise_mix: [PlainDword; 3],
    /// (0x6D3060): `physics::probe_ground`: the cell's row and row offset.
    pub ground_cell_row: [PlainDword; 2],
    /// (0x6D3068): `dword_6D3068`: the deepest point's depth (`physics::lowest_body_point`, `crash_contact`, `ground_contact`, `find_clear_place`).
    pub lowest_depth: PlainDword,
    /// (0x6D306C): `dword_6D306C`: the deepest point's number, -1 for none.
    pub lowest_point: PlainDword,
    /// (0x6D3070): `flt_6D3070`: a share held to 0 .. 1 (`lowest_body_point`, `set_wheel_weights`).
    pub wheel_share: PlainFloat,
    /// (0x6D3078): The surface's normal (`physics::contact_force`, `crash_contact`, `wall_edge_contact`, `wall_push_back`).
    pub contact_normal: [PlainFloat; 3],
    /// (0x6D3090): The contact's point in the car frame (`physics::contact_force`, `crash_contact`, `face_contact`, `bump_cars`).
    pub contact_point: [PlainFloat; 3],
    /// (0x6D30A8): The car's speed copied (`physics::face_contact`, `wall_edge_contact`, `crossed_edge_contact`).
    pub contact_car_speed: [PlainFloat; 3],
    /// (0x6D30B4): The car's turn speed (`physics::apply_torque`, `crash_contact`, `contact_force`).
    pub contact_turn_speed: [PlainFloat; 3],
    /// (0x6D30C0): Turn speed crossed with the point (`physics::contact_force`).
    pub contact_turn_cross: [PlainFloat; 3],
    /// (0x6D30CC): The point's speed (`physics::contact_force`).
    pub contact_point_speed: [PlainFloat; 3],
    /// (0x6D30E4): The speed along the normal (`physics::contact_force`).
    pub contact_normal_speed: [PlainFloat; 3],
    /// (0x6D30F0): The speed across the normal (`physics::contact_force`).
    pub contact_side_speed: [PlainFloat; 3],
    /// (0x6D30FC): The slide limit as `contact_force` holds it (a reading).
    pub contact_slide_limit_held: PlainFloat,
    /// (0x6D3100): `flt_6D3100`: the slide limit the callers set (0 for none).
    pub contact_slide_limit: PlainFloat,
    /// (0x6D3108): `flt_6D3108`: the speed's length (`contact_force`, `wall_contacts`, `point_hits_car`, sound bands).
    pub contact_speed_length: PlainFloat,
    /// (0x6D310C): `dword_6D310C`: the across speed's length, into the car's contact value (`contact_force`, `crash_contact`).
    pub contact_value: PlainFloat,
    /// (0x6D3130): The normal push (`physics::contact_force`).
    pub contact_normal_push: [PlainFloat; 3],
    /// (0x6D3140): The slide against the motion (`physics::contact_force`).
    pub contact_slide: [PlainFloat; 3],
    /// (0x6D3150): The sum turned into the record's force (`physics::contact_force`).
    pub contact_total: [PlainFloat; 3],
    /// (0x6D3160): A part of the normal added (`physics::contact_force`).
    pub contact_extra: [PlainFloat; 3],
    /// (0x6D3170): `physics::wall_contacts`: the probe's face before (the wall).
    pub wall_face_before: PlainDword,
    /// (0x6D3174): `physics::wall_contacts`: the probe's face now.
    pub wall_face_now: PlainDword,
    /// (0x6D3178): `physics::wall_edge_contact`: the shared edge's first end.
    pub wall_edge_from: PlainDword,
    /// (0x6D317C): `physics::wall_edge_contact`: its second end.
    pub wall_edge_to: PlainDword,
    /// (0x6D3180): The face's four corners' x, z (`physics::crossed_edge_contact`).
    pub crossed_corners: [PlainDword; 8],
    /// (0x6D31A0): The probe's before/now figures and face (`physics::crossed_edge_contact`; written 31A0 .. 31B0).
    pub crossed_figures: [PlainDword; 5],
    /// (0x6D31B4): The crossed edge's heading (`physics::crossed_edge_contact`).
    pub crossed_heading: PlainDword,
    /// (0x6D31BC): `g_carPointCount`: `race::set_up_car_points`' count of the car's points.
    pub car_point_count: PlainDword,
    /// (0x6D31C4): The car's up turned by the crash matrix (`physics::crash_contact`).
    pub crash_car_up: [PlainFloat; 3],
    /// (0x6D31D0): Minus up dotted with the normal (`physics::crash_contact`).
    pub crash_up_dot: PlainFloat,
    /// (0x6D31D8): `flt_6D31D8`: the push the turn gives along the normal (`contact_force`).
    pub contact_turn_push: PlainFloat,
    /// (0x6D31DC): `flt_6D31DC`: the depth along the normal (`contact_force`).
    pub contact_depth: PlainFloat,
    /// (0x6D31E0): `flt_6D31E0`: the share (`contact_force`).
    pub contact_share: PlainFloat,
    /// (0x6D31E4): Work vector, 16 bytes (`physics::contact_force`).
    pub contact_scratch_a: [PlainFloat; 4],
    /// (0x6D31F4): Work vector, 16 bytes (`physics::contact_force`).
    pub contact_scratch_b: [PlainFloat; 4],
    /// (0x6D3204): A factor read as extended (`physics::contact_force`).
    pub contact_factor: PlainFloat,
    /// (0x6D3208): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_1430: [PlainDword; 3],
    /// (0x6D3214): The probe point being handled (`wall_contacts`, `crossed_edge_contact`, `wall_edge_contact`).
    pub probe_index: PlainDword,
    /// (0x6D3218): The course face tried (`physics::face_contact`, `touch_face`).
    pub face_number: PlainDword,
    /// (0x6D321C): The object the face names (`physics::face_contact`, `object_hits_car`, `object_turn_matrix`, `place_sound_object`).
    pub face_object: PlainDword,
    /// (0x6D3220): `g_litFaces`: the numbers of the course faces lit (`physics::touch_face`, `drive::animate_lit_faces`, `race::frame_draw`).
    pub lit_faces: [PlainDword; 256],
    /// (0x6D3620): `g_litFaceCount`.
    pub lit_face_count: PlainDword,
    /// (0x6D3624): `dword_6D3624`: the contact kind being looked at (`physics::face_contact`, `point_hits_car`).
    pub contact_kind: PlainDword,
    /// (0x6D3628): `physics::probe_ground`: the last probe x float (a reading).
    pub probe_turned_x_last: PlainFloat,
    /// (0x6D3630): `physics::probe_ground`: the last probe z float (a reading).
    pub probe_turned_z_last: PlainFloat,
    /// (0x6D3B40): The object's offset, whole numbers (`physics::scene_point_hits_car`, `object_hits_car`, `point_hits_car`).
    pub object_offset: [PlainDword; 3],
    /// (0x6D3B4C): The point relative to the car, whole numbers (`physics::point_hits_car`, `place_sound_object`).
    pub object_point_offset: [PlainDword; 3],
    /// (0x6D3B58): The point / 65536 turned by the heading (`physics::point_hits_car`, `face_contact`).
    pub object_point_turned: [PlainFloat; 3],
    /// (0x6D3B70): The contact's angle byte (`physics::face_contact`, `touch_face`).
    pub face_angle: PlainDword,
    /// (0x6D3B74): The car's speed x 65536 (`physics::face_contact`, `touch_face`).
    pub face_speed: PlainDword,
    /// (0x6D3B78): The point over the scale (`physics::point_hits_car`, `face_contact`).
    pub object_point_scaled: [PlainFloat; 3],
    /// (0x6D3B84): `dword_6D3B84`: the probe point that crossed an edge, -1 for none (`probe_ground`, `crossed_edge`).
    pub crossed_probe: PlainDword,
    /// (0x6D3B88): `flt_6D3B88`: scales the places' difference (`physics::bump_cars`).
    pub bump_push_scale: PlainFloat,
    /// (0x6D3B9C): The push apart (`physics::bump_cars`).
    pub bump_push: [PlainFloat; 3],
    /// (0x6D3BAC): `flt_6D3BAC` (`physics::bump_cars`).
    pub bump_share_a: PlainFloat,
    /// (0x6D3BB0): `flt_6D3BB0` (`physics::bump_cars`).
    pub bump_share_b: PlainFloat,
    /// (0x6D3BB4): `dword_6D3BB4`: 1 when a corner is inside (`physics::car_collisions`, `corner_hits_car`).
    pub corner_inside: PlainDword,
    /// (0x6D3BB8): `dword_6D3BB8`: the other car's block (`car_collisions`, `bump_cars`).
    pub other_car_block: PlainDword,
    /// (0x6D3BBC): The box's size (`physics::corner_hits_car`).
    pub box_size: [PlainFloat; 3],
    /// (0x6D3BC8): The corner turned and scaled (`physics::corner_hits_car`, `bump_cars`).
    pub corner_turned: [PlainFloat; 3],
    /// (0x6D3BD4): The other car's corner being tried (`physics::car_collisions`, `corner_hits_car`).
    pub corner_in: [PlainFloat; 3],
    /// (0x6D3BE0): The eight corners' x (`physics::car_collisions`).
    pub corner_x: [PlainFloat; 8],
    /// (0x6D3C00): The eight corners' y (`physics::car_collisions`).
    pub corner_y: [PlainFloat; 8],
    /// (0x6D3C20): The eight corners' z (`physics::car_collisions`).
    pub corner_z: [PlainFloat; 8],
    /// (0x6D3C40): `flt_6D3C40`: the matrix between the cars (96 bytes; `car_collisions`, `corner_hits_car`).
    pub corner_matrix: [PlainFloat; 24],
    /// (0x6D3CA0): The note of the meeting (`physics::bump_cars`).
    pub bump_note: [PlainDword; 8],
    /// (0x6D3CD0): `flt_6D3CD0` (`physics::bump_cars`, sound bands).
    pub bump_speed_length: PlainFloat,
    /// (0x6D3CD4): `dword_6D3CD4` (`car_collisions`, `bump_cars`, `place_sound_e1_other`).
    pub other_car_number: PlainDword,
    /// (0x6D3CD8): The speed between the cars (`physics::bump_cars`).
    pub bump_relative_speed: [PlainFloat; 3],
    /// (0x6D3CEC): the scene batch's index count, which the original's `flush_draw_batch` read and cleared and nothing counted up; no code reads or writes it (the field's name nowhere in Rust), it only keeps the image's layout.
    pub unused_1f14: PlainDword,
    /// (0x6D3CF0): the scene batch's vertex count, likewise; no code reads or writes it (the field's name nowhere in Rust), it only keeps the image's layout.
    pub unused_1f18: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ContactWork {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::{PlainDword, PlainFloat};
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
