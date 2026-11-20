//! The camera's work state and the car parts' axle figures: the lights file's path text, the two
//! views' figures (heading, roll, pitch, turns, offsets, points, shares), the blended camera offset
//! and turn, the pitch ease limits, the ground probe counts, the axle figures, the listener's
//! points and the car draw flags. The stage prefs buffer (`race::prefs_store::StagePrefs`) is a
//! separate piece. Names are readings unless taken from the game's own.

use crate::memory::session::{PlainDword, PlainFloat, PlainInt};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CameraWork {
    /// The text `files\s\data\ldata` (19 bytes + pad, read in place; `race::stage_prefs`).
    pub lights_path: [PlainDword; 5],
    /// The view's zoom handed on (a reading; pushed by `physics::update_camera`).
    pub camera_zoom_kept: PlainFloat,
    /// A view figure the camera arithmetic adds and subtracts (a reading;
    /// `physics::update_camera`).
    pub view_shift_a: PlainFloat,
    /// A further view figure (a reading; `physics::update_camera`).
    pub view_shift_b: PlainFloat,
    /// A further view figure (a reading; `physics::update_camera`).
    pub view_shift_c: PlainFloat,
    /// A further view figure (a reading; `physics::update_camera`).
    pub view_shift_d: PlainFloat,
    /// The camera's offset, blended between the two views, x 65536 (`physics::update_camera`,
    /// `physics::place_camera`).
    pub camera_offset_x: PlainDword,
    /// The camera's offset, y (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_y: PlainDword,
    /// The camera's offset, z (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_z: PlainDword,
    /// The start's heading, also the heading the camera eases to (`race::car_starts`,
    /// `physics::ease_view_turn`).
    pub start_heading: PlainDword,
    /// The start's roll, and the roll turn target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_roll: PlainDword,
    /// The start's pitch, and the pitch target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_pitch: PlainDword,
    /// The first view's heading, from the car (`physics::take_first_view`).
    pub view_a_heading: PlainDword,
    /// The first view's roll, halved with the mode's flag (`physics::take_first_view`).
    pub view_a_roll: PlainInt,
    /// The first view's pitch, halved with the mode's flag (`physics::take_first_view`).
    pub view_a_pitch: PlainInt,
    /// The second view's heading (`physics::take_second_view`).
    pub view_b_heading: PlainDword,
    /// The second view's roll (`physics::take_second_view`).
    pub view_b_roll: PlainInt,
    /// The second view's pitch (`physics::take_second_view`).
    pub view_b_pitch: PlainInt,
    /// The blended view's turn, x (`physics::update_camera`, `physics::place_camera`).
    pub view_turn_x: PlainDword,
    /// The blended view's turn, y (same).
    pub view_turn_y: PlainDword,
    /// The blended view's turn, z (same).
    pub view_turn_z: PlainDword,
    /// The first view's turn figures (from the view record; `physics::take_first_view`).
    pub view_a_turn: [PlainFloat; 3],
    /// The second view's turn figures (`physics::take_second_view`).
    pub view_b_turn: [PlainFloat; 3],
    /// The first view's offset: the record's offset plus the camera shake
    /// (`physics::take_first_view`).
    pub view_a_offset: [PlainFloat; 3],
    /// The second view's offset (`physics::take_second_view`).
    pub view_b_offset: [PlainFloat; 3],
    /// The first view's point (from the view record; `physics::take_first_view`).
    pub view_a_point: [PlainFloat; 3],
    /// The second view's point (`physics::take_second_view`).
    pub view_b_point: [PlainFloat; 3],
    /// A copy of the first view's figure kept beside it (a reading; `physics::update_camera`).
    pub view_blend_aux: PlainDword,
    /// The pitch's ease share, 1.0 when `ReplayFrames.step_first` is set
    /// (`physics::ease_view_turn`).
    pub view_share_pitch: PlainFloat,
    /// The heading's ease share (same).
    pub view_share_heading: PlainFloat,
    /// The roll's ease share (same).
    pub view_share_roll: PlainFloat,
    /// A whole-number view figure copied on (a reading; `physics::update_camera`).
    pub view_blend_aux_2: PlainDword,
    /// A figure of the first view's record (a reading; `physics::take_first_view`).
    pub view_a_aux: PlainDword,
    /// The first view's shares, copied to the three shares above (`physics::take_first_view`,
    /// `physics::update_camera`).
    pub view_a_shares: [PlainFloat; 3],
    /// Another figure of the first view's record (a reading; `physics::take_first_view`).
    pub view_a_aux_2: PlainDword,
    /// A figure of the second view's record (a reading; `physics::take_second_view`).
    pub view_b_aux: PlainDword,
    /// The second view's shares (`physics::take_second_view`).
    pub view_b_shares: [PlainFloat; 3],
    /// Another figure of the second view's record (a reading; `physics::take_second_view`).
    pub view_b_aux_2: PlainDword,
    /// The pitch the camera eases down to (`physics::ease_pitch_down`,
    /// `physics::camera_avoid_ground`).
    pub pitch_floor: PlainDword,
    /// The pitch the camera eases up to; starts as `camera_ground::PITCH_START`
    /// (`physics::ease_pitch_up`).
    pub pitch_ceiling: PlainDword,
    /// The last pitch step, 0 ends the search (`physics::ease_pitch_up`,
    /// `physics::ease_pitch_down`).
    pub pitch_step: PlainDword,
    /// Ground probes made, `physics::count_ground_probe` (a reading: statistics).
    pub ground_probes: PlainDword,
    /// Probes that found ground (`physics::count_ground_probe`).
    pub ground_found: PlainDword,
    /// Probes that found the last face again (`physics::count_ground_probe`).
    pub ground_same: PlainDword,
    /// The car parts' axle figure block (`race::car_parts`).
    pub car_axles: PlainDword,
    /// The front axle's x, negated (`race::car_parts`).
    pub axle_front_neg: PlainDword,
    /// The front axle's x (`race::car_parts`).
    pub axle_front: PlainDword,
    /// The rear axle's lift (`race::car_parts`).
    pub axle_rear_lift: PlainDword,
    /// The axles' height (`race::car_parts`).
    pub axle_height: PlainDword,
    /// The front axle's z (`race::car_parts`).
    pub axle_front_z: PlainDword,
    /// Two points of the car's light record x 65536 / 1000: the sound listener's
    /// (`drive::set_listener`).
    pub listener_points: [PlainDword; 6],
    /// Unused: each view keeps its own (`ctx.view.cars.car_draw_kept`).
    pub car_draw_kept: PlainDword,
    /// The car's faded bits, -1 after; the drawing's is the view's (`race::face_work`), this one
    /// is not used.
    pub faded_bits: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl CameraWork {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        use crate::memory::session::{PlainDword, PlainFloat};
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
