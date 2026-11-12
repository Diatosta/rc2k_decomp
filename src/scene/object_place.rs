//! The fading, listing and reflecting of the object being drawn (frame plan F4): the object's
//! position and turned place, its faces' first and count, the fade, the see-through value, the
//! reflection being worked out and the reflections kept, one in each view's slot
//! (`race::view_scratch`, `ctx.view.object`) and one for the front end.
//!
//! The original kept them in two stores for every view at once: the object work
//! (`ObjectWork`, 0x634FA0) and the ground work (`GroundWork`, 0x63967C). Their fields stay
//! there (the stores keep the 1999 layout); the ones listed under each field's name below
//! are now these.
//!
//! Left in `ObjectWork` because a drawing reads them before the same view has written them:
//! `object_place` and `only_object` (the first shown object of a view reads what the last
//! described object left).
//!
//! `place`, `size` and `light_strength` are here too since F6: every reading of a view's
//! drawing finds them written by the same view's drawing first (the sky, the slot objects,
//! the flares, the described objects and the cars each set them before the matrix and the
//! light read them), and the front end's menu draws all go through the front end's slot
//! (`view_scratch::front_end()`), so what its set-ups leave (`set_up_carousel`,
//! `prepare_menu_scene`, `set_up_lit_engine`, the main menu's loop) is what its shown car
//! and highlighted object read. The physics' contact tests keep `ObjectWork.size` as their
//! own (`face_contact` writes it before `object_turn_matrix` reads it); `ObjectWork.place`
//! and `ObjectWork.light_strength` are read by none now. `angles` too (F6 step 5): measured
//! with a copy beside the store's (three held races, nine menu flows), no reading of the
//! drawing found other than the same view's last write; `ObjectWork.angles` stays the
//! physics' own (`physics::face_contact`, `race::get_mat_crash`, `physics::object_turn_matrix`).

use crate::session::{Dword, Float};

