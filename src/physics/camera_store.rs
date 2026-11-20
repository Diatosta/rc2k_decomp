//! The camera's work state and the car parts' axle figures: the lights file's path text, the two
//! views' figures (heading, roll, pitch, turns, offsets, points, shares), the blended camera offset
//! and turn, the pitch ease limits, the ground probe counts, the axle figures, the listener's
//! points and the car draw flags. The stage prefs buffer (`race::prefs_store::StagePrefs`) is a
//! separate piece. Names are readings unless taken from the game's own.

#[derive(Clone, Copy)]
pub struct CameraWork {
    /// The text `files\s\data\ldata` (19 bytes + pad, read in place; `race::stage_prefs`).
    pub lights_path: [u32; 5],
    /// The view's zoom handed on (a reading; pushed by `physics::update_camera`).
    pub camera_zoom_kept: f32,
    /// A view figure the camera arithmetic adds and subtracts (a reading;
    /// `physics::update_camera`).
    pub view_shift_a: f32,
    /// A further view figure (a reading; `physics::update_camera`).
    pub view_shift_b: f32,
    /// A further view figure (a reading; `physics::update_camera`).
    pub view_shift_c: f32,
    /// A further view figure (a reading; `physics::update_camera`).
    pub view_shift_d: f32,
    /// The camera's offset, blended between the two views, x 65536 (`physics::update_camera`,
    /// `physics::place_camera`).
    pub camera_offset_x: u32,
    /// The camera's offset, y (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_y: u32,
    /// The camera's offset, z (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_z: u32,
    /// The start's heading, also the heading the camera eases to (`race::car_starts`,
    /// `physics::ease_view_turn`).
    pub start_heading: u32,
    /// The start's roll, and the roll turn target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_roll: u32,
    /// The start's pitch, and the pitch target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_pitch: u32,
    /// The first view's heading, from the car (`physics::take_first_view`).
    pub view_a_heading: u32,
    /// The first view's roll, halved with the mode's flag (`physics::take_first_view`).
    pub view_a_roll: i32,
    /// The first view's pitch, halved with the mode's flag (`physics::take_first_view`).
    pub view_a_pitch: i32,
    /// The second view's heading (`physics::take_second_view`).
    pub view_b_heading: u32,
    /// The second view's roll (`physics::take_second_view`).
    pub view_b_roll: i32,
    /// The second view's pitch (`physics::take_second_view`).
    pub view_b_pitch: i32,
    /// The blended view's turn, x (`physics::update_camera`, `physics::place_camera`).
    pub view_turn_x: u32,
    /// The blended view's turn, y (same).
    pub view_turn_y: u32,
    /// The blended view's turn, z (same).
    pub view_turn_z: u32,
    /// The first view's turn figures (from the view record; `physics::take_first_view`).
    pub view_a_turn: [f32; 3],
    /// The second view's turn figures (`physics::take_second_view`).
    pub view_b_turn: [f32; 3],
    /// The first view's offset: the record's offset plus the camera shake
    /// (`physics::take_first_view`).
    pub view_a_offset: [f32; 3],
    /// The second view's offset (`physics::take_second_view`).
    pub view_b_offset: [f32; 3],
    /// The first view's point (from the view record; `physics::take_first_view`).
    pub view_a_point: [f32; 3],
    /// The second view's point (`physics::take_second_view`).
    pub view_b_point: [f32; 3],
    /// The pitch's ease share, 1.0 when `ReplayFrames.step_first` is set
    /// (`physics::ease_view_turn`).
    pub view_share_pitch: f32,
    /// The heading's ease share (same).
    pub view_share_heading: f32,
    /// The roll's ease share (same).
    pub view_share_roll: f32,
    /// A whole-number view figure copied on (a reading; `physics::update_camera`).
    pub view_blend_aux_2: u32,
    /// A figure of the first view's record (a reading; `physics::take_first_view`).
    pub view_a_aux: u32,
    /// The first view's shares, copied to the three shares above (`physics::take_first_view`,
    /// `physics::update_camera`).
    pub view_a_shares: [f32; 3],
    /// Another figure of the first view's record (a reading; `physics::take_first_view`).
    pub view_a_aux_2: u32,
    /// A figure of the second view's record (a reading; `physics::take_second_view`).
    pub view_b_aux: u32,
    /// The second view's shares (`physics::take_second_view`).
    pub view_b_shares: [f32; 3],
    /// Another figure of the second view's record (a reading; `physics::take_second_view`).
    pub view_b_aux_2: u32,
    /// The pitch the camera eases down to (`physics::ease_pitch_down`,
    /// `physics::camera_avoid_ground`).
    pub pitch_floor: u32,
    /// The pitch the camera eases up to; starts as `camera_ground::PITCH_START`
    /// (`physics::ease_pitch_up`).
    pub pitch_ceiling: u32,
    /// The last pitch step, 0 ends the search (`physics::ease_pitch_up`,
    /// `physics::ease_pitch_down`).
    pub pitch_step: u32,
    /// Ground probes made, `physics::count_ground_probe` (a reading: statistics).
    pub ground_probes: u32,
    /// Probes that found ground (`physics::count_ground_probe`).
    pub ground_found: u32,
    /// Probes that found the last face again (`physics::count_ground_probe`).
    pub ground_same: u32,
    /// The front axle's x, negated (`race::car_parts`).
    pub axle_front_neg: u32,
    /// The front axle's x (`race::car_parts`).
    pub axle_front: u32,
    /// The rear axle's lift (`race::car_parts`).
    pub axle_rear_lift: u32,
    /// The axles' height (`race::car_parts`).
    pub axle_height: u32,
    /// The front axle's z (`race::car_parts`).
    pub axle_front_z: u32,
    /// Two points of the car's light record x 65536 / 1000: the sound listener's
    /// (`drive::set_listener`).
    pub listener_points: [u32; 6],
    /// Unused: each view keeps its own (`ctx.view.cars.car_draw_kept`).
    pub car_draw_kept: u32,
    /// The car's faded bits, -1 after; the drawing's is the view's (`race::face_work`), this one
    /// is not used.
    pub faded_bits: u32,
}

