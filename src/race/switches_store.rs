//! The frame and object work, first part (IDA 0x621480 .. 0x6218B0; phase 7, typed): the light and view switches, the object being drawn (number, material, fade, face flags, group mask), the frame-rate clock, the courses found, and a few race flags. `#[repr(C)]` with the image layout over the race block bytes. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FrameSwitches {
    /// +0x0 (0x621480): `dword_621480`: the 3D scene is set up for drawing through the light tables: written 1 only, by `scene::set_up_front_end_scene` (twice), `race::stage_loading` and, for the length of `scene::fill_light_tables`, `scene::fill_scene_light_tables` (IDA writers 0x475027, 0x475BF0, 0x444213, 0x4433B3/C2; none writes 0). So it is 1 whenever a scene is drawn, front end or race, and its 0 branches (`scene::vertex_light`, `scene::draw_object_face`, `scene::choose_view_window`, `race::course_faces`, `race::draw_mirror_view`, the plain light ramp of `fill_light_tables`) are never taken in play. Not "the mirror on" nor "in a race".
    pub scene_set_up: Dword,
    /// +0x4 (0x621484): holds the text `files\fmv\intro.avi` (0x6214AC), used only by the unported `sub_442FDB`.
    pub intro_film_text: [Dword; 22],
    /// +0x5c (0x6214DC): `dword_6214DC`: set by `scene::set_object_material`, read by `scene::draw_object_face`; now each view's (`ctx.view.cars.other_way_kind`); unused.
    pub other_way_kind: Dword,
    /// +0x60 (0x6214E0): `qword_6214E0`: the other kind and the distance, written as a pair (`scene::set_object_fade`); now each view's (`ctx.view.cars.fade_words`); unused.
    pub fade_words: [Dword; 2],
    /// +0x68 (0x6214E8): `dword_6214E8`: the clock now (`race::measure_frame_rate`).
    pub frame_now: Dword,
    /// +0x6c (0x6214EC): `dword_6214EC`: the clock at the last frame (`race::measure_frame_rate`).
    pub frame_last: Dword,
    /// +0x70 (0x6214F0): `g_frameRate`: the measured frame rate (`race::measure_frame_rate`, `race::frame_text`).
    pub frame_rate: Float,
    /// +0x74 (0x6214F4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_74: [Dword; 2],
    /// +0x7c (0x6214FC): `g_objectNumber`: the object being drawn or set up (`scene::draw_object_face`, `race::frame_draw`, `race::car_draw`); now each view's (`ctx.view.cars.object_number`; the bad polygon's log reads the view's); this one not used.
    pub object_number: Dword,
    /// +0x80 (0x621500): `g_faceReflects` (`scene::draw_object_face`); now each view's (`race::face_work::FaceWork`); unused.
    pub face_reflects: Dword,
    /// +0x84 (0x621504): `g_faceAnyShine` (`scene::draw_object_face`); now each view's (`race::face_work::FaceWork`); unused.
    pub face_any_shine: Dword,
    /// +0x88 (0x621508): `dword_621508`: the marker object (`race::loadlevel`, `race::frame_draw`).
    pub marker_object: Dword,
    /// +0x8c (0x62150C): `dword_62150C`: the face groups shown, built per car; the drawing's is the view's
    /// (`race::car_draw_state`), this one not used.
    pub group_mask: Dword,
    /// +0x90 (0x621510): `dword_621510`: set by `race::frame_draw` for the sky (`scene::clip_polygon_by_depth`); now each view's (`race::face_work::FaceWork`); unused.
    pub no_far_clip: Dword,
    /// +0x94 (0x621514): `dword_621514`: nonzero in the editor / framed loading (`race::loadlevel`, `race::loading_bar_frame`, `scene::choose_view_window`, `scene::texture_levels`).
    pub editor_mode: Dword,
    /// +0x98 (0x621518): `g_coursePresent`: one flag per course found (`race::find_present_courses`).
    pub course_present: [Dword; 100],
    /// +0x228 (0x6216A8): the developers' front end's table (`dev_front_end::draw`, `sub_442F87`, `sub_442FB1` use 0x6216A8 .. 0x62183C); the port keeps those values in `debug::dev_front_end`, not here.
    pub dev_front_end_choices: [Dword; 103],
    /// +0x3c4 (0x621844): `dword_621844`: copied to `dword_5CCF7C` (`race::objects`).
    pub kept_kind: Dword,
    /// +0x3c8 (0x621848): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3c8: [Dword; 10],
    /// +0x3f0 (0x621870): `dword_621870`: the mask kind, 8 tested by `race::frame_draw`, cleared by `race::race_end` (`scene::list_object_face`).
    pub mask_kind: Dword,
    /// +0x3f4 (0x621874): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3f4: [Dword; 4],
    /// +0x404 (0x621884): `dword_621884`: 1 lets the levers push the car (`physics::lever_pushes`, `drive::race_tick`).
    pub levers_on: Dword,
    /// +0x408 (0x621888): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_408: [Dword; 2],
    /// +0x410 (0x621890): `dword_621890`: the view marks (`race::frame_views`, `race::stage_loading`, `drive::finish_race`).
    pub view_marks_on: Dword,
    /// +0x414 (0x621894): holds 0x6218A0, used by `dev_front_end::draw` / `set_up_race` and the unported `sub_4431B7` (the developers' front end); the port keeps it in `debug::dev_front_end`, not here.
    pub unknown_414: [Dword; 6],
    /// +0x42c (0x6218AC): `dword_6218AC`: set to 0 by `screens::draw_service_area`, read nowhere known.
    pub service_cleared: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl FrameSwitches {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x621480, the first byte of the piece.
pub const START: usize = 0x621480;
/// IDA 0x6218B0, the end of the piece.
pub const END: usize = 0x6218B0;

const _: () = assert!(END - START == core::mem::size_of::<FrameSwitches>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    std::ptr::from_ref(&race().shared.switches).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &race().shared.switches,
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(FrameSwitches, scene_set_up), 0x0);
        assert_eq!(offset_of!(FrameSwitches, object_number), 0x7c);
        assert_eq!(offset_of!(FrameSwitches, editor_mode), 0x94);
        assert_eq!(offset_of!(FrameSwitches, service_cleared), 0x42c);
    }
}
