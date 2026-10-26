//! The frame and object work, first part (IDA 0x621480 .. 0x6218B0; phase 7, typed): the light and view switches, the object being drawn (number, material, fade, face flags, group mask), the frame-rate clock, the courses found, and a few race flags. `#[repr(C)]` with the image layout over the race block bytes. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct FrameSwitches {
    /// +0x0 (0x621480): `dword_621480`: nonzero in a race, 0 in the front end scene; 1 set by `race::stage_loading` (`scene::vertex_light`, `scene::draw_object_face`, `race::course_faces`) (a reading).
    pub other_way: Dword,
    /// +0x4 (0x621484): not named.
    pub unknown_4: [Dword; 22],
    /// +0x5c (0x6214DC): `dword_6214DC`: set by `scene::set_object_material`, read by `scene::draw_object_face`.
    pub other_way_kind: Dword,
    /// +0x60 (0x6214E0): `qword_6214E0`: the other kind and the distance, written as a pair (`scene::set_object_fade`).
    pub fade_words: [Dword; 2],
    /// +0x68 (0x6214E8): `dword_6214E8`: the clock now (`race::measure_frame_rate`).
    pub frame_now: Dword,
    /// +0x6c (0x6214EC): `dword_6214EC`: the clock at the last frame (`race::measure_frame_rate`).
    pub frame_last: Dword,
    /// +0x70 (0x6214F0): `g_frameRate`: the measured frame rate (`race::measure_frame_rate`, `race::frame_text`).
    pub frame_rate: Float,
    /// +0x74 (0x6214F4): not named.
    pub unknown_74: [Dword; 2],
    /// +0x7c (0x6214FC): `g_objectNumber`: the object being drawn or set up (`scene::draw_object_face`, `race::frame_draw`, `race::car_draw`).
    pub object_number: Dword,
    /// +0x80 (0x621500): `g_faceReflects` (`scene::draw_object_face`).
    pub face_reflects: Dword,
    /// +0x84 (0x621504): `g_faceAnyShine` (`scene::draw_object_face`).
    pub face_any_shine: Dword,
    /// +0x88 (0x621508): `dword_621508`: the marker object (`race::loadlevel`, `race::frame_draw`).
    pub marker_object: Dword,
    /// +0x8c (0x62150C): `dword_62150C`: the face groups shown, built per car (`race::car_draw`, `scene::list_object_face`).
    pub group_mask: Dword,
    /// +0x90 (0x621510): `dword_621510`: set by `race::frame_draw` for the sky (`scene::clip_polygon_by_depth`).
    pub no_far_clip: Dword,
    /// +0x94 (0x621514): `dword_621514`: nonzero in the editor / framed loading (`race::loadlevel`, `race::loading_bar_frame`, `scene::choose_view_window`, `scene::texture_levels`) (a reading).
    pub editor_mode: Dword,
    /// +0x98 (0x621518): `g_coursePresent`: one flag per course found (`race::find_present_courses`).
    pub course_present: [Dword; 100],
    /// +0x228 (0x6216A8): not named.
    pub unknown_228: [Dword; 103],
    /// +0x3c4 (0x621844): `dword_621844`: copied to `dword_5CCF7C` (`race::objects`).
    pub kept_kind: Dword,
    /// +0x3c8 (0x621848): not named.
    pub unknown_3c8: [Dword; 10],
    /// +0x3f0 (0x621870): `dword_621870`: the mask kind, 8 tested by `race::frame_draw`, cleared by `race::race_end` (`scene::list_object_face`).
    pub mask_kind: Dword,
    /// +0x3f4 (0x621874): not named.
    pub unknown_3f4: [Dword; 4],
    /// +0x404 (0x621884): `dword_621884`: 1 lets the levers push (`physics::lever_pushes`, `drive::race_tick`) (a reading).
    pub levers_on: Dword,
    /// +0x408 (0x621888): not named.
    pub unknown_408: [Dword; 2],
    /// +0x410 (0x621890): `dword_621890`: the view marks (`race::frame_views`, `race::stage_loading`, `drive::finish_race`).
    pub view_marks_on: Dword,
    /// +0x414 (0x621894): not named.
    pub unknown_414: [Dword; 6],
    /// +0x42c (0x6218AC): `dword_6218AC`: set to 0 by `screens::draw_service_area`, read nowhere known.
    pub service_cleared: Dword,
}

/// The piece: the race block's bytes at IDA 0x621480 seen as `FrameSwitches`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = FrameSwitches;
    fn deref(&self) -> &FrameSwitches {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host(piece_addr()) as *const FrameSwitches) }
    }
}

pub static FRAME_SWITCHES: InRaceBlock = InRaceBlock;

/// IDA 0x621480, the first byte of the piece.
pub const START: usize = 0x621480;
/// IDA 0x6218B0, the end of the piece.
pub const END: usize = 0x6218B0;

const _: () = assert!(END - START == core::mem::size_of::<FrameSwitches>());

/// Where IDA 0x621480 is now (in the race block).
pub fn piece_addr() -> u32 {
    crate::race::store::home(START) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(FrameSwitches, other_way), 0x621480 - START);
        assert_eq!(offset_of!(FrameSwitches, object_number), 0x6214FC - START);
        assert_eq!(offset_of!(FrameSwitches, editor_mode), 0x621514 - START);
        assert_eq!(offset_of!(FrameSwitches, service_cleared), 0x6218AC - START);
    }
}
