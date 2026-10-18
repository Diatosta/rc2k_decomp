//! The marks (part Q): `marks::init_marks` (0x4D12F7, the stage loading's "init mark") and
//! `marks::run_frame` (0x4D1549, each frame of the race): the weather and surface effects
//! (files\m\: masks, edges, snow, splines, surface data), ported bottom-up.

use crate::framework::GameHook;

pub mod add_drift_particle;
pub mod add_face_points;
pub mod add_mark_slot;
pub mod adjugate3;
pub mod aim_vector;
pub mod allocate_links;
pub mod allocate_places;
pub mod allocate_ring;
pub mod begin_quads;
pub mod bend_points;
pub mod blend_corner;
pub mod blend_points;
pub mod blocks;
pub mod bound_spline_points;
pub mod burst_particles;
pub mod cars_particles;
pub mod centre_spline_points;
pub mod clamp_pair;
pub mod clear_slots;
pub mod clip_quad;
pub mod collect_points_a;
pub mod collect_points_b;
pub mod copy_block_points;
pub mod copy_first_point;
pub mod corners;
pub mod draw_cell_sprite;
pub mod draw_cell_sprite_2;
pub mod draw_flare_part;
pub mod draw_frame;
pub mod draw_glow;
pub mod draw_lens_flare;
pub mod draw_mark_slots;
pub mod draw_mask;
pub mod draw_mask_shape;
pub mod draw_overlays;
pub mod draw_particle;
pub mod draw_plane;
pub mod draw_pong_box;
pub mod draw_ribbon;
pub mod draw_shadow;
pub mod draw_snow;
pub mod draw_square;
pub mod draw_streak;
pub mod draw_subcell_sprite;
pub mod draw_turned_sprite;
pub mod drift_offsets;
pub mod edges;
pub mod emit_body_particles;
pub mod emit_bursts;
pub mod emit_bursts_2;
pub mod emit_car_particles;
pub mod emit_kind_1;
pub mod emit_particle_11;
pub mod emit_particle_13;
pub mod emit_particle_18;
pub mod emit_particle_19;
pub mod emit_particle_21;
pub mod emit_particle_22;
pub mod emit_particle_5;
pub mod emit_particle_6;
pub mod emit_wheel_particles;
pub mod emitters;
pub mod expire_hexagons;
pub mod expire_squares;
pub mod fill_mask_spans;
pub mod find_farthest_pairs;
pub mod find_farthest_points;
pub mod find_spline_ends;
pub mod flare_frame;
pub mod frame_marks;
pub mod gather_spline_points;
pub mod grade_spline_points;
pub mod hexagons_drift;
pub mod hexagons_frame;
pub mod hexagons_setup;
pub mod hexagons_spawn;
pub mod hexagons_step;
pub mod hexagons_top;
pub mod init_mark_step;
pub mod init_marks;
pub mod jitter_point;
pub mod keep_vectors;
pub mod kept_turns;
pub mod kind_values;
pub mod leaf_test;
pub mod lines;
pub mod load_edges;
pub mod load_kind_data;
pub mod load_mask_pictures;
pub mod load_splines;
pub mod make_corners;
pub mod make_fade_colours;
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
pub mod morph_object_points;
pub mod move_pong_ball;
pub mod move_pong_bat;
pub mod order_spline_points;
pub mod pick_hexagon;
pub mod pick_shapes;
pub mod pick_square;
pub mod pixels;
pub mod place_graded_points;
pub mod place_shape_points;
pub mod place_spline;
pub mod place_view;
pub mod place_view_2;
pub mod pong;
pub mod pong_ball_hits;
pub mod pong_bat_back;
pub mod pong_cell;
pub mod pong_digit;
pub mod pong_score;
pub mod pong_setup;
pub mod profiles;
pub mod reset_mask_cursor;
pub mod reset_pong;
pub mod reset_settings_blocks;
pub mod ring_frame;
pub mod rings;
pub mod run_frame;
pub mod run_particles;
pub mod run_pong;
pub mod run_rain;
pub mod run_ribbons;
pub mod run_ring;
pub mod run_shapes;
pub mod run_snow;
pub mod run_splines;
pub mod run_squares;
pub mod scale_axes;
pub mod scale_corners;
pub mod scale_kept_vectors;
pub mod scale_rows3;
pub mod scale_setting;
pub mod scale_vectors_by;
pub mod seed_noise;
pub mod set_kind_values_1;
pub mod set_kind_values_2;
pub mod set_line_turn;
pub mod set_line_turn_back;
pub mod set_quad_uvs;
pub mod set_up_blocks;
pub mod set_up_hexagons;
pub mod set_up_squares;
pub mod set_up_surface;
pub mod setup;
pub mod shape_points;
pub mod shapes;
pub mod shapes_frame;
pub mod smooth_steps;
pub mod span_edges;
pub mod span_quad;
pub mod span_ring;
pub mod span_triangle;
pub mod spans;
pub mod spawn_hexagon;
pub mod spawn_square;
pub mod splines;
pub mod spread_point;
pub mod squares_frame;
pub mod squares_setup;
pub mod squares_spawn;
pub mod squares_step;
pub mod stage_record;
pub mod swap_mask;
pub mod tick_clock;
pub mod tidy_marks;
pub mod tilt_shape;
pub mod trail_particles;
pub mod turn_all_shapes;
pub mod turn_kept_vectors;
pub mod turn_kept_xz;
pub mod turn_kept_zy;
pub mod turn_pair;
pub mod turn_shape_points;
pub mod turn_spline_points;
pub mod turns;
pub mod view_frame;
pub mod view_heading;
pub mod view_motion;
pub mod view_step;

