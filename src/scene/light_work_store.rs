//! The scene's light and object-drawing work (IDA 0x62DA44 .. 0x62DB5C): the object
//! being drawn, the lamp tint and the point's place, the light-dot inputs, the light's colour and
//! kind, the lamp and ground colours the car lighting keeps, the plain light colours, the
//! material phases' lengths and clock, and the count of listed faces. Names are readings
//! unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LightWork {
    /// (0x62DA44): `g_drawingObject`: the object being drawn; now each view's (`scene::object_place`); unused.
    pub drawing_object: PlainDword,
    /// (0x62DA48): 13 dwords after the drawing object; no code references them (a gap).
    pub unknown_4: [PlainDword; 13],
    /// (0x62DA7C): `dword_62DA7C`: the lamp's tint taken from its entry (`scene::add_point_lights`, `race::frame_draw`); now each view's (`scene::object_light`); unused.
    pub lamp_tint: PlainDword,
    /// (0x62DA80): three dwords before the point's place; the original's only references are from undefined (non-function) code, the port has none (a gap).
    pub unknown_3c: [PlainDword; 3],
    /// (0x62DA8C): `dword_62DA8C` ..: the point's place (`scene::draw_object_face`; read by `fsub`); now each view's (`scene::object_light`); unused.
    pub point_place: [PlainFloat; 3],
    /// (0x62DA98): `flt_62DA98` .. `flt_62DAA4`: float work of the lighting (a reading; written and read in the object drawing code only).
    pub light_work_floats: [PlainFloat; 4],
    /// (0x62DAA8): `dword_62DAA8` ..: the point of `scene::light_dot`'s inputs (a reading: floats, loaded by `fld`); now each view's (`scene::object_light`); unused.
    pub light_dot_point: [PlainFloat; 3],
    /// (0x62DAB4): `dword_62DAB4` ..: the light's direction vector, the second input of `scene::light_dot`; now each view's (`scene::object_light`); unused.
    pub light_dot_light: [PlainFloat; 3],
    /// (0x62DAC0): `flt_62DAC0`: the factor in `scene::light_dot`'s inputs; now each view's (`scene::object_light`); unused.
    pub light_dot_factor: PlainFloat,
    /// (0x62DAC4): `g_lightColour`: the light's colour (`scene::vertex_light`, `scene::add_point_lights`, `race::course_faces`, `race::frame_draw`); now each view's (`scene::object_light`); unused.
    pub light_colour: PlainDword,
    /// (0x62DAC8): one dword between the light's colour and the lamps' mask; no code references it (a gap).
    pub unknown_84: PlainDword,
    /// (0x62DACC): `dword_62DACC`: the lamp parts' mask (`race::car_parts`, `race::draw_race_scene`, `race::loadlevel`).
    pub lamps_mask: PlainDword,
    /// (0x62DAD0): `dword_62DAD0`: the lone-group switch / marks mask (`race::car_parts`, `physics::drop_broken_parts`, `scene::list_object_face`, `race::draw_race_scene`, `race::loadlevel`).
    pub marks_mask: PlainDword,
    /// (0x62DAD4): `dword_62DAD4`: the base light colour of the view or car; now each view's
    /// (`scene::object_light::ObjectLight::car_light`), this one not used.
    pub view_light_base: PlainDword,
    /// (0x62DAD8): the 8 bytes of padding before `sse_fill`; no code references them.
    pub unknown_94: [PlainDword; 2],
    /// (0x62DAE0): `xmmword_62DAE0`: 16 bytes the point records are cleared with by SSE (`race::frame_draw`); 8 bytes of padding before it are unused.
    pub sse_fill: [PlainDword; 4],
    /// (0x62DAF0): `dword_62DAF0`: the fog's step; never read, no longer written (`scene::fog`).
    pub fog_step: PlainDword,
    /// (0x62DAF4): `g_lightKind`: the kind of light being set (`scene::set_object_light`, `scene::vertex_light`, `race::car_draw`, `scene::draw_described_object`, `scene::draw_menu_object`, `race::frame_draw`); now each view's (`scene::object_light`); unused.
    pub light_kind: PlainDword,
    /// (0x62DAF8): `dword_62DAF8`: set along with the light kind, 0 for a car (`race::car_draw`); no reader shows what it selects; now each view's (`scene::object_light`); unused.
    pub light_kind_extra: PlainDword,
    /// (0x62DAFC): `dword_62DAFC`: the car's light colour (`physics::car_lighting`).
    pub car_light_colour: PlainDword,
    /// (0x62DB00): `dword_62DB00`: first lamp table's colour (`physics::car_lighting`).
    pub lamp_a_colour: PlainDword,
    /// (0x62DB04): `dword_62DB04`: second lamp table's colour (`physics::car_lighting`).
    pub lamp_b_colour: PlainDword,
    /// (0x62DB08): `dword_62DB08`: the ground's texture s kept (`physics::car_lighting`).
    pub ground_texture_s: PlainDword,
    /// (0x62DB0C): `dword_62DB0C`: the ground's texture t kept (`physics::car_lighting`).
    pub ground_texture_t: PlainDword,
    /// (0x62DB10): `dword_62DB10` (int[8]): the plain light tables' colours, two colours (`scene::fill_light_tables`).
    pub plain_light_colours: [PlainDword; 8],
    /// (0x62DB30): `dword_62DB30` (nine dwords to 0x62DB54): the material phases' four bases, four random parts and a 0 (200, 400, 200, 400, 400, 400, 400, 400, 0), which the original's `scene::set_object_material` stored on each call; nothing reads them and nothing writes them (0 from the image).
    pub phase_lengths: [PlainDword; 9],
    /// (0x62DB54): `dword_62DB54`: the clock when `scene::set_object_material` was last here (read by nothing, no longer written).
    pub phase_clock: PlainDword,
    /// (0x62DB58): `g_listedFaceCount`: faces in the list (`scene::list_object_face`, `scene::draw_object_faces`, `scene::sort_listed_faces`).
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
