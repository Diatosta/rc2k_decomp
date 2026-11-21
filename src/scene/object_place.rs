//! The fading, listing and reflecting of the object being drawn: the object's position and turned
//! place, its faces' first and count, the fade, the see-through value, the reflection being worked
//! out and the reflections kept, one in each view's slot (`race::view_scratch`, `ctx.view.object`)
//! and one for the front end.
//!
//! They stand for two stores shared by every view in turn: the object work (`ObjectWork`) and the
//! ground work (`GroundWork`). Their fields stay there (the stores keep their layout); the ones
//! listed under each field's name below are now these.
//!
//! Left in `ObjectWork` because a drawing reads them before the same view has written them:
//! `object_place` and `only_object` (the first shown object of a view reads what the last described
//! object left).
//!
//! `place`, `size` and `light_strength` are here too: every reading of a view's drawing finds them
//! written by the same view's drawing first (the sky, the slot objects, the flares, the described
//! objects and the cars each set them before the matrix and the light read them), and the front
//! end's menu draws all go through the front end's slot (`DrawingScratch::front_end`), so what its
//! set-ups leave (`set_up_carousel`, `prepare_menu_scene`, `set_up_lit_engine`, the main menu's
//! loop) is what its shown car and highlighted object read. The physics' contact tests keep
//! `ObjectWork.size` as their own (`face_contact` writes it before `object_turn_matrix` reads it);
//! `ObjectWork.place` and `ObjectWork.light_strength` are read by none now. `angles` too: no
//! reading of the drawing found other than the same view's last write; `ObjectWork.angles` stays
//! the physics' own (`physics::face_contact`, `race::get_mat_crash`,
//! `physics::object_turn_matrix`).

use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// The kept reflections' room: four caches of 1024 by point number.
pub const KEPT_REFLECTIONS: usize = 4096;

/// A view's object placing.
pub type ObjectPlace = PlainSlot<ObjectPlaceValues>;

/// A view's object placing, plain; the float is held as its bits.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ObjectPlaceValues {
    /// `ObjectWork.object_position`: the object's x, y, depth.
    pub position: [u32; 3],
    /// `ObjectWork.face_depth`.
    pub face_depth: u32,
    /// `ObjectWork.car_switches`.
    pub car_switches: u32,
    /// `ObjectWork.turned_place`: the place turned.
    pub turned_place: [u32; 3],
    /// `ObjectWork.place`: the matrix's place, x y z.
    pub place: [u32; 3],
    /// `ObjectWork.size`: the object's size (16 = natural).
    pub size: u32,
    /// `ObjectWork.light_strength`: the light's strength.
    pub light_strength: u32,
    /// `ObjectWork.angles`: the object's x, y and z turns.
    pub angles: [u32; 3],
    /// `ObjectWork.object_place`: the scene point of the object
    /// being drawn, -1 none, each view's own.
    pub object_place: u32,
    /// `ObjectWork.only_object`: the described object's mode bits, each view's own.
    pub only_object: u32,
    /// `ObjectWork.object_first_face`.
    pub first_face: u32,
    /// `ObjectWork.object_face_count`.
    pub face_count: u32,
    /// `GroundWork.normal_work`: `face_normal`'s work: three points and six
    /// differences.
    pub normal_work: [u32; 15],
    /// `GroundWork.object_fading`: the object fades.
    pub fading: u32,
    /// `GroundWork.object_see_through`.
    pub see_through: u32,
    /// `GroundWork.object_fade`: the fade value (0x100 = none), twice.
    pub fade: [u32; 2],
    /// `GroundWork.reflection_normal_number`: the normal's number asked for.
    pub reflection_normal_number: u32,
    /// `GroundWork.reflection_point_number`: the point's number asked for.
    pub reflection_point_number: u32,
    /// `GroundWork.reflection_coordinates`: the two coordinates worked out.
    pub reflection_coordinates: [u32; 2],
    /// `GroundWork.kept_reflections`: four caches of 1024 by point number: the
    /// normals (cleared to -999 by `race::start_kept_reflections`), the points, u and v.
    pub kept_reflections: [u32; KEPT_REFLECTIONS],
    /// `LightWork.drawing_object`: 1 while an object is drawn
    /// (`scene::draw_shown_object`), read by `scene::log_bad_polygon`.
    pub drawing_object: u32,
    /// `RaceVars.move_by`: a moving object's points' step to their other places
    /// (bytes; 0 when it does not move), set by `scene::set_object_material`.
    pub move_by: u32,
    /// `RaceVars.move_fraction`: how far the moving object has gone (a float).
    pub move_fraction: u32,
    /// `GroundWork.object_texture_set`: added to the
    /// object's face texture numbers (`scene::draw_scene_object`, `scene::draw_object_faces`).
    pub texture_set: u32,
    /// `ObjectWork.view_cleared`: 0 at each race view's scene's start
    /// (`race::draw_view_scene`), the first read by its cars' drawing (`clears_depth`).
    pub view_cleared: [u32; 2],
    /// `ObjectWork.described_cleared`: a described object's three extra turns
    /// (`scene::draw_described_object`; 0 unless its description has them), read as its
    /// matrix is made (`race::frame_draw`).
    pub described_turns: [u32; 3],
}

