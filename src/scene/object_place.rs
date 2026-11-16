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

use crate::memory::session::{PlainDword, PlainFloat};

/// The kept reflections' room: four caches of 1024 by point number.
pub const KEPT_REFLECTIONS: usize = 4096;

#[derive(rc2k_derive::Cells)]
pub struct ObjectPlace {
    /// `ObjectWork.object_position`: the object's x, y, depth.
    pub position: [PlainDword; 3],
    /// `ObjectWork.face_depth`.
    pub face_depth: PlainDword,
    /// `ObjectWork.car_switches`.
    pub car_switches: PlainDword,
    /// `ObjectWork.turned_place`: the place turned.
    pub turned_place: [PlainDword; 3],
    /// `ObjectWork.place`: the matrix's place, x y z.
    pub place: [PlainDword; 3],
    /// `ObjectWork.size`: the object's size (16 = natural).
    pub size: PlainDword,
    /// `ObjectWork.light_strength`: the light's strength.
    pub light_strength: PlainDword,
    /// `ObjectWork.angles`: the object's x, y and z turns.
    pub angles: [PlainDword; 3],
    /// `ObjectWork.object_place`: the scene point of the object
    /// being drawn, -1 none, each view's own.
    pub object_place: PlainDword,
    /// `ObjectWork.only_object`: the described object's mode bits, each view's own.
    pub only_object: PlainDword,
    /// `ObjectWork.object_first_face`.
    pub first_face: PlainDword,
    /// `ObjectWork.object_face_count`.
    pub face_count: PlainDword,
    /// `GroundWork.normal_work`: `face_normal`'s work: three points and six
    /// differences.
    pub normal_work: [PlainDword; 15],
    /// `GroundWork.whole_polygon_count`: polygons drawn whole (read by none).
    pub whole_polygon_count: PlainDword,
    /// `GroundWork.split_polygon_count`: polygons drawn split (read by none).
    pub split_polygon_count: PlainDword,
    /// `GroundWork.object_fading`: the object fades.
    pub fading: PlainDword,
    /// `GroundWork.object_see_through`.
    pub see_through: PlainDword,
    /// `GroundWork.object_fade`: the fade value (0x100 = none), twice.
    pub fade: [PlainDword; 2],
    /// `GroundWork.reflection_normal_number`: the normal's number asked for.
    pub reflection_normal_number: PlainDword,
    /// `GroundWork.reflection_point_number`: the point's number asked for.
    pub reflection_point_number: PlainDword,
    /// `GroundWork.reflection_coordinates`: the two coordinates worked out.
    pub reflection_coordinates: [PlainDword; 2],
    /// `GroundWork.kept_reflections`: four caches of 1024 by point number: the
    /// normals (cleared to -999 by `race::start_kept_reflections`), the points, u and v.
    pub kept_reflections: [PlainDword; KEPT_REFLECTIONS],
    /// `LightWork.drawing_object`: 1 while an object is drawn
    /// (`scene::draw_shown_object`), read by `scene::log_bad_polygon`.
    pub drawing_object: PlainDword,
    /// `RaceVars.move_by`: a moving object's points' step to their other places
    /// (bytes; 0 when it does not move), set by `scene::set_object_material`.
    pub move_by: PlainDword,
    /// `RaceVars.move_fraction`: how far the moving object has gone (a float).
    pub move_fraction: PlainFloat,
    /// `GroundWork.object_texture_set`: added to the
    /// object's face texture numbers (`scene::draw_scene_object`, `scene::draw_object_faces`).
    pub texture_set: PlainDword,
    /// `ObjectWork.view_cleared`: 0 at each race view's scene's start
    /// (`race::draw_view_scene`), the first read by its cars' drawing (`clears_depth`).
    pub view_cleared: [PlainDword; 2],
    /// `ObjectWork.described_cleared`: a described object's three extra turns
    /// (`scene::draw_described_object`; 0 unless its description has them), read as its
    /// matrix is made (`race::frame_draw`).
    pub described_turns: [PlainDword; 3],
}

impl ObjectPlace {
    pub const fn new() -> Self {
        Self {
            position: [const { PlainDword::new(0) }; 3],
            face_depth: PlainDword::new(0),
            car_switches: PlainDword::new(0),
            turned_place: [const { PlainDword::new(0) }; 3],
            place: [const { PlainDword::new(0) }; 3],
            size: PlainDword::new(0),
            light_strength: PlainDword::new(0),
            angles: [const { PlainDword::new(0) }; 3],
            object_place: PlainDword::new(u32::MAX),
            only_object: PlainDword::new(0),
            first_face: PlainDword::new(0),
            face_count: PlainDword::new(0),
            normal_work: [const { PlainDword::new(0) }; 15],
            whole_polygon_count: PlainDword::new(0),
            split_polygon_count: PlainDword::new(0),
            fading: PlainDword::new(0),
            see_through: PlainDword::new(0),
            fade: [const { PlainDword::new(0) }; 2],
            reflection_normal_number: PlainDword::new(0),
            reflection_point_number: PlainDword::new(0),
            reflection_coordinates: [const { PlainDword::new(0) }; 2],
            kept_reflections: [const { PlainDword::new(0) }; KEPT_REFLECTIONS],
            drawing_object: PlainDword::new(0),
            move_by: PlainDword::new(0),
            move_fraction: PlainFloat::new(0.0),
            texture_set: PlainDword::new(0),
            view_cleared: [const { PlainDword::new(0) }; 2],
            described_turns: [const { PlainDword::new(0) }; 3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slots begin where the stores did: the image's first values.
    #[test]
    fn begins_as_the_stores_did() {
        let o = ObjectPlace::new();
        let work = crate::scene::object_store::ObjectWork::new();
        let ground = crate::physics::ground_store::GroundWork::new();
        fn bits<D: crate::memory::session::DwordLike>(c: &[D]) -> Vec<u32> {
            c.iter().map(D::get).collect()
        }
        assert_eq!(bits(&o.position), bits(&work.object_position));
        assert_eq!(o.face_depth.get(), work.face_depth.get());
        assert_eq!(o.car_switches.get(), work.car_switches.get());
        assert_eq!(bits(&o.turned_place), bits(&work.turned_place));
        assert_eq!(bits(&o.turned_place), bits(&work.turned_place));
        assert_eq!(o.first_face.get(), work.object_first_face.get());
        assert_eq!(o.face_count.get(), work.object_face_count.get());
        assert_eq!(bits(&o.normal_work), bits(&ground.normal_work));
        assert_eq!(o.fading.get(), ground.object_fading.get());
        assert_eq!(o.see_through.get(), ground.object_see_through.get());
        assert_eq!(bits(&o.fade), bits(&ground.object_fade));
        assert_eq!(bits(&o.kept_reflections), bits(&ground.kept_reflections));
        assert_eq!(
            o.reflection_normal_number.get(),
            ground.reflection_normal_number.get()
        );
        assert_eq!(
            bits(&o.reflection_coordinates),
            bits(&ground.reflection_coordinates)
        );
    }
}