pub static HOOKS: &[&dyn GameHook] = &[
    &add_drift_particle::HOOK,
    &add_face_points::HOOK,
    &add_mark_slot::HOOK,
    &adjugate3::HOOK,
    &aim_vector::HOOK,
    &allocate_links::HOOK,
    &allocate_places::HOOK,
    &allocate_ring::HOOK,
    &begin_quads::HOOK,
    &bend_points::HOOK,
    &blend_corner::HOOK,
    &blend_points::HOOK,
    &bound_spline_points::HOOK,
    &burst_particles::HOOK,
    &cars_particles::HOOK,
    &centre_spline_points::HOOK,
    &clamp_pair::HOOK,
    &clear_slots::HOOK,
    &clip_quad::HOOK,
    &collect_points_a::HOOK,
    &collect_points_b::HOOK,
    &copy_block_points::HOOK,
    &copy_first_point::HOOK,
    &draw_cell_sprite::HOOK,
    &draw_cell_sprite_2::HOOK,
    &draw_flare_part::HOOK,
    &draw_frame::HOOK,
    &draw_glow::HOOK,
    &draw_lens_flare::HOOK,
    &draw_mark_slots::HOOK,
    &draw_mask::HOOK,
    &draw_mask_shape::HOOK,
    &draw_overlays::HOOK,
    &draw_particle::HOOK,
    &draw_plane::HOOK,
    &draw_pong_box::HOOK,
    &draw_ribbon::HOOK,
    &draw_shadow::HOOK,
    &draw_snow::HOOK,
    &draw_square::HOOK,
    &draw_streak::HOOK,
    &draw_subcell_sprite::HOOK,
    &draw_turned_sprite::HOOK,
    &drift_offsets::HOOK,
    &emit_body_particles::HOOK,
    &emit_bursts::HOOK,
    &emit_bursts_2::HOOK,
    &emit_car_particles::HOOK,
    &emit_kind_1::HOOK,
    &emit_particle_11::HOOK,
    &emit_particle_13::HOOK,
    &emit_particle_18::HOOK,
    &emit_particle_19::HOOK,
    &emit_particle_21::HOOK,
    &emit_particle_22::HOOK,
    &emit_particle_5::HOOK,
    &emit_particle_6::HOOK,
    &emit_wheel_particles::HOOK,
    &expire_hexagons::HOOK,
    &expire_squares::HOOK,
    &fill_mask_spans::HOOK,
    &find_farthest_pairs::HOOK,
    &find_farthest_points::HOOK,
    &find_spline_ends::HOOK,
    &flare_frame::HOOK,
    &frame_marks::HOOK,
    &gather_spline_points::HOOK,
    &grade_spline_points::HOOK,
    &hexagons_drift::HOOK,
    &hexagons_frame::HOOK,
    &hexagons_setup::HOOK,
    &hexagons_spawn::HOOK,
    &hexagons_step::HOOK,
    &hexagons_top::HOOK,
    &init_mark_step::HOOK,
    &init_marks::HOOK,
    &jitter_point::HOOK,
    &keep_vectors::HOOK,
    &load_edges::HOOK,
    &load_kind_data::HOOK,
    &load_mask_pictures::HOOK,
    &load_splines::HOOK,
    &make_corners::HOOK,
    &make_fade_colours::HOOK,
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
    &morph_object_points::HOOK,
    &move_pong_ball::HOOK,
    &move_pong_bat::HOOK,
    &order_spline_points::HOOK,
    &pick_hexagon::HOOK,
    &pick_shapes::HOOK,
    &pick_square::HOOK,
    &place_graded_points::HOOK,
    &place_shape_points::HOOK,
    &place_spline::HOOK,
    &place_view::HOOK,
    &place_view_2::HOOK,
    &pong_ball_hits::HOOK,
    &pong_bat_back::HOOK,
    &pong_cell::HOOK,
    &pong_digit::HOOK,
    &pong_score::HOOK,
    &pong_setup::HOOK,
    &reset_mask_cursor::HOOK,
    &reset_pong::HOOK,
    &reset_settings_blocks::HOOK,
    &ring_frame::HOOK,
    &run_frame::HOOK,
    &run_particles::HOOK,
    &run_pong::HOOK,
    &run_rain::HOOK,
    &run_ribbons::HOOK,
    &run_ring::HOOK,
    &run_shapes::HOOK,
    &run_snow::HOOK,
    &run_splines::HOOK,
    &run_squares::HOOK,
    &scale_axes::HOOK,
    &scale_corners::HOOK,
    &scale_kept_vectors::HOOK,
    &scale_rows3::HOOK,
    &scale_setting::HOOK,
    &scale_vectors_by::HOOK,
    &seed_noise::HOOK,
    &set_kind_values_1::HOOK,
    &set_kind_values_2::HOOK,
    &set_line_turn::HOOK,
    &set_line_turn_back::HOOK,
    &set_quad_uvs::HOOK,
    &set_up_blocks::HOOK,
    &set_up_hexagons::HOOK,
    &set_up_squares::HOOK,
    &set_up_surface::HOOK,
    &shapes_frame::HOOK,
    &smooth_steps::HOOK,
    &span_edges::HOOK,
    &span_quad::HOOK,
    &span_ring::HOOK,
    &span_triangle::HOOK,
    &spawn_hexagon::HOOK,
    &spawn_square::HOOK,
    &spread_point::HOOK,
    &squares_frame::HOOK,
    &squares_setup::HOOK,
    &squares_spawn::HOOK,
    &squares_step::HOOK,
    &stage_record::HOOK,
    &swap_mask::HOOK,
    &tick_clock::HOOK,
    &tidy_marks::HOOK,
    &tilt_shape::HOOK,
    &trail_particles::HOOK,
    &turn_all_shapes::HOOK,
    &turn_kept_vectors::HOOK,
    &turn_kept_xz::HOOK,
    &turn_kept_zy::HOOK,
    &turn_pair::HOOK,
    &turn_shape_points::HOOK,
    &turn_spline_points::HOOK,
    &view_frame::HOOK,
    &view_heading::HOOK,
    &view_motion::HOOK,
    &view_step::HOOK,
];
