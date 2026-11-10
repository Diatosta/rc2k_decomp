//! A view's object lighting (frame plan F4): what the lighting of the objects and faces drawn
//! in the view works out and reads again, one in each view's slot (`race::view_scratch`) and
//! one for the front end.
//!
//! The original kept it in three shared pieces for every view at once: the light work
//! (`scene::light_work_store`: the lamp's tint, the point's place, the inputs of
//! `scene::light_dot`, the light's colour, the kind and its extra), the light state
//! (`scene::light_store`: the lit place, the turned car light, the view kind, the face's bits)
//! and the object work (`scene::object_store`: the light added to the object, the lowest,
//! highest and level of the lit colour, the sun's turns). Each is written before it is read
//! within a view: the sky writes the view kind, the lowest, highest, level, turns and the kind
//! at the start of every race view, and the lit place, the colour and the added light are set
//! by whoever gathers the lamps before they are used.
//!
//! Here too since F6: the car's base light (`view_light_base`, as `car_light`), the other
//! light (`OBJECT_WORK.other_light`) and the light's direction (`g_lightDirection`): each
//! measured with the shared one kept beside, no reading saw another value (three held races,
//! nine menu flows). The sky sets the other light 0 and, by placing its object, the
//! direction, at every view's start.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ObjectLight {
    /// The lamp's tint taken from its entry (`dword_62DA7C`).
    pub lamp_tint: Dword,
    /// The point's place (`dword_62DA8C` ..).
    pub point_place: [Float; 3],
    /// The point of `scene::light_dot`'s inputs (`dword_62DAA8` ..).
    pub light_dot_point: [Float; 3],
    /// The light's vector in `scene::light_dot`'s inputs (`dword_62DAB4` ..).
    pub light_dot_light: [Float; 3],
    /// The factor in `scene::light_dot`'s inputs (`flt_62DAC0`).
    pub light_dot_factor: Float,
    /// The light's colour (`g_lightColour`).
    pub light_colour: Dword,
    /// The base light of the car being drawn (`LIGHT_WORK.view_light_base`, `dword_62DAD4`):
    /// the view's car's light colour at the view's start (`race::draw_view_scene`), each car's
    /// as it is drawn (`race::draw_car_entry`, `race::draw_own_car`); read by the light kind 3
    /// (`scene::vertex_light`).
    pub car_light: Dword,
    /// The view's light colour (`FRAME_WORK.view_light`, `g_viewLight`, 0x6415E4): the view's
    /// car's light colour plus the light (`race::view_light`), each view's own; the effects
    /// after the views read the shared one.
    pub view_light: Dword,
    /// The other light of the object being drawn (`OBJECT_WORK.other_light`, 0x635054):
    /// 0 from the sky on, a described object's or lamp's or flare's own; added to the light
    /// (`scene::set_object_light`) or the faces' light in the material way 2.
    pub other_light: Dword,
    /// The light's direction (`g_lightDirection`, 0x6240A8 ..), set with the
    /// object's turned matrix (`scene::matrix::set_light_direction`).
    pub light_direction: [Float; 3],
    /// The kind of light being set (`g_lightKind`).
    pub light_kind: Dword,
    /// Set with the light kind, 0 for a car (`dword_62DAF8`).
    pub light_kind_extra: Dword,
    /// The place (x, y, z) being lit, a car or a point (`g_litPlace`).
    pub lit_place: [Dword; 3],
    /// The turned light direction seen by the car lights (`dword_6365B8` ..).
    pub turned_light: [Dword; 3],
    /// The view kind (`dword_6365C8`): 4 for the sky, 1 and 2 for objects, 5 for the menus'.
    pub view_kind: Dword,
    /// The face's bits (`dword_636608`).
    pub face_bits: Dword,
    /// The light added to each vertex of the object (`dword_635068`).
    pub light_added: Dword,
    /// The lit colour's lowest place (`g_lightLowest`).
    pub light_lowest: Dword,
    /// The lit colour's highest place (`g_lightHighest`).
    pub light_highest: Dword,
    /// The lit colour's level (`g_lightLevel`).
    pub light_level: Dword,
    /// The light direction's two turns (`dword_635094`, `dword_635098`).
    pub light_turns: [Dword; 2],
}

impl ObjectLight {
    pub const fn new() -> Self {
        Self {
            lamp_tint: Dword::new(0),
            point_place: [const { Float::from_bits(0) }; 3],
            light_dot_point: [const { Float::from_bits(0) }; 3],
            light_dot_light: [const { Float::from_bits(0) }; 3],
            light_dot_factor: Float::from_bits(0),
            light_colour: Dword::new(0),
            car_light: Dword::new(0),
            view_light: Dword::new(0),
            other_light: Dword::new(0),
            light_direction: [const { Float::from_bits(0) }; 3],
            light_kind: Dword::new(0),
            light_kind_extra: Dword::new(0),
            lit_place: [const { Dword::new(0) }; 3],
            turned_light: [const { Dword::new(0) }; 3],
            view_kind: Dword::new(0),
            face_bits: Dword::new(0),
            light_added: Dword::new(0),
            light_lowest: Dword::new(0),
            light_highest: Dword::new(0),
            light_level: Dword::new(0),
            light_turns: [const { Dword::new(0) }; 2],
        }
    }
}
