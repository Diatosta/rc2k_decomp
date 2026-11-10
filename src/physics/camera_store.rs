//! The camera's work and the car parts' axle figures (IDA 0x608B08 .. 0x608CB4): the lights file's path text, the two views' figures (heading, roll, pitch, turns, offsets, points, shares) and the blended camera offset and turn, the pitch ease limits, the ground probe counts, the axle figures, the listener's points, and the car draw flags. `#[repr(C)]` with the image's layout, over the race block's bytes. The stage prefs buffer `g_stagePrefs` (0x608CB4 ..) is a separate piece.
//! Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Bytes, Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CameraWork {
    /// +0x0 (0x608B08): the text `files\s\data\ldata` (19 bytes + pad, read in place; `race::stage_prefs`).
    pub lights_path: [Dword; 5],
    /// +0x14 (0x608B1C): the view's zoom handed on (a reading; pushed by `physics::update_camera`, `dword_608B1C`).
    pub camera_zoom_kept: Float,
    /// +0x18 (0x608B20): `flt_608B20`: a view figure the camera arithmetic adds and subtracts (a reading; `physics::update_camera`).
    pub view_shift_a: Float,
    /// +0x1c (0x608B24): `flt_608B24` (a reading; `physics::update_camera`).
    pub view_shift_b: Float,
    /// +0x20 (0x608B28): `flt_608B28` (a reading; `physics::update_camera`).
    pub view_shift_c: Float,
    /// +0x24 (0x608B2C): `flt_608B2C` (a reading; `physics::update_camera`).
    pub view_shift_d: Float,
    /// +0x28 (0x608B30): the camera's offset, blended between the two views, x 65536 (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_x: Dword,
    /// +0x2c (0x608B34): the camera's offset, y (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_y: Dword,
    /// +0x30 (0x608B38): the camera's offset, z (`physics::update_camera`, `physics::place_camera`).
    pub camera_offset_z: Dword,
    /// +0x34 (0x608B3C): not named.
    pub unknown_34: [Dword; 5],
    /// +0x48 (0x608B50): the start's heading, also the heading the camera eases to (`race::car_starts`, `physics::ease_view_turn`).
    pub start_heading: Dword,
    /// +0x4c (0x608B54): the start's roll, and the roll turn target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_roll: Dword,
    /// +0x50 (0x608B58): the start's pitch, and the pitch target (`race::car_starts`, `physics::ease_view_turn`).
    pub start_pitch: Dword,
    /// +0x54 (0x608B5C): the first view's heading, from the car (`physics::take_view_a`).
    pub view_a_heading: Dword,
    /// +0x58 (0x608B60): the first view's roll, halved with the mode's flag (`physics::take_view_a`).
    pub view_a_roll: Dword,
    /// +0x5c (0x608B64): the first view's pitch, halved with the mode's flag (`physics::take_view_a`).
    pub view_a_pitch: Dword,
    /// +0x60 (0x608B68): the second view's heading (`physics::take_view_b`).
    pub view_b_heading: Dword,
    /// +0x64 (0x608B6C): the second view's roll (`physics::take_view_b`).
    pub view_b_roll: Dword,
    /// +0x68 (0x608B70): the second view's pitch (`physics::take_view_b`).
    pub view_b_pitch: Dword,
    /// +0x6c (0x608B74): the blended view's turn, x (`physics::update_camera`, `physics::place_camera`).
    pub view_turn_x: Dword,
    /// +0x70 (0x608B78): the blended view's turn, y (same).
    pub view_turn_y: Dword,
    /// +0x74 (0x608B7C): the blended view's turn, z (same).
    pub view_turn_z: Dword,
    /// +0x78 (0x608B80): the first view's turn figures (record +0x10 .. +0x18; `physics::take_view_a`).
    pub view_a_turn: [Float; 3],
    /// +0x84 (0x608B8C): the second view's turn figures (`physics::take_view_b`).
    pub view_b_turn: [Float; 3],
    /// +0x90 (0x608B98): the first view's offset: record +4 .. +0xC plus `dword_637638` .. (`physics::take_view_a`).
    pub view_a_offset: [Float; 3],
    /// +0x9c (0x608BA4): unreferenced bytes (0xc of them).
    pub _gap_9c: Bytes<12>,
    /// +0xa8 (0x608BB0): the second view's offset (`physics::take_view_b`).
    pub view_b_offset: [Float; 3],
    /// +0xb4 (0x608BBC): unreferenced bytes (0xc of them).
    pub _gap_b4: Bytes<12>,
    /// +0xc0 (0x608BC8): the first view's point (record +0x1C .. +0x24; `physics::take_view_a`).
    pub view_a_point: [Float; 3],
    /// +0xcc (0x608BD4): the second view's point (`physics::take_view_b`).
    pub view_b_point: [Float; 3],
    /// +0xd8 (0x608BE0): copied from `dword_608BF4` (a reading; `physics::update_camera`).
    pub view_blend_aux: Dword,
    /// +0xdc (0x608BE4): the pitch's ease share, 1.0 when `dword_6EF7CC` (`physics::ease_view_turn`).
    pub view_share_pitch: Float,
    /// +0xe0 (0x608BE8): the heading's ease share (same).
    pub view_share_heading: Float,
    /// +0xe4 (0x608BEC): the roll's ease share (same).
    pub view_share_roll: Float,
    /// +0xe8 (0x608BF0): copied from `dword_608C04`, read by `fild` (a reading; `physics::update_camera`).
    pub view_blend_aux_2: Dword,
    /// +0xec (0x608BF4): the first view's record +0x34 (a reading; `physics::take_view_a`).
    pub view_a_aux: Dword,
    /// +0xf0 (0x608BF8): the first view's shares (record +0x28 .. +0x30), copied to the three shares above (`physics::take_view_a`, `physics::update_camera`).
    pub view_a_shares: [Float; 3],
    /// +0xfc (0x608C04): the first view's record +0x38 (a reading; `physics::take_view_a`).
    pub view_a_aux_2: Dword,
    /// +0x100 (0x608C08): the second view's record +0x34 (a reading; `physics::take_view_b`).
    pub view_b_aux: Dword,
    /// +0x104 (0x608C0C): the second view's shares (`physics::take_view_b`).
    pub view_b_shares: [Float; 3],
    /// +0x110 (0x608C18): the second view's record +0x38 (a reading; `physics::take_view_b`).
    pub view_b_aux_2: Dword,
    /// +0x114 (0x608C1C): the pitch the camera eases down to (`physics::ease_pitch_down`, `physics::camera_avoid_ground`).
    pub pitch_floor: Dword,
    /// +0x118 (0x608C20): the pitch the camera eases up to; starts as `camera_ground::PITCH_START` (`physics::ease_pitch_up`).
    pub pitch_ceiling: Dword,
    /// +0x11c (0x608C24): the last pitch step, 0 ends the search (`physics::ease_pitch_up`, `physics::ease_pitch_down`).
    pub pitch_step: Dword,
    /// +0x120 (0x608C28): not named.
    pub unknown_120: [Dword; 5],
    /// +0x134 (0x608C3C): ground probes made, `physics::count_ground_probe` (a reading: statistics).
    pub ground_probes: Dword,
    /// +0x138 (0x608C40): probes that found ground (`physics::count_ground_probe`).
    pub ground_found: Dword,
    /// +0x13c (0x608C44): probes that found the last face again (`physics::count_ground_probe`).
    pub ground_same: Dword,
    /// +0x140 (0x608C48): `g_carAxles` (`race::car_parts`).
    pub car_axles: Dword,
    /// +0x144 (0x608C4C): the front axle's x, negated (`race::car_parts`).
    pub axle_front_neg: Dword,
    /// +0x148 (0x608C50): the front axle's x (`race::car_parts`).
    pub axle_front: Dword,
    /// +0x14c (0x608C54): the rear axle's lift (`race::car_parts`).
    pub axle_rear_lift: Dword,
    /// +0x150 (0x608C58): the axles' height (`race::car_parts`).
    pub axle_height: Dword,
    /// +0x154 (0x608C5C): the front axle's z (`race::car_parts`).
    pub axle_front_z: Dword,
    /// +0x158 (0x608C60): two points of the car's light record x 65536 / 1000: the sound listener's (`drive::set_listener`).
    pub listener_points: [Dword; 6],
    /// +0x170 (0x608C78): unreferenced bytes (0x30 of them).
    pub _gap_170: Bytes<48>,
    /// +0x1a0 (0x608CA8): `dword_608CA8`, set to 0x2100000 while a car is drawn (`race::car_draw`); now each view's (`ctx.view.cars.car_draw_kept`); unused.
    pub car_draw_kept: Dword,
    /// +0x1a4 (0x608CAC): the car's faded bits (record +0x10), -1 after; the drawing's is the view's (`race::face_work`), this one not used.
    pub faded_bits: Dword,
    /// +0x1a8 (0x608CB0): 1 while the car body is drawn (`race::car_draw`); read by nothing, no
    /// longer written.
    pub drawing_car: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl CameraWork {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x608B08, the first byte of the piece.
pub const START: usize = 0x608B08;
/// IDA 0x608CB4, the end of the piece.
pub const END: usize = 0x608CB4;

const _: () = assert!(END - START == core::mem::size_of::<CameraWork>());

/// Where IDA 0x608B08 is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().physics.camera))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::race::owner::race().physics.camera,
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(CameraWork, lights_path), 0x608B08 - START);
        assert_eq!(offset_of!(CameraWork, camera_offset_x), 0x608B30 - START);
        assert_eq!(offset_of!(CameraWork, view_a_heading), 0x608B5C - START);
        assert_eq!(offset_of!(CameraWork, view_turn_x), 0x608B74 - START);
        assert_eq!(offset_of!(CameraWork, view_b_offset), 0x608BB0 - START);
        assert_eq!(offset_of!(CameraWork, view_share_roll), 0x608BEC - START);
        assert_eq!(offset_of!(CameraWork, view_b_shares), 0x608C0C - START);
        assert_eq!(offset_of!(CameraWork, ground_found), 0x608C40 - START);
        assert_eq!(offset_of!(CameraWork, axle_height), 0x608C58 - START);
    }
}
