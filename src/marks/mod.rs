//! The marks (part Q): `marks::init_marks` (0x4D12F7, the stage loading's "init mark") and
//! `marks::run_frame` (0x4D1549, each frame of the race): the weather and surface effects
//! (files\m\: masks, edges, snow, splines, surface data), ported bottom-up.

use crate::framework::GameHook;

pub mod add_face_points;
pub mod allocate_links;
pub mod allocate_places;
pub mod allocate_ring;
pub mod blocks;
pub mod bound_spline_points;
pub mod centre_spline_points;
pub mod clamp_pair;
pub mod clear_slots;
pub mod collect_points_a;
pub mod collect_points_b;
pub mod copy_block_points;
pub mod corners;
pub mod draw_mask;
pub mod draw_mask_shape;
pub mod edges;
pub mod fill_mask_spans;
pub mod find_farthest_pairs;
pub mod find_farthest_points;
pub mod find_spline_ends;
pub mod gather_spline_points;
pub mod grade_spline_points;
pub mod kind_values;
pub mod lines;
pub mod load_edges;
pub mod load_kind_data;
pub mod load_splines;
pub mod make_corners;
pub mod make_masks;
pub mod make_noise_curve;
pub mod make_profiles;
pub mod make_shape_set;
pub mod make_shape_sets;
pub mod make_spline;
pub mod make_wave_tables;
pub mod map_pixel;
pub mod map_shape;
pub mod map_shape_rows;
pub mod map_shapes;
pub mod masks;
pub mod measure_kind_lines;
pub mod measure_lines;
pub mod measure_slot_lines;
pub mod measure_starting_sums;
pub mod order_spline_points;
pub mod pixels;
pub mod profiles;
pub mod reset_mask_cursor;
pub mod reset_outline;
pub mod reset_settings_blocks;
pub mod rings;
pub mod scale_corners;
pub mod scale_rows3;
pub mod seed_noise;
pub mod set_kind_values_1;
pub mod set_kind_values_2;
pub mod set_line_turn;
pub mod set_line_turn_back;
pub mod set_up_blocks;
pub mod set_up_hexagons;
pub mod set_up_squares;
pub mod setup;
pub mod shape_points;
pub mod shapes;
pub mod smooth_steps;
pub mod span_edges;
pub mod span_quad;
pub mod span_ring;
pub mod span_triangle;
pub mod spans;
pub mod splines;
pub mod tick_clock;
pub mod turn_all_shapes;
pub mod turn_pair;
pub mod turn_shape_points;
pub mod turn_spline_points;
pub mod turns;

pub static HOOKS: &[&dyn GameHook] = &[
    &add_face_points::HOOK,
    &allocate_links::HOOK,
    &allocate_places::HOOK,
    &allocate_ring::HOOK,
    &bound_spline_points::HOOK,
    &centre_spline_points::HOOK,
    &clamp_pair::HOOK,
    &clear_slots::HOOK,
    &collect_points_a::HOOK,
    &collect_points_b::HOOK,
    &copy_block_points::HOOK,
    &draw_mask::HOOK,
    &draw_mask_shape::HOOK,
    &fill_mask_spans::HOOK,
    &find_farthest_pairs::HOOK,
    &find_farthest_points::HOOK,
    &find_spline_ends::HOOK,
    &gather_spline_points::HOOK,
    &grade_spline_points::HOOK,
    &load_edges::HOOK,
    &load_kind_data::HOOK,
    &load_splines::HOOK,
    &make_corners::HOOK,
    &make_masks::HOOK,
    &make_noise_curve::HOOK,
    &make_profiles::HOOK,
    &make_shape_set::HOOK,
    &make_shape_sets::HOOK,
    &make_spline::HOOK,
    &make_wave_tables::HOOK,
    &map_pixel::HOOK,
    &map_shape::HOOK,
    &map_shape_rows::HOOK,
    &map_shapes::HOOK,
    &measure_kind_lines::HOOK,
    &measure_lines::HOOK,
    &measure_slot_lines::HOOK,
    &measure_starting_sums::HOOK,
    &order_spline_points::HOOK,
    &reset_mask_cursor::HOOK,
    &reset_outline::HOOK,
    &reset_settings_blocks::HOOK,
    &scale_corners::HOOK,
    &scale_rows3::HOOK,
    &seed_noise::HOOK,
    &set_kind_values_1::HOOK,
    &set_kind_values_2::HOOK,
    &set_line_turn::HOOK,
    &set_line_turn_back::HOOK,
    &set_up_blocks::HOOK,
    &set_up_hexagons::HOOK,
    &set_up_squares::HOOK,
    &smooth_steps::HOOK,
    &span_edges::HOOK,
    &span_quad::HOOK,
    &span_ring::HOOK,
    &span_triangle::HOOK,
    &tick_clock::HOOK,
    &turn_all_shapes::HOOK,
    &turn_pair::HOOK,
    &turn_shape_points::HOOK,
    &turn_spline_points::HOOK,
];