impl ObjectPlaceValues {
    pub const fn new() -> Self {
        Self {
            position: [0; 3],
            face_depth: 0,
            car_switches: 0,
            turned_place: [0; 3],
            place: [0; 3],
            size: 0,
            light_strength: 0,
            angles: [0; 3],
            object_place: u32::MAX,
            only_object: 0,
            first_face: 0,
            face_count: 0,
            normal_work: [0; 15],
            fading: 0,
            see_through: 0,
            fade: [0; 2],
            reflection_normal_number: 0,
            reflection_point_number: 0,
            reflection_coordinates: [0; 2],
            kept_reflections: [0; KEPT_REFLECTIONS],
            drawing_object: 0,
            move_by: 0,
            move_fraction: 0,
            texture_set: 0,
            view_cleared: [0; 2],
            described_turns: [0; 3],
        }
    }
}

impl Merge for ObjectPlaceValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.position, &now.position, &start.position);
        merge_value(&mut self.face_depth, &now.face_depth, &start.face_depth);
        merge_value(
            &mut self.car_switches,
            &now.car_switches,
            &start.car_switches,
        );
        merge_run(
            &mut self.turned_place,
            &now.turned_place,
            &start.turned_place,
        );
        merge_run(&mut self.place, &now.place, &start.place);
        merge_value(&mut self.size, &now.size, &start.size);
        merge_value(
            &mut self.light_strength,
            &now.light_strength,
            &start.light_strength,
        );
        merge_run(&mut self.angles, &now.angles, &start.angles);
        merge_value(
            &mut self.object_place,
            &now.object_place,
            &start.object_place,
        );
        merge_value(&mut self.only_object, &now.only_object, &start.only_object);
        merge_value(&mut self.first_face, &now.first_face, &start.first_face);
        merge_value(&mut self.face_count, &now.face_count, &start.face_count);
        merge_run(&mut self.normal_work, &now.normal_work, &start.normal_work);
        merge_value(&mut self.fading, &now.fading, &start.fading);
        merge_value(&mut self.see_through, &now.see_through, &start.see_through);
        merge_run(&mut self.fade, &now.fade, &start.fade);
        merge_value(
            &mut self.reflection_normal_number,
            &now.reflection_normal_number,
            &start.reflection_normal_number,
        );
        merge_value(
            &mut self.reflection_point_number,
            &now.reflection_point_number,
            &start.reflection_point_number,
        );
        merge_run(
            &mut self.reflection_coordinates,
            &now.reflection_coordinates,
            &start.reflection_coordinates,
        );
        merge_run(
            &mut self.kept_reflections,
            &now.kept_reflections,
            &start.kept_reflections,
        );
        merge_value(
            &mut self.drawing_object,
            &now.drawing_object,
            &start.drawing_object,
        );
        merge_value(&mut self.move_by, &now.move_by, &start.move_by);
        merge_value(
            &mut self.move_fraction,
            &now.move_fraction,
            &start.move_fraction,
        );
        merge_value(&mut self.texture_set, &now.texture_set, &start.texture_set);
        merge_run(
            &mut self.view_cleared,
            &now.view_cleared,
            &start.view_cleared,
        );
        merge_run(
            &mut self.described_turns,
            &now.described_turns,
            &start.described_turns,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slots begin where the stores did: the image's first values.
    #[test]
    fn begins_as_the_stores_did() {
        let o = ObjectPlaceValues::new();
        let work = crate::scene::object_store::ObjectWork::new();
        let ground = crate::physics::ground_store::GroundWork::new();
        fn bits<D: crate::memory::session::DwordLike>(c: &[D]) -> Vec<u32> {
            c.iter().map(D::get).collect()
        }
        assert_eq!(o.position.to_vec(), work.object_position.to_vec());
        assert_eq!(o.face_depth, 0);
        assert_eq!(o.car_switches, 0);
        assert_eq!(o.turned_place, [0, 0, 0]);
        assert_eq!(o.first_face, 0);
        assert_eq!(o.face_count, 0);
        assert_eq!(o.normal_work.to_vec(), bits(&ground.normal_work));
        assert_eq!(o.fading, ground.object_fading.get());
        assert_eq!(o.see_through, ground.object_see_through.get());
        assert_eq!(o.fade.to_vec(), bits(&ground.object_fade));
        assert_eq!(o.kept_reflections.to_vec(), bits(&ground.kept_reflections));
        assert_eq!(
            o.reflection_normal_number,
            ground.reflection_normal_number.get()
        );
        assert_eq!(
            o.reflection_coordinates.to_vec(),
            bits(&ground.reflection_coordinates)
        );
    }
}
