//! A view's object lighting: what the lighting of the objects and faces drawn in the view works
//! out and reads again, one in each view's slot (`race::view_scratch`) and one for the front
//! end.
//!
//! It stands for three pieces: the light work (`scene::light_work_store`: the lamp's tint, the
//! point's place, the inputs of `scene::light_dot`, the light's colour, the kind and its
//! extra), the light state (`scene::light_store`: the lit place, the turned car light, the view
//! kind, the face's bits) and the object work (`scene::object_store`: the light added to the
//! object, the lowest, highest and level of the lit colour, the sun's turns). Each is written
//! before it is read within a view: the sky writes the view kind, the lowest, highest, level,
//! turns and the kind at the start of every race view, and the lit place, the colour and the
//! added light are set by whoever gathers the lamps before they are used.
//!
//! The car's base light (`view_light_base`, as `car_light`), the other light
//! (`other_light`) and the light's direction are each view's own too. The sky sets
//! the other light 0 and, by placing its object, the direction, at every view's start.

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells)]
pub struct ObjectLight {
    /// The lamp's tint taken from its entry (`LightWork.lamp_tint`).
    pub lamp_tint: PlainDword,
    /// The point's place (`LightWork.point_place[0]` ..).
    pub point_place: [PlainFloat; 3],
    /// The point of `scene::light_dot`'s inputs (`LightWork.light_dot_point[0]` ..).
    pub light_dot_point: [PlainFloat; 3],
    /// The light's vector in `scene::light_dot`'s inputs (`LightWork.light_dot_light[0]` ..).
    pub light_dot_light: [PlainFloat; 3],
    /// The factor in `scene::light_dot`'s inputs (`LightWork.light_dot_factor`).
    pub light_dot_factor: PlainFloat,
    /// The light's colour.
    pub light_colour: PlainDword,
    /// The base light of the car being drawn (`LightWork.view_light_base`):
    /// the view's car's light colour at the view's start (`race::draw_view_scene`), each car's
    /// as it is drawn (`race::draw_car_entry`, `race::draw_own_car`); read by the light kind 3
    /// (`scene::vertex_light`).
    pub car_light: PlainDword,
    /// The view's light colour (`view_light`): the view's
    /// car's light colour plus the light (`race::view_light`), each view's own; its effects
    /// (snow, windscreen flakes, the plane) are given it.
    pub view_light: PlainDword,
    /// The other light of the object being drawn (`other_light`):
    /// 0 from the sky on, a described object's or lamp's or flare's own; added to the light
    /// (`scene::set_object_light`) or the faces' light in the material way 2.
    pub other_light: PlainDword,
    /// The light's direction, set with the
    /// object's turned matrix (`scene::matrix::set_light_direction`).
    pub light_direction: [PlainFloat; 3],
    /// The kind of light being set.
    pub light_kind: PlainDword,
    /// Set with the light kind, 0 for a car (`LightWork.light_kind_extra`).
    pub light_kind_extra: PlainDword,
    /// The place (x, y, z) being lit, a car or a point.
    pub lit_place: [PlainDword; 3],
    /// The turned light direction seen by the car lights (`LightState.turned_light[0]` ..).
    pub turned_light: [PlainDword; 3],
    /// The view kind (`LightState.view_kind`): 4 for the sky, 1 and 2 for objects, 5 for the
    /// menus'.
    pub view_kind: PlainDword,
    /// The face's bits (`LightState.face_bits`).
    pub face_bits: PlainDword,
    /// The light added to each vertex of the object (`ObjectWork.light_added`).
    pub light_added: PlainDword,
    /// The lit colour's lowest place.
    pub light_lowest: PlainDword,
    /// The lit colour's highest place.
    pub light_highest: PlainDword,
    /// The lit colour's level.
    pub light_level: PlainDword,
    /// The light direction's two turns (`ObjectWork.light_turns[0]`, `ObjectWork.light_turns[1]`).
    pub light_turns: [PlainDword; 2],
    /// `GroundWork.drawn_car_light`: the car whose object is being lit, or -1
    /// (`scene::set_object_light`); `scene::add_car_lights` skips its lights.
    pub drawn_car_light: PlainDword,
}

impl ObjectLight {
    pub const fn new() -> Self {
        Self {
            lamp_tint: PlainDword::new(0),
            point_place: [const { PlainFloat::new(0.0) }; 3],
            light_dot_point: [const { PlainFloat::new(0.0) }; 3],
            light_dot_light: [const { PlainFloat::new(0.0) }; 3],
            light_dot_factor: PlainFloat::new(0.0),
            light_colour: PlainDword::new(0),
            car_light: PlainDword::new(0),
            view_light: PlainDword::new(0),
            other_light: PlainDword::new(0),
            light_direction: [const { PlainFloat::new(0.0) }; 3],
            light_kind: PlainDword::new(0),
            light_kind_extra: PlainDword::new(0),
            lit_place: [const { PlainDword::new(0) }; 3],
            turned_light: [const { PlainDword::new(0) }; 3],
            view_kind: PlainDword::new(0),
            face_bits: PlainDword::new(0),
            light_added: PlainDword::new(0),
            light_lowest: PlainDword::new(0),
            light_highest: PlainDword::new(0),
            light_level: PlainDword::new(0),
            light_turns: [const { PlainDword::new(0) }; 2],
            drawn_car_light: PlainDword::new(u32::MAX),
        }
    }
}
