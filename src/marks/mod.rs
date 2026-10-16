//! The marks (part Q): `marks::init_marks` (0x4D12F7, the stage loading's "init mark") and
//! `marks::run_frame` (0x4D1549, each frame of the race): the weather and surface effects
//! (files\m\: masks, edges, snow, splines, surface data), ported bottom-up.

use crate::framework::GameHook;

pub mod add_face_points;
pub mod allocate_links;
pub mod allocate_places;
pub mod allocate_ring;
pub mod blocks;
pub mod centre_spline_points;
pub mod clear_slots;
pub mod copy_block_points;
pub mod find_farthest_points;
pub mod find_spline_ends;
pub mod gather_spline_points;
pub mod grade_spline_points;
pub mod kind_values;
pub mod lines;
pub mod make_noise_curve;
pub mod make_profiles;
pub mod make_wave_tables;
pub mod measure_kind_lines;
pub mod measure_lines;
pub mod measure_slot_lines;
pub mod measure_starting_sums;
pub mod order_spline_points;
pub mod profiles;
pub mod reset_outline;
pub mod reset_settings_blocks;
pub mod scale_corners;
pub mod seed_noise;
pub mod set_kind_values_1;
pub mod set_kind_values_2;
pub mod set_up_blocks;
pub mod setup;
pub mod splines;
pub mod tick_clock;
pub mod turn_spline_points;

pub static HOOKS: &[&dyn GameHook] = &[
    &add_face_points::HOOK,
    &allocate_links::HOOK,
    &allocate_places::HOOK,
    &allocate_ring::HOOK,
    &centre_spline_points::HOOK,
    &clear_slots::HOOK,
    &copy_block_points::HOOK,
    &find_farthest_points::HOOK,
    &find_spline_ends::HOOK,
    &gather_spline_points::HOOK,
    &grade_spline_points::HOOK,
    &make_noise_curve::HOOK,
    &make_profiles::HOOK,
    &make_wave_tables::HOOK,
    &measure_kind_lines::HOOK,
    &measure_lines::HOOK,
    &measure_slot_lines::HOOK,
    &measure_starting_sums::HOOK,
    &order_spline_points::HOOK,
    &reset_outline::HOOK,
    &reset_settings_blocks::HOOK,
    &scale_corners::HOOK,
    &seed_noise::HOOK,
    &set_kind_values_1::HOOK,
    &set_kind_values_2::HOOK,
    &set_up_blocks::HOOK,
    &tick_clock::HOOK,
    &turn_spline_points::HOOK,
];
