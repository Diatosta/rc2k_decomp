//! The frame and object work, first part: the light and view switches, the object being drawn
//! (number, material, fade, face flags, group mask), the frame-rate clock, the courses found, and a
//! few race flags. `#[repr(C)]`, laid over the race block bytes.

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameSwitches {
    /// The 3D scene is set up for drawing through the light tables: written 1 only, by
    /// `scene::set_up_front_end_scene` (twice), `race::stage_loading` and, for the length of
    /// `scene::fill_light_tables`, `scene::fill_scene_light_tables`. So it is 1 whenever a scene is
    /// drawn, front end or race, and its 0 branches (`scene::vertex_light`,
    /// `scene::draw_object_face`, `scene::choose_view_window`, `race::course_faces`,
    /// `race::draw_mirror_view`, the plain light ramp of `fill_light_tables`) are never taken in
    /// play. Not "the mirror on" nor "in a race".
    pub scene_set_up: PlainDword,
    /// Holds the text `files\fmv\intro.avi`; unused.
    pub intro_film_text: [PlainDword; 22],
    /// Set by `scene::set_object_material`, read by `scene::draw_object_face`; now each
    /// view's (`ctx.view.cars.other_way_kind`); unused.
    pub other_way_kind: PlainDword,
    /// The other kind and the distance, written as a pair (`scene::set_object_fade`); now
    /// each view's (`ctx.view.cars.fade_words`); unused.
    pub fade_words: [PlainDword; 2],
    /// The clock now (`race::measure_frame_rate`).
    pub frame_now: PlainDword,
    /// The clock at the last frame (`race::measure_frame_rate`).
    pub frame_last: PlainDword,
    /// The measured frame rate (`race::measure_frame_rate`, `race::frame_text`).
    pub frame_rate: PlainFloat,
    /// The object being drawn or set up (`scene::draw_object_face`, `race::frame_draw`,
    /// `race::car_draw`); now each view's (`ctx.view.cars.object_number`; the bad polygon's log
    /// reads the view's); this one not used.
    pub object_number: PlainDword,
    /// Whether the face reflects (`scene::draw_object_face`); now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub face_reflects: PlainDword,
    /// Whether any face shines (`scene::draw_object_face`); now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub face_any_shine: PlainDword,
    /// The marker object (`race::loadlevel`, `race::frame_draw`).
    pub marker_object: PlainDword,
    /// The face groups shown, built per car; the drawing's is the view's
    /// (`race::car_draw_state`), this one not used.
    pub group_mask: PlainDword,
    /// Set by `race::frame_draw` for the sky (`scene::clip_polygon_by_depth`); now each
    /// view's (`race::face_work::FaceWork`); unused.
    pub no_far_clip: PlainDword,
    /// Nonzero in the editor / framed loading (`race::loadlevel`, `race::loading_bar_frame`,
    /// `scene::choose_view_window`, `scene::texture_levels`).
    pub editor_mode: PlainDword,
    /// One flag per course found (`race::find_present_courses`).
    pub course_present: [PlainDword; 100],
    /// The developers' front end's table; its values are kept in `debug::dev_front_end`,
    /// not here.
    pub dev_front_end_choices: [PlainDword; 103],
    /// A word copied on by `race::objects`.
    pub kept_kind: PlainDword,
    /// The mask kind, 8 tested by `race::frame_draw`, cleared by `race::race_end`
    /// (`scene::list_object_face`).
    pub mask_kind: PlainDword,
    /// 1 lets the levers push the car (`physics::lever_pushes`, `drive::race_tick`).
    pub levers_on: PlainDword,
    /// The view marks (`race::frame_views`, `race::stage_loading`, `drive::finish_race`).
    pub view_marks_on: PlainDword,
    /// Set to 0 by `screens::draw_service_area`; never read.
    pub service_cleared: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl FrameSwitches {
    pub const fn new() -> Self {
        Self {
            intro_film_text: {
                let mut a: [PlainDword; 22] = ZeroCells::ZERO;
                a[10] = PlainDword::new(0x656C6966);
                a[11] = PlainDword::new(0x6D665C73);
                a[12] = PlainDword::new(0x6E695C76);
                a[13] = PlainDword::new(0x2E6F7274);
                a[14] = PlainDword::new(0x697661);
                a
            },
            group_mask: PlainDword::new(0xFFFFFFFF),
            dev_front_end_choices: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(0x2A),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            levers_on: PlainDword::new(1),
            view_marks_on: PlainDword::new(1),
            ..ZeroCells::ZERO
        }
    }
}
