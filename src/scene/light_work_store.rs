//! The scene's light and object-drawing work: the object being drawn, the lamp tint and the point's
//! place, the light-dot inputs, the light's colour and kind, the lamp and ground colours the car
//! lighting keeps, the plain light colours, the material phases' lengths and clock, and the count
//! of listed faces.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LightWork {
    /// The object being drawn. Unused: each view has its own in `scene::object_place`.
    pub drawing_object: PlainDword,
    /// 13 dwords after the drawing object; no code uses them (a gap).
    pub unknown_4: [PlainDword; 13],
    /// The lamp's tint taken from its entry (`scene::add_point_lights`, `race::frame_draw`).
    /// Unused: each view has its own in `scene::object_light`.
    pub lamp_tint: PlainDword,
    /// Three dwords before the point's place; nothing uses them (a gap).
    pub unknown_3c: [PlainDword; 3],
    /// The point's place (`scene::draw_object_face`). Unused: each view has its own in
    /// `scene::object_light`.
    pub point_place: [PlainFloat; 3],
    /// Float work of the lighting (written and read in the object drawing code only).
    pub light_work_floats: [PlainFloat; 4],
    /// The point of `scene::light_dot`'s inputs (floats). Unused: each view has its own in
    /// `scene::object_light`.
    pub light_dot_point: [PlainFloat; 3],
    /// The light's direction vector, the second input of `scene::light_dot`. Unused: each view
    /// has its own in `scene::object_light`.
    pub light_dot_light: [PlainFloat; 3],
    /// The factor in `scene::light_dot`'s inputs. Unused: each view has its own in
    /// `scene::object_light`.
    pub light_dot_factor: PlainFloat,
    /// The light's colour (`scene::vertex_light`, `scene::add_point_lights`,
    /// `race::course_faces`, `race::frame_draw`). Unused: each view has its own in
    /// `scene::object_light`.
    pub light_colour: PlainDword,
    /// One dword between the light's colour and the lamps' mask; no code uses it (a gap).
    pub unknown_84: PlainDword,
    /// The lamp parts' mask (`race::car_parts`, `race::draw_race_scene`, `race::loadlevel`).
    pub lamps_mask: PlainDword,
    /// The lone-group switch / marks mask (`race::car_parts`, `physics::drop_broken_parts`,
    /// `scene::list_object_face`, `race::draw_race_scene`, `race::loadlevel`).
    pub marks_mask: PlainDword,
    /// The base light colour of the view or car. Unused: each view has its own in
    /// `scene::object_light::ObjectLight::car_light`.
    pub view_light_base: PlainDword,
    /// The 8 bytes of padding before `sse_fill`; no code references them.
    pub unknown_94: [PlainDword; 2],
    /// 16 bytes the point records are cleared with by SSE (`race::frame_draw`); 8 bytes of
    /// padding before it are unused.
    pub sse_fill: [PlainDword; 4],
    /// The fog's step; never read, no longer written (`scene::fog`).
    pub fog_step: PlainDword,
    /// The kind of light being set (`scene::set_object_light`, `scene::vertex_light`,
    /// `race::car_draw`, `scene::draw_described_object`, `scene::draw_menu_object`,
    /// `race::frame_draw`). Unused: each view has its own in `scene::object_light`.
    pub light_kind: PlainDword,
    /// Set along with the light kind, 0 for a car (`race::car_draw`); no reader shows what it
    /// selects. Unused: each view has its own in `scene::object_light`.
    pub light_kind_extra: PlainDword,
    /// The car's light colour (`physics::car_lighting`).
    pub car_light_colour: PlainDword,
    /// First lamp table's colour (`physics::car_lighting`).
    pub lamp_a_colour: PlainDword,
    /// Second lamp table's colour (`physics::car_lighting`).
    pub lamp_b_colour: PlainDword,
    /// The ground's texture s kept (`physics::car_lighting`).
    pub ground_texture_s: PlainDword,
    /// The ground's texture t kept (`physics::car_lighting`).
    pub ground_texture_t: PlainDword,
    /// The plain light tables' colours, two colours (`scene::fill_light_tables`).
    pub plain_light_colours: [PlainDword; 8],
    /// The material phases' four bases, four random parts and a 0 (200, 400, 200, 400, 400,
    /// 400, 400, 400, 0); nothing reads them and nothing writes them (0 from the start).
    pub phase_lengths: [PlainDword; 9],
    /// The clock when `scene::set_object_material` was last here (read by nothing, no longer
    /// written).
    pub phase_clock: PlainDword,
    /// Faces in the list (`scene::list_object_face`, `scene::draw_object_faces`,
    /// `scene::sort_listed_faces`).
    pub listed_face_count: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightWork {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            unknown_4: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x10000),
                PlainDword::new(2),
                PlainDword::new(0x30002),
                PlainDword::new(0x30000),
                PlainDword::new(4),
                PlainDword::new(0x50004),
                PlainDword::new(0x50000),
                PlainDword::new(6),
                PlainDword::new(0x70006),
                PlainDword::new(0x70000),
                PlainDword::new(8),
            ],
            sse_fill: [
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
            ],
            plain_light_colours: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xFF),
                PlainDword::new(0xFF),
                PlainDword::new(0xFF),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            ..ZeroCells::ZERO
        }
    }
}
