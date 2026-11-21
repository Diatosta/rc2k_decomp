//! The frame switches: the scene-set-up flag, the frame-rate clock, the courses found, the draw
//! mask kind, the marker object and a few race flags. A plain record (`session::DrawPlain`):
//! written on the game's thread between frames and read there and, as a copy made when a drawing
//! scope opens, by the drawing parts.

use crate::memory::session::{DrawPlain, PlainRead};

#[derive(Clone)]
pub struct FrameSwitches {
    /// The 3D scene is set up for drawing through the light tables: written 1 only, by
    /// `scene::set_up_front_end_scene` (twice), `race::stage_loading` and, for the length of
    /// `scene::fill_light_tables`, `scene::fill_scene_light_tables`. So it is 1 whenever a scene is
    /// drawn, front end or race, and its 0 branches (`scene::vertex_light`,
    /// `scene::draw_object_face`, `scene::choose_view_window`, `race::course_faces`,
    /// `race::draw_mirror_view`, the plain light ramp of `fill_light_tables`) are never taken in
    /// play. Not "the mirror on" nor "in a race".
    pub scene_set_up: u32,
    /// The clock now (`race::measure_frame_rate`).
    pub frame_now: u32,
    /// The clock at the last frame (`race::measure_frame_rate`).
    pub frame_last: u32,
    /// The measured frame rate (`race::measure_frame_rate`, `race::frame_text`).
    pub frame_rate: f32,
    /// The marker object (`race::loadlevel`, `race::frame_draw`).
    pub marker_object: u32,
    /// Nonzero in the editor / framed loading (`race::loadlevel`, `race::loading_bar_frame`,
    /// `scene::choose_view_window`, `scene::texture_levels`).
    pub editor_mode: u32,
    /// One flag per course found (`race::find_present_courses`).
    pub course_present: [u32; 100],
    /// A word copied on by `race::objects`.
    pub kept_kind: u32,
    /// The mask kind, 8 tested by `race::frame_draw`, cleared by `race::race_end`
    /// (`scene::list_object_face`).
    pub mask_kind: u32,
    /// 1 lets the levers push the car (`physics::lever_pushes`, `drive::race_tick`).
    pub levers_on: u32,
    /// The view marks (`race::frame_views`, `race::stage_loading`, `drive::finish_race`).
    pub view_marks_on: u32,
    /// Set to 0 by `screens::draw_service_area`; never read.
    pub service_cleared: u32,
}

impl FrameSwitches {
    pub const fn new() -> Self {
        Self {
            scene_set_up: 0,
            frame_now: 0,
            frame_last: 0,
            frame_rate: 0.0,
            marker_object: 0,
            editor_mode: 0,
            course_present: [0; 100],
            kept_kind: 0,
            mask_kind: 0,
            levers_on: 1,
            view_marks_on: 1,
            service_cleared: 0,
        }
    }
}

/// The switches as one reader finds them: the game's thread's own (read through the store when
/// used), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum FrameSwitchesRef<'a> {
    Store(&'a DrawPlain<FrameSwitches>),
    Copy(&'a FrameSwitches),
}

impl FrameSwitchesRef<'_> {
    /// `f` on the switches as they stand (a short read: nothing in `f` may write the store).
    pub fn read<R>(self, f: impl FnOnce(&FrameSwitches) -> R) -> R {
        match self {
            Self::Store(store) => f(&store.borrow()),
            Self::Copy(switches) => f(switches),
        }
    }

    /// Whether the scene is set up through the light tables (see `FrameSwitches::scene_set_up`).
    pub fn scene_set_up(self) -> u32 {
        self.read(|switches| switches.scene_set_up)
    }

    /// The mask kind.
    pub fn mask_kind(self) -> u32 {
        self.read(|switches| switches.mask_kind)
    }

    /// The marker object.
    pub fn marker_object(self) -> u32 {
        self.read(|switches| switches.marker_object)
    }

    /// The effects on the view.
    pub fn view_marks_on(self) -> u32 {
        self.read(|switches| switches.view_marks_on)
    }

    /// The levers work.
    pub fn levers_on(self) -> u32 {
        self.read(|switches| switches.levers_on)
    }
}

impl<'a> From<&'a PlainRead<FrameSwitches>> for FrameSwitchesRef<'a> {
    fn from(copy: &'a PlainRead<FrameSwitches>) -> Self {
        Self::Copy(copy)
    }
}
