//! The view/camera variables of the race block at 0x60626C .. 0x60674C: the drawn car, face group mask,
//! fade alpha, view place and angles (int and float), the 52 view-record pointers at 0x606320, the
//! per-view hide table, the light copies and the camera placement figures. Gaps are unknown bytes.
//! Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct ViewVars {
    /// +0x0 (0x60626C): car +0x69C copied at the frame start (`race::frame_draw`; `car_draw` tests it against the drawn car) (a reading).
    pub view_car_flag_a: Dword,
    /// +0x4 (0x606270): car +0x684 copied at the frame start; index into the stage records (`race::car_draw`, `race::frame_draw`) (a reading).
    pub view_car_stage: Dword,
    /// +0x8 (0x606274): car +0x6F0 copied at the frame start (`race::frame_draw`) (a reading).
    pub view_car_flag_b: Dword,
    /// +0xc (0x606278): `dword_606278`: copied to `view_scale` by `race::frame_draw` (a reading).
    pub view_scale_kept: Float,
    /// +0x10 (0x60627C): `g_drawnCar`: the car being drawn (`race::car_draw`, `scene::set_object_light`, `scene::list_object_face`).
    pub drawn_car: Dword,
    /// +0x14 (0x606280): `dword_606280`: face group mask for the drawn car (`race::car_draw`, `scene::list_object_face`).
    pub group_mask: Dword,
    /// +0x18 (0x606284): `dword_606284`: fade alpha (`scene::draw_polygon`, `scene::draw_object_face`, `race::car_draw`).
    pub fade_alpha: Dword,
    /// +0x1c (0x606288): `dword_606288`: camera height bias (`physics::camera_height_bias`).
    pub camera_height_bias: Dword,
    /// +0x20 (0x60628C): `flt_60628C` (`physics::camera_height_bias`).
    pub camera_bias_scale: Float,
    /// +0x24 (0x606290): `flt_606290`: held to 0 ..= 1 (`physics::camera_height_bias`).
    pub camera_bias_limit: Float,
    /// +0x28 (0x606294): `dword_606294`: a float, the far/scale (`scene::set_view_window`, `race::draw_mirror_view`, `race::race_end`).
    pub view_scale: Float,
    /// +0x2c (0x606298): `g_viewPlace`, `dword_60629C`, `dword_6062A0`: the view place x, y, z (`scene::view_matrix`, `race::frame_draw`, `drive::set_listener`). The original writes the run at once.
    pub view_place: [Dword; 3],
    /// +0x38 (0x6062A4): `flt_6062A4` ..: the place as floats (`race::course_faces`, `race::frame_draw`).
    pub view_place_f: [Float; 3],
    /// +0x44 (0x6062B0): `dword_6062B0`: view x (`race::frame_draw`).
    pub view_x: Dword,
    /// +0x48 (0x6062B4): `dword_6062B4`: view z (`race::frame_draw`).
    pub view_z: Dword,
    /// +0x4c (0x6062B8): `g_viewAngles` y, x, z then a second triple at `dword_6062C4` ..; (`scene::view_matrix`, `race::frame_draw`).
    pub view_angles: [Dword; 6],
    /// +0x64 (0x6062D0): `flt_6062D0` (`physics::update_camera`).
    pub camera_zoom_hold: Float,
    /// +0x68 (0x6062D4): not named.
    pub unknown_68: Dword,
    /// +0x6c (0x6062D8): `flt_6062D8` ..: vector whose length is taken (`physics::update_camera`) (a reading).
    pub camera_vector: [Float; 3],
    /// +0x78 (0x6062E4): not named.
    pub unknown_78: [Dword; 14],
    /// +0xb0 (0x60631C): `flt_60631C`: held to 0 ..= 1 (`physics::camera_height_bias`, `physics::update_camera`).
    pub camera_distance: Float,
    /// +0xb4 (0x606320): `off_606320`: 0x34 pointers to view records (`physics::camera_view`, `drive::pick_replay_camera`, `marks::stage_record`, `race::car_draw`).
    pub view_records: [Dword; 52],
    /// +0x184 (0x6063F0): `dword_6063F0`: the view kind (`drive::set_camera_view`) (a reading).
    pub view_kind: Dword,
    /// +0x188 (0x6063F4): not named.
    pub unknown_188: [Dword; 33],
    /// +0x20c (0x606478): `dword_606478[]`: by camera view (`race::frame_draw`).
    pub view_hides: [Dword; 34],
    /// +0x294 (0x606500): not named.
    pub unknown_294: [Dword; 28],
    /// +0x304 (0x606570): `flt_606570`: from light +0x24 (`physics::update_camera`).
    pub light_a: Float,
    /// +0x308 (0x606574): `flt_606574`: from light +0x28 (`physics::update_camera`).
    pub light_b: Float,
    /// +0x30c (0x606578): not named.
    pub unknown_30c: [Dword; 46],
    /// +0x3c4 (0x606630): `dword_606630`: from light +0xC / +0x1C (`physics::update_camera`).
    pub light_c: Dword,
    /// +0x3c8 (0x606634): not named.
    pub unknown_3c8: [Dword; 17],
    /// +0x40c (0x606678): `dword_606678` (`physics::place_camera`).
    pub place_a: Dword,
    /// +0x410 (0x60667C): `dword_60667C` (`physics::place_camera`).
    pub place_b: Dword,
    /// +0x414 (0x606680): not named.
    pub unknown_414: [Dword; 3],
    /// +0x420 (0x60668C): `dword_60668C` (`physics::place_camera`).
    pub place_last_a: Dword,
    /// +0x424 (0x606690): not named.
    pub unknown_424: [Dword; 17],
    /// +0x468 (0x6066D4): `dword_6066D4` (`physics::place_camera`).
    pub place_c: Dword,
    /// +0x46c (0x6066D8): `dword_6066D8` (`physics::place_camera`).
    pub place_d: Dword,
    /// +0x470 (0x6066DC): not named.
    pub unknown_470: [Dword; 3],
    /// +0x47c (0x6066E8): `dword_6066E8` (`physics::place_camera`).
    pub place_last_b: Dword,
    /// +0x480 (0x6066EC): not named.
    pub unknown_480: [Dword; 16],
    /// +0x4c0 (0x60672C): `dword_60672C`: set 0 (`drive::finish_camera`).
    pub finish_zero: Dword,
    /// +0x4c4 (0x606730): `dword_606730` (`physics::place_camera`).
    pub place_e: Dword,
    /// +0x4c8 (0x606734): `dword_606734` (`physics::place_camera`).
    pub place_f: Dword,
    /// +0x4cc (0x606738): not named.
    pub unknown_4cc: [Dword; 3],
    /// +0x4d8 (0x606744): `dword_606744` (`physics::place_camera`, `drive::finish_race`).
    pub place_last_c: Dword,
    /// +0x4dc (0x606748): `dword_606748` (`drive::finish_race`).
    pub finish_value: Dword,
}

/// The piece: the race block's bytes at IDA 0x60626C seen as `ViewVars`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = ViewVars;
    fn deref(&self) -> &ViewVars {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const ViewVars) }
    }
}

pub static VIEW_VARS: InRaceBlock = InRaceBlock;

/// IDA 0x60626C, the first byte of the piece.
pub const START: usize = 0x60626C;
/// IDA 0x60674C, the end of the piece.
pub const END: usize = 0x60674C;

const _: () = assert!(END - START == core::mem::size_of::<ViewVars>());

/// Where IDA 0x60626C is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
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
        assert_eq!(offset_of!(ViewVars, view_car_flag_a), 0x60626C - START);
        assert_eq!(offset_of!(ViewVars, fade_alpha), 0x606284 - START);
        assert_eq!(offset_of!(ViewVars, view_place_f), 0x6062A4 - START);
        assert_eq!(offset_of!(ViewVars, camera_distance), 0x60631C - START);
        assert_eq!(offset_of!(ViewVars, light_c), 0x606630 - START);
        assert_eq!(offset_of!(ViewVars, place_last_b), 0x6066E8 - START);
    }
}