/// The piece, with the image's first values.
impl CameraWork {
    pub const fn new() -> Self {
        Self {
            lights_path: [0x656C6966, 0x5C735C73, 0x61746164, 0x61646C5C, 0x6174],
            camera_zoom_kept: 384.0,
            view_shift_a: 2000.0,
            view_shift_b: 384.0,
            view_shift_c: 11.1,
            view_shift_d: 3.5,
            camera_offset_x: 0,
            camera_offset_y: 0,
            camera_offset_z: 0,
            start_heading: 0,
            start_roll: 0,
            start_pitch: 0,
            view_a_heading: 0,
            view_a_roll: 0,
            view_a_pitch: 0,
            view_b_heading: 0,
            view_b_roll: 0,
            view_b_pitch: 0,
            view_turn_x: 0,
            view_turn_y: 0,
            view_turn_z: 0,
            view_a_turn: [0.0; 3],
            view_b_turn: [0.0; 3],
            view_a_offset: [0.0; 3],
            view_b_offset: [0.0; 3],
            view_a_point: [0.0; 3],
            view_b_point: [0.0; 3],
            view_share_pitch: 0.0,
            view_share_heading: 0.0,
            view_share_roll: 0.0,
            view_blend_aux_2: 0,
            view_a_aux: 0,
            view_a_shares: [0.0; 3],
            view_a_aux_2: 0,
            view_b_aux: 0,
            view_b_shares: [0.0; 3],
            view_b_aux_2: 0,
            pitch_floor: 0,
            pitch_ceiling: 0,
            pitch_step: 0,
            ground_probes: 0,
            ground_found: 0,
            ground_same: 0,
            axle_front_neg: 0,
            axle_front: 0,
            axle_rear_lift: 0,
            axle_height: 0,
            axle_front_z: 0,
            listener_points: [0; 6],
            car_draw_kept: 0,
            faded_bits: 0xFFFF_FFFF,
        }
    }
}
