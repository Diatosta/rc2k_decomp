//! The camera's work and the car parts' axle figures (IDA 0x608B08 .. 0x608CB4): the lights file's path text, the two views' figures (heading, roll, pitch, turns, offsets, points, shares) and the blended camera offset and turn, the pitch ease limits, the ground probe counts, the axle figures, the listener's points, and the car draw flags. `#[repr(C)]`: the derive reads the fields in order. The stage prefs buffer `g_stagePrefs` (0x608CB4 ..) is a separate piece.
//! Names are readings unless IDA's.

use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CameraWork {
    /// (0x608B08): the text `files\s\data\ldata` (19 bytes + pad, read in place; `race::stage_prefs`).
    pub lights_path: [PlainDword; 5],
    /// (0x608B1C): the view's zoom handed on (a reading; pushed by `physics::update_camera`, `dword_608B1C`).
    pub camera_zoom_kept: PlainFloat,
    /// (0x608B20): `flt_608B20`: a view figure the camera arithmetic adds and subtracts (a reading; `physics::update_camera`).
    pub view_shift_a: PlainFloat,
    /// (0x608B24): `flt_608B24` (a reading; `physics::update_camera`).
    pub view_shift_b: PlainFloat,
    /// (0x608B28): `flt_608B28` (a reading; `physics::update_camera`).
    pub view_shift_c: PlainFloat,
    /// (0x608B2C): `flt_608B2C` (a reading; `physics::update_camera`).
    pub view_shift_d: PlainFloat,
    /// (0x608B30): the camera's offset, blended between the two views, x 65536 (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_x: PlainDword,
    /// (0x608B34): the camera's offset, y (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_y: PlainDword,
    /// (0x608B38): the camera's offset, z (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_z: PlainDword,
    /// (0x608B3C): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_34: [PlainDword; 5],
    /// (0x608B50): the start's heading, also the heading the camera eases to (`race::car_starts`, `physics::ease_view_turn`).
    pub start_heading: PlainDword,
    /// (0x608B54): the start's roll, and the roll turn target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_roll: PlainDword,
    /// (0x608B58): the start's pitch, and the pitch target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_pitch: PlainDword,
    /// (0x608B5C): the first view's heading, from the car (`physics::take_first_view`).
    pub view_a_heading: PlainDword,
    /// (0x608B60): the first view's roll, halved with the mode's flag (`physics::take_first_view`).
    pub view_a_roll: PlainDword,
    /// (0x608B64): the first view's pitch, halved with the mode's flag (`physics::take_first_view`).
    pub view_a_pitch: PlainDword,
    /// (0x608B68): the second view's heading (`physics::take_second_view`).
    pub view_b_heading: PlainDword,
    /// (0x608B6C): the second view's roll (`physics::take_second_view`).
    pub view_b_roll: PlainDword,
    /// (0x608B70): the second view's pitch (`physics::take_second_view`).
    pub view_b_pitch: PlainDword,
    /// (0x608B74): the blended view's turn, x (`physics::update_camera`, `physics::place_camera`).
    pub view_turn_x: PlainDword,
    /// (0x608B78): the blended view's turn, y (same).
    pub view_turn_y: PlainDword,
    /// (0x608B7C): the blended view's turn, z (same).
    pub view_turn_z: PlainDword,
    /// (0x608B80): the first view's turn figures (record +0x10 .. +0x18; `physics::take_first_view`).
    pub view_a_turn: [PlainFloat; 3],
    /// (0x608B8C): the second view's turn figures (`physics::take_second_view`).
    pub view_b_turn: [PlainFloat; 3],
    /// (0x608B98): the first view's offset: record +4 .. +0xC plus `dword_637638` .. (`physics::take_first_view`).
    pub view_a_offset: [PlainFloat; 3],
    /// (0x608BB0): the second view's offset (`physics::take_second_view`).
    pub view_b_offset: [PlainFloat; 3],
    /// (0x608BC8): the first view's point (record +0x1C .. +0x24; `physics::take_first_view`).
    pub view_a_point: [PlainFloat; 3],
    /// (0x608BD4): the second view's point (`physics::take_second_view`).
    pub view_b_point: [PlainFloat; 3],
    /// (0x608BE0): copied from `dword_608BF4` (a reading; `physics::update_camera`).
    pub view_blend_aux: PlainDword,
    /// (0x608BE4): the pitch's ease share, 1.0 when `dword_6EF7CC` (`physics::ease_view_turn`).
    pub view_share_pitch: PlainFloat,
    /// (0x608BE8): the heading's ease share (same).
    pub view_share_heading: PlainFloat,
    /// (0x608BEC): the roll's ease share (same).
    pub view_share_roll: PlainFloat,
    /// (0x608BF0): copied from `dword_608C04`, read by `fild` (a reading; `physics::update_camera`).
    pub view_blend_aux_2: PlainDword,
    /// (0x608BF4): the first view's record +0x34 (a reading; `physics::take_first_view`).
    pub view_a_aux: PlainDword,
    /// (0x608BF8): the first view's shares (record +0x28 .. +0x30), copied to the three shares above (`physics::take_first_view`, `physics::update_camera`).
    pub view_a_shares: [PlainFloat; 3],
    /// (0x608C04): the first view's record +0x38 (a reading; `physics::take_first_view`).
    pub view_a_aux_2: PlainDword,
    /// (0x608C08): the second view's record +0x34 (a reading; `physics::take_second_view`).
    pub view_b_aux: PlainDword,
    /// (0x608C0C): the second view's shares (`physics::take_second_view`).
    pub view_b_shares: [PlainFloat; 3],
    /// (0x608C18): the second view's record +0x38 (a reading; `physics::take_second_view`).
    pub view_b_aux_2: PlainDword,
    /// (0x608C1C): the pitch the camera eases down to (`physics::ease_pitch_down`, `physics::camera_avoid_ground`).
    pub pitch_floor: PlainDword,
    /// (0x608C20): the pitch the camera eases up to; starts as `camera_ground::PITCH_START` (`physics::ease_pitch_up`).
    pub pitch_ceiling: PlainDword,
    /// (0x608C24): the last pitch step, 0 ends the search (`physics::ease_pitch_up`, `physics::ease_pitch_down`).
    pub pitch_step: PlainDword,
    /// (0x608C28): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_120: [PlainDword; 5],
    /// (0x608C3C): ground probes made, `physics::count_ground_probe` (a reading: statistics).
    pub ground_probes: PlainDword,
    /// (0x608C40): probes that found ground (`physics::count_ground_probe`).
    pub ground_found: PlainDword,
    /// (0x608C44): probes that found the last face again (`physics::count_ground_probe`).
    pub ground_same: PlainDword,
    /// (0x608C48): `g_carAxles` (`race::car_parts`).
    pub car_axles: PlainDword,
    /// (0x608C4C): the front axle's x, negated (`race::car_parts`).
    pub axle_front_neg: PlainDword,
    /// (0x608C50): the front axle's x (`race::car_parts`).
    pub axle_front: PlainDword,
    /// (0x608C54): the rear axle's lift (`race::car_parts`).
    pub axle_rear_lift: PlainDword,
    /// (0x608C58): the axles' height (`race::car_parts`).
    pub axle_height: PlainDword,
    /// (0x608C5C): the front axle's z (`race::car_parts`).
    pub axle_front_z: PlainDword,
    /// (0x608C60): two points of the car's light record x 65536 / 1000: the sound listener's (`drive::set_listener`).
    pub listener_points: [PlainDword; 6],
    /// (0x608CA8): `dword_608CA8`, set to 0x2100000 while a car is drawn (`race::car_draw`); now each view's (`ctx.view.cars.car_draw_kept`); unused.
    pub car_draw_kept: PlainDword,
    /// (0x608CAC): the car's faded bits (record +0x10), -1 after; the drawing's is the view's (`race::face_work`), this one not used.
    pub faded_bits: PlainDword,
    /// (0x608CB0): 1 while the car body is drawn (`race::car_draw`); read by nothing, no
    /// longer written.
    pub drawing_car: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl CameraWork {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::{PlainDword, PlainFloat};
        Self {
            lights_path: {
                let mut v: [PlainDword; 5] = ZeroCells::ZERO;
                v[0] = PlainDword::new(0x656C6966);
                v[1] = PlainDword::new(0x5C735C73);
                v[2] = PlainDword::new(0x61746164);
                v[3] = PlainDword::new(0x61646C5C);
                v[4] = PlainDword::new(0x6174);
                v
            },
            camera_zoom_kept: PlainFloat::new(384.0),
            view_shift_a: PlainFloat::new(2000.0),
            view_shift_b: PlainFloat::new(384.0),
            view_shift_c: PlainFloat::new(11.1),
            view_shift_d: PlainFloat::new(3.5),
            faded_bits: PlainDword::new(0xFFFFFFFF),
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::race::owner::race().physics.camera(),
        );
    }
}
