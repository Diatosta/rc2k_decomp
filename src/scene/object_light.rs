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

use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// A view's object lighting. The floats are held as their bits.
pub type ObjectLight = PlainSlot<ObjectLightValues>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ObjectLightValues {
    /// The lamp's tint taken from its entry (`LightWork.lamp_tint`).
    pub lamp_tint: u32,
    /// The point's place (`LightWork.point_place[0]` ..).
    pub point_place: [u32; 3],
    /// The point of `scene::light_dot`'s inputs (`LightWork.light_dot_point[0]` ..).
    pub light_dot_point: [u32; 3],
    /// The light's vector in `scene::light_dot`'s inputs (`LightWork.light_dot_light[0]` ..).
    pub light_dot_light: [u32; 3],
    /// The factor in `scene::light_dot`'s inputs (`LightWork.light_dot_factor`).
    pub light_dot_factor: u32,
    /// The light's colour.
    pub light_colour: u32,
    /// The base light of the car being drawn (`LightWork.view_light_base`):
    /// the view's car's light colour at the view's start (`race::draw_view_scene`), each car's
    /// as it is drawn (`race::draw_car_entry`, `race::draw_own_car`); read by the light kind 3
    /// (`scene::vertex_light`).
    pub car_light: u32,
    /// The view's light colour (`view_light`): the view's
    /// car's light colour plus the light (`race::view_light`), each view's own; its effects
    /// (snow, windscreen flakes, the plane) are given it.
    pub view_light: u32,
    /// The other light of the object being drawn (`other_light`):
    /// 0 from the sky on, a described object's or lamp's or flare's own; added to the light
    /// (`scene::set_object_light`) or the faces' light in the material way 2.
    pub other_light: u32,
    /// The light's direction, set with the
    /// object's turned matrix (`scene::matrix::set_light_direction`).
    pub light_direction: [u32; 3],
    /// The kind of light being set.
    pub light_kind: u32,
    /// Set with the light kind, 0 for a car (`LightWork.light_kind_extra`).
    pub light_kind_extra: u32,
    /// The place (x, y, z) being lit, a car or a point.
    pub lit_place: [u32; 3],
    /// The turned light direction seen by the car lights (`LightState.turned_light[0]` ..).
    pub turned_light: [u32; 3],
    /// The view kind (`LightState.view_kind`): 4 for the sky, 1 and 2 for objects, 5 for the
    /// menus'.
    pub view_kind: u32,
    /// The face's bits (`LightState.face_bits`).
    pub face_bits: u32,
    /// The light added to each vertex of the object (`ObjectWork.light_added`).
    pub light_added: u32,
    /// The lit colour's lowest place.
    pub light_lowest: u32,
    /// The lit colour's highest place.
    pub light_highest: u32,
    /// The lit colour's level.
    pub light_level: u32,
    /// The light direction's two turns (`ObjectWork.light_turns[0]`, `ObjectWork.light_turns[1]`).
    pub light_turns: [u32; 2],
    /// `GroundWork.drawn_car_light`: the car whose object is being lit, or -1
    /// (`scene::set_object_light`); `scene::add_car_lights` skips its lights.
    pub drawn_car_light: u32,
}

impl ObjectLightValues {
    pub const fn new() -> Self {
        Self {
            lamp_tint: 0,
            point_place: [0; 3],
            light_dot_point: [0; 3],
            light_dot_light: [0; 3],
            light_dot_factor: 0,
            light_colour: 0,
            car_light: 0,
            view_light: 0,
            other_light: 0,
            light_direction: [0; 3],
            light_kind: 0,
            light_kind_extra: 0,
            lit_place: [0; 3],
            turned_light: [0; 3],
            view_kind: 0,
            face_bits: 0,
            light_added: 0,
            light_lowest: 0,
            light_highest: 0,
            light_level: 0,
            light_turns: [0; 2],
            drawn_car_light: u32::MAX,
        }
    }
}

impl Merge for ObjectLightValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_value(&mut self.lamp_tint, &now.lamp_tint, &start.lamp_tint);
        merge_run(&mut self.point_place, &now.point_place, &start.point_place);
        merge_run(
            &mut self.light_dot_point,
            &now.light_dot_point,
            &start.light_dot_point,
        );
        merge_run(
            &mut self.light_dot_light,
            &now.light_dot_light,
            &start.light_dot_light,
        );
        merge_value(
            &mut self.light_dot_factor,
            &now.light_dot_factor,
            &start.light_dot_factor,
        );
        merge_value(
            &mut self.light_colour,
            &now.light_colour,
            &start.light_colour,
        );
        merge_value(&mut self.car_light, &now.car_light, &start.car_light);
        merge_value(&mut self.view_light, &now.view_light, &start.view_light);
        merge_value(&mut self.other_light, &now.other_light, &start.other_light);
        merge_run(
            &mut self.light_direction,
            &now.light_direction,
            &start.light_direction,
        );
        merge_value(&mut self.light_kind, &now.light_kind, &start.light_kind);
        merge_value(
            &mut self.light_kind_extra,
            &now.light_kind_extra,
            &start.light_kind_extra,
        );
        merge_run(&mut self.lit_place, &now.lit_place, &start.lit_place);
        merge_run(
            &mut self.turned_light,
            &now.turned_light,
            &start.turned_light,
        );
        merge_value(&mut self.view_kind, &now.view_kind, &start.view_kind);
        merge_value(&mut self.face_bits, &now.face_bits, &start.face_bits);
        merge_value(&mut self.light_added, &now.light_added, &start.light_added);
        merge_value(
            &mut self.light_lowest,
            &now.light_lowest,
            &start.light_lowest,
        );
        merge_value(
            &mut self.light_highest,
            &now.light_highest,
            &start.light_highest,
        );
        merge_value(&mut self.light_level, &now.light_level, &start.light_level);
        merge_run(&mut self.light_turns, &now.light_turns, &start.light_turns);
        merge_value(
            &mut self.drawn_car_light,
            &now.drawn_car_light,
            &start.drawn_car_light,
        );
    }
}
