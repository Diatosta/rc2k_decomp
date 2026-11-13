//! The scene's lighting and fog state in the race block: the lit place, the turned car light, the view
//! and draw flags, the best-line checkpoint place, the fog values and its 0x1000-byte table, the
//! noise samples, the camera frame values and the polygon hit work. Stops before the error list
//! at 0x637660 (0x2000 bytes, `race::level_init`), and leaves out the lone word at 0x636526.
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LightState {
    /// (0x636588): `g_litPlace`, `dword_63658C`, `dword_636590`: the place (x, y, z) being lit, a car or a point (`scene::add_point_lights`, `race::frame_draw`, `race::course_faces`); now each view's (`scene::object_light`); unused.
    pub lit_place: [PlainDword; 3],
    /// (0x636594): nine dwords before the turned light; the original's code does not reference them and nothing in the port does (a gap).
    pub unknown_c: [PlainDword; 9],
    /// (0x6365B8): `dword_6365B8` .. `dword_6365C0`: the turned light direction seen by the car lights (`scene::add_car_light`, `race::frame_draw`); now each view's (`scene::object_light`); unused.
    pub turned_light: [PlainDword; 3],
    /// (0x6365C4): `dword_6365C4`: the draw mode: 0 normal (set by `scene::draw_car_object`, the carousel and the highlighted object); 1 lowers each projected point by a fixed amount (`scene::project_point`) and, with 2, skips the sky (`race::frame_draw::view::draw_sky`); non-0 changes how `scene::draw_object_face` and `scene::draw_scene_object` draw. Nothing in the port sets 1 or 2, so they come from the image's first value only (probably).
    pub draw_mode: PlainDword,
    /// (0x6365C8): `dword_6365C8`: the view kind (2 in described objects) (`scene::draw_object_face`, `scene::fog_polygon`, `race::frame_draw`); now each view's (`scene::object_light`); unused.
    pub view_kind: PlainDword,
    /// (0x6365CC): `dword_6365CC`: the scene state / loading-bar mark (`menu::frame_callback`, `race::loading_bar_frame`).
    pub scene_state: PlainDword,
    /// (0x6365D0): `dword_6365D0`: `race::place_object_matrix`'s switch: 0 turns the object's matrix by the angles in `OBJECT_WORK` (y, x, z); non-0 uses the material rules (facing the view, turned by the view's angles) (`race::frame_draw::scene_objects`).
    pub light_turn_off: PlainDword,
    /// (0x6365D4): four dwords after the light switch; no code references them (a gap).
    pub unknown_4c: [PlainDword; 4],
    /// (0x6365E4): `dword_6365E4`: the checkpoint the car is on, on the best line (`race::start_place`, `physics::reset_car`, `race::best_line`).
    pub line_place: PlainDword,
    /// (0x6365E8): one dword between the line place and its fraction; no code references it (a gap).
    pub unknown_60: PlainDword,
    /// (0x6365EC): `dword_6365EC`: the fraction on to the next checkpoint, / 65536 (`race::start_place`, `physics::reset_car_on_line`).
    pub line_fraction: PlainDword,
    /// (0x6365F0): `dword_6365F0`: the fog's near distance; never read, no longer written (`scene::fog`).
    pub fog_near: PlainDword,
    /// (0x6365F4): `dword_6365F4`: the fog's far value, now each view's (`scene::fog::Fog::far`); unused.
    pub fog_far: PlainDword,
    /// (0x6365F8): four dwords after the fog's far value; no code references them (a gap).
    pub unknown_70: [PlainDword; 4],
    /// (0x636608): `dword_636608`: the face bits of the object being drawn (`scene::draw_object_face`); now each view's (`scene::object_light`); unused.
    pub face_bits: PlainDword,
    /// (0x63660C): `dword_63660C`: the faded flag; the drawing's is the view's (`race::face_work`), this one not used.
    pub faded: PlainDword,
    /// (0x636610): `dword_636610`: the fog's next step, now each view's (`scene::fog::Fog::next`); unused.
    pub fog_next: PlainDword,
    /// (0x636614): two dwords before the fog table; no code references them (a gap).
    pub unknown_8c: [PlainDword; 2],
    /// (0x63661C): `g_fogTable`: 0x1000 bytes of fog levels, now each view's (`scene::fog::Fog`); unused.
    pub fog_table: [PlainDword; 1024],
    /// (0x63761C): `g_fogTableLength`, now each view's (`scene::fog::Fog::length`); unused.
    pub fog_table_length: PlainDword,
    /// (0x637620): `dword_637620` .. `dword_637634`: the four corners, then the x blends, of the noise map: `race::sample_noise`'s own steps, read by nothing else; no longer written (`rc2k_core::race::view::sample_noise` returns the value).
    pub noise_samples: [PlainDword; 6],
    /// (0x637638): `dword_637638`: the camera shake's first component, added to view A's / view B's camera offset (`physics::take_first_view`, `physics::take_second_view`). Never written (0 from the image, nothing stores to it; the original's `physics::camera_frame` cleared it each call): the shake is all in `camera_shake_y`. Probably x.
    pub camera_shake_x: PlainDword,
    /// (0x63763C): `dword_63763C`: the camera shake's second component: the car's bump value (`unknown_90c`) times the physics table's scale, held within plus / minus its limit, 0 when the car stands still; added to the camera offsets. Probably y, the vertical (`physics::camera_frame`).
    pub camera_shake_y: PlainDword,
    /// (0x637640): `dword_637640`: the camera shake's third component; never written (0 from the image; the original's `physics::camera_frame` cleared it each call), added to the camera offsets. Probably z.
    pub camera_shake_z: PlainDword,
    /// (0x637644): `flt_637644`: the shake's strength for the sound: a random number / 65536 / 4096 times 3 times `camera_shake_y`; 0 outside the shaking camera modes. Read by `drive::set_listener`: added to the steering angle / 0x20000000 to make `listener_scale` (`physics::camera_frame`).
    pub camera_shake_listener: PlainFloat,
    /// (0x637648): `dword_637648` .. `dword_63765C`: the polygon hit test's work (`scene::draw_polygon`).
    pub hit_work: [PlainDword; 6],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightState {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            light_turn_off: PlainDword::new(1),
            unknown_4c: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(1),
            ],
            ..ZeroCells::ZERO
        }
    }
}
