//! The scene's lighting and fog state in the race block: the lit place, the turned car light, the
//! view and draw flags, the best-line checkpoint place, the fog values and its 0x1000-byte table,
//! the noise samples, the camera frame values and the polygon hit work. Stops before the error list
//! (0x2000 bytes, `race::level_init`) and leaves out one lone word after the fog.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LightState {
    /// The place (x, y, z) being lit, a car or a point (`scene::add_point_lights`,
    /// `race::frame_draw`, `race::course_faces`). Unused: each view has its own in
    /// `scene::object_light`.
    pub lit_place: [PlainDword; 3],
    /// The turned light direction seen by the car lights (`scene::add_car_light`,
    /// `race::frame_draw`). Unused: each view has its own in `scene::object_light`.
    pub turned_light: [PlainDword; 3],
    /// The draw mode: 0 normal (set by `scene::draw_car_object`, the carousel and the
    /// highlighted object); 1 lowers each projected point by a fixed amount
    /// (`scene::project_point`) and, with 2, skips the sky
    /// (`race::frame_draw::view::draw_sky`); non-0 changes how `scene::draw_object_face` and
    /// `scene::draw_scene_object` draw. Nothing sets 1 or 2 (probably).
    pub draw_mode: PlainDword,
    /// The view kind (2 in described objects) (`scene::draw_object_face`, `scene::fog_polygon`,
    /// `race::frame_draw`). Unused: each view has its own in `scene::object_light`.
    pub view_kind: PlainDword,
    /// `race::place_object_matrix`'s switch: 0 turns the object's matrix by the angles in
    /// the object work angles (y, x, z); non-0 uses the material rules (facing the view, turned
    /// by the view's angles) (`race::frame_draw::scene_objects`).
    pub light_turn_off: PlainDword,
    /// The checkpoint the car is on, on the best line (`race::start_place`,
    /// `physics::reset_car`, `race::best_line`).
    pub line_place: PlainDword,
    /// The fraction on to the next checkpoint, / 65536 (`race::start_place`,
    /// `physics::reset_car_on_line`).
    pub line_fraction: PlainDword,
    /// The fog's near distance; never read, no longer written (`scene::fog`).
    pub fog_near: PlainDword,
    /// The fog's far value, now each view's (`scene::fog::Fog::far`); unused.
    pub fog_far: PlainDword,
    /// The face bits of the object being drawn (`scene::draw_object_face`). Unused: each view
    /// has its own in `scene::object_light`.
    pub face_bits: PlainDword,
    /// The faded flag; the drawing's is the view's (`race::face_work`), this one not used.
    pub faded: PlainDword,
    /// 0x1000 bytes of fog levels, now each view's (`scene::fog::Fog`); unused.
    pub fog_table: [PlainDword; 1024],
    /// Now each view's (`scene::fog::Fog::length`); unused.
    pub fog_table_length: PlainDword,
    /// The camera shake's first component, added to view A's / view B's camera offset
    /// (`physics::take_first_view`, `physics::take_second_view`). Never written (0 from the
    /// start): the shake is all in `camera_shake_y`. Probably x.
    pub camera_shake_x: PlainDword,
    /// The camera shake's second component: the car's bump value (`vertical_speed_change`) times
    /// the physics table's scale, held within plus / minus its limit, 0 when the car stands still;
    /// added to the camera offsets. Probably y, the vertical (`physics::camera_frame`).
    pub camera_shake_y: PlainDword,
    /// The camera shake's third component; never written (0 from the start), added to the
    /// camera offsets. Probably z.
    pub camera_shake_z: PlainDword,
    /// The shake's strength for the sound: a random number / 65536 / 4096 times 3 times
    /// `camera_shake_y`; 0 outside the shaking camera modes. Read by `drive::set_listener`:
    /// added to the steering angle / 0x20000000 to make `listener_scale`
    /// (`physics::camera_frame`).
    pub camera_shake_listener: PlainFloat,
    /// The polygon hit test's work (`scene::draw_polygon`).
    pub hit_work: [PlainDword; 6],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightState {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            light_turn_off: PlainDword::new(1),
            ..ZeroCells::ZERO
        }
    }
}