/// The kept reflections' room: four caches of 1024 by point number.
pub const KEPT_REFLECTIONS: usize = 4096;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ObjectPlace {
    /// `ObjectWork.object_position` (0x635004): the object's x, y, depth.
    pub position: [Dword; 3],
    /// `ObjectWork.face_depth` (0x635010).
    pub face_depth: Dword,
    /// `ObjectWork.car_switches` (0x635028).
    pub car_switches: Dword,
    /// `ObjectWork.turned_place` (0x63502C): the place turned.
    pub turned_place: [Dword; 3],
    /// `ObjectWork.place` (0x63501C): the matrix's place, x y z.
    pub place: [Dword; 3],
    /// `ObjectWork.size` (0x635038): the object's size (16 = natural).
    pub size: Dword,
    /// `ObjectWork.light_strength` (`g_lightStrength`, 0x635090).
    pub light_strength: Dword,
    /// `ObjectWork.angles` (0x63503C): the object's x, y and z turns.
    pub angles: [Dword; 3],
    /// `ObjectWork.object_place` (`g_objectPlace`, 0x634FFC): the scene point of the object
    /// being drawn, -1 none, each view's own.
    pub object_place: Dword,
    /// `ObjectWork.only_object` (0x635058): the described object's mode bits, each view's own.
    pub only_object: Dword,
    /// `ObjectWork.object_first_face` (`g_objectFirstFace`, 0x6350BC).
    pub first_face: Dword,
    /// `ObjectWork.object_face_count` (`g_objectFaceCount`, 0x6350C0).
    pub face_count: Dword,
    /// `GroundWork.normal_work` (0x6396A0): `face_normal`'s work: three points and six
    /// differences.
    pub normal_work: [Dword; 15],
    /// `GroundWork.whole_polygon_count` (0x639960): polygons drawn whole (read by none).
    pub whole_polygon_count: Dword,
    /// `GroundWork.split_polygon_count` (0x639964): polygons drawn split (read by none).
    pub split_polygon_count: Dword,
    /// `GroundWork.object_fading` (`g_objectFading`, 0x63996C): the object fades.
    pub fading: Dword,
    /// `GroundWork.object_see_through` (`g_objectSeeThrough`, 0x639970).
    pub see_through: Dword,
    /// `GroundWork.object_fade` (0x639974): the fade value (0x100 = none), twice.
    pub fade: [Dword; 2],
    /// `GroundWork.reflection_normal_number` (0x6399A8): the normal's number asked for.
    pub reflection_normal_number: Dword,
    /// `GroundWork.reflection_point_number` (0x6399AC): the point's number asked for.
    pub reflection_point_number: Dword,
    /// `GroundWork.reflection_coordinates` (0x6399B0): the two coordinates worked out.
    pub reflection_coordinates: [Dword; 2],
    /// `GroundWork.kept_reflections` (0x6399B8): four caches of 1024 by point number: the
    /// normals (cleared to -999 by `race::start_kept_reflections`), the points, u and v.
    pub kept_reflections: [Dword; KEPT_REFLECTIONS],
    /// `LightWork.drawing_object` (`g_drawingObject`, 0x62DA44): 1 while an object is drawn
    /// (`scene::draw_shown_object`), read by `scene::log_bad_polygon`.
    pub drawing_object: Dword,
    /// `RaceVars.move_by` (0x623DD0): a moving object's points' step to their other places
    /// (bytes; 0 when it does not move), set by `scene::set_object_material`.
    pub move_by: Dword,
    /// `RaceVars.move_fraction` (0x623DD4): how far the moving object has gone (a float).
    pub move_fraction: Float,
    /// `GroundWork.object_texture_set` (`g_objectTextureSet`, 0x639954): added to the
    /// object's face texture numbers (`scene::draw_scene_object`, `scene::draw_object_faces`).
    pub texture_set: Dword,
    /// `ObjectWork.view_cleared` (0x635014, 0x635018): 0 at each race view's scene's start
    /// (`race::draw_view_scene`), the first read by its cars' drawing (`clears_depth`).
    pub view_cleared: [Dword; 2],
    /// `ObjectWork.described_cleared` (0x63505C ..): a described object's three extra turns
    /// (`scene::draw_described_object`; 0 unless its description has them), read as its
    /// matrix is made (`race::frame_draw`).
    pub described_turns: [Dword; 3],
}

impl ObjectPlace {
    pub const fn new() -> Self {
        Self {
            position: [const { Dword::new(0) }; 3],
            face_depth: Dword::new(0),
            car_switches: Dword::new(0),
            turned_place: [const { Dword::new(0) }; 3],
            place: [const { Dword::new(0) }; 3],
            size: Dword::new(0),
            light_strength: Dword::new(0),
            angles: [const { Dword::new(0) }; 3],
            object_place: Dword::new(u32::MAX),
            only_object: Dword::new(0),
            first_face: Dword::new(0),
            face_count: Dword::new(0),
            normal_work: [const { Dword::new(0) }; 15],
            whole_polygon_count: Dword::new(0),
            split_polygon_count: Dword::new(0),
            fading: Dword::new(0),
            see_through: Dword::new(0),
            fade: [const { Dword::new(0) }; 2],
            reflection_normal_number: Dword::new(0),
            reflection_point_number: Dword::new(0),
            reflection_coordinates: [const { Dword::new(0) }; 2],
            kept_reflections: [const { Dword::new(0) }; KEPT_REFLECTIONS],
            drawing_object: Dword::new(0),
            move_by: Dword::new(0),
            move_fraction: Float::new(0.0),
            texture_set: Dword::new(0),
            view_cleared: [const { Dword::new(0) }; 2],
            described_turns: [const { Dword::new(0) }; 3],
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
        fn bits<D: crate::session::DwordLike>(c: &[D]) -> Vec<u32> {
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
