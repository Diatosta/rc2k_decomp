//! The 3D scene: the game's own code above the Direct3D layer (`d3d`).

use crate::framework::GameHook;

pub mod add_car_light;
pub mod add_car_lights;
pub mod add_object_shape;
pub mod add_point_lights;
pub mod allocate_object_memory;
pub mod allocate_point_records;
pub mod angle_between;
pub mod approximate_distance;
pub mod assign_kind_textures;
pub mod assign_menu_textures;
pub mod begin_scene;
pub mod build_texture_catalogue;
pub mod cached_reflection_coordinates;
pub mod carousel_pictures;
pub mod carousel_set_ups;
pub mod choose_carousel_pictures;
pub mod clear_depth_buffer;
pub mod clear_fog;
pub mod clear_slot_kinds;
pub mod clear_slot_textures;
pub mod clip;
pub mod clip_asm;
pub mod clip_polygon_by_depth;
pub mod clip_polygon_to_screen;
pub mod clip_test;
pub mod copy_matrix_with_place;
pub mod count_texture_needs;
pub mod cross_product;
pub mod draw_car_object;
pub mod draw_carousel;
pub mod draw_carousel_entry;
pub mod draw_clipped_polygon;
pub mod draw_highlighted_object;
pub mod draw_main_menu_carousel;
pub mod draw_main_menu_objects;
pub mod draw_marked_polygon;
pub mod draw_menu_object;
pub mod draw_object;
pub mod draw_object_face;
pub mod draw_object_faces;
pub mod draw_object_polygon;
pub mod draw_polygon;
pub mod draw_scene_object;
pub mod draw_shown_object;
pub mod end_scene;
pub mod face_faces_away;
pub mod face_sort_key;
pub mod fill_engine_tables;
pub mod fill_light_ramp;
pub mod fill_light_scale;
pub mod fill_light_tables;
pub mod fill_scene_light_tables;
pub mod find_object_of_kind;
pub mod fog_polygon;
pub mod init_texture_levels;
pub mod invert_turn;
pub mod known_kind;
pub mod known_kind_picture;
pub mod known_kind_plate;
pub mod light_dot;
pub mod light_tables;
pub mod list_object_face;
pub mod list_object_faces;
pub mod list_object_textures;
pub mod load_changed_texture_picture;
pub mod load_front_end_objects;
pub mod load_number_plates;
pub mod load_object;
pub mod load_slot_texture;
pub mod load_slot_textures;
pub mod load_texture_picture;
pub mod load_wanted_textures;
pub mod make_object_matrix;
pub mod map_texture_places;
pub mod matrix;
pub mod measure_object;
pub mod multiply_matrices;
pub mod next_noise;
pub mod objects;
pub mod place_object_points;
pub mod polygon;
pub mod prepare_menu_scene;
pub mod project_point;
pub mod project_polygon;
pub mod raise_texture_needs;
pub mod read_kind_records;
pub mod reflect_vector;
pub mod reflection_coordinates;
pub mod reload_slot_textures;
pub mod reload_system_textures;
pub mod reset_objects;
pub mod reset_texture_places;
pub mod reset_texture_tables;
pub mod restart_clock;
pub mod scale_texture_coordinates;
pub mod set_filters;
pub mod set_fine_turn_x;
pub mod set_fine_turn_y;
pub mod set_light_direction;
pub mod set_object_fade;
pub mod set_object_file_names;
pub mod set_object_light;
pub mod set_object_material;
pub mod set_place;
pub mod set_render_state;
pub mod set_texture_levels;
pub mod set_turn_x;
pub mod set_turn_y;
pub mod set_turn_z;
pub mod set_turn_z_16;
pub mod set_up_carousel;
pub mod set_up_engine;
pub mod set_up_front_end_scene;
pub mod set_up_object_memory;
pub mod set_up_textures;
pub mod set_view_matrices;
pub mod set_view_window;
pub mod sort_listed_faces;
pub mod square_matrix;
pub mod step_loading_bar;
pub mod submit_polygon;
pub mod texture_levels;
pub mod texture_needs;
pub mod texture_steps;
pub mod textures;
pub mod try_allocate_textures;
pub mod unit_vector;
pub mod update_carousel_pictures;
pub mod upload_plate;
pub mod vertex_light;
pub mod view_matrix;

pub static HOOKS: &[&dyn GameHook] = &[
    &draw_polygon::HOOK,
    &submit_polygon::HOOK,
    &draw_marked_polygon::HOOK,
    &project_point::HOOK,
    &project_polygon::HOOK,
    &scale_texture_coordinates::HOOK,
    &fog_polygon::HOOK,
    &clip_polygon_by_depth::HOOK,
    &clip_polygon_to_screen::HOOK,
    &draw_clipped_polygon::HOOK,
    &draw_object_polygon::HOOK,
    &light_dot::HOOK,
    &reflect_vector::HOOK,
    &reflection_coordinates::HOOK,
    &cached_reflection_coordinates::HOOK,
    &vertex_light::HOOK,
    &draw_object_face::HOOK,
    &draw_object_faces::HOOK,
    &draw_object::HOOK,
    &draw_shown_object::HOOK,
    &draw_scene_object::HOOK,
    &set_turn_x::HOOK,
    &set_turn_y::HOOK,
    &set_place::HOOK,
    &multiply_matrices::HOOK,
    &copy_matrix_with_place::HOOK,
    &set_light_direction::HOOK,
    &make_object_matrix::HOOK,
    &set_fine_turn_x::HOOK,
    &set_fine_turn_y::HOOK,
    &set_turn_z::HOOK,
    &set_turn_z_16::HOOK,
    &unit_vector::HOOK,
    &cross_product::HOOK,
    &square_matrix::HOOK,
    &invert_turn::HOOK,
    &set_view_matrices::HOOK,
    &set_up_front_end_scene::HOOK,
    &fill_engine_tables::HOOK,
    &next_noise::HOOK,
    &angle_between::HOOK,
    &set_view_window::HOOK,
    &prepare_menu_scene::HOOK,
    &set_up_carousel::HOOK,
    &draw_menu_object::HOOK,
    &draw_carousel_entry::HOOK,
    &draw_carousel::HOOK,
    &draw_main_menu_carousel::HOOK,
    &assign_kind_textures::HOOK,
    &map_texture_places::HOOK,
    &load_slot_texture::HOOK,
    &load_slot_textures::HOOK,
    &assign_menu_textures::HOOK,
    &update_carousel_pictures::HOOK,
    &choose_carousel_pictures::HOOK,
    &clear_depth_buffer::HOOK,
    &draw_main_menu_objects::HOOK,
    &set_object_file_names::HOOK,
    &measure_object::HOOK,
    &list_object_textures::HOOK,
    &add_object_shape::HOOK,
    &reset_objects::HOOK,
    &load_object::HOOK,
    &load_front_end_objects::HOOK,
    &reset_texture_tables::HOOK,
    &clear_slot_kinds::HOOK,
    &clear_fog::HOOK,
    &reset_texture_places::HOOK,
    &clear_slot_textures::HOOK,
    &fill_light_ramp::HOOK,
    &fill_light_scale::HOOK,
    &fill_light_tables::HOOK,
    &known_kind::HOOK,
    &known_kind_picture::HOOK,
    &raise_texture_needs::HOOK,
    &count_texture_needs::HOOK,
    &allocate_object_memory::HOOK,
    &allocate_point_records::HOOK,
    &read_kind_records::HOOK,
    &set_up_object_memory::HOOK,
    &load_wanted_textures::HOOK,
    &reload_slot_textures::HOOK,
    &fill_scene_light_tables::HOOK,
    &try_allocate_textures::HOOK,
    &build_texture_catalogue::HOOK,
    &set_up_textures::HOOK,
    &load_texture_picture::HOOK,
    &load_changed_texture_picture::HOOK,
    &step_loading_bar::HOOK,
    &known_kind_plate::HOOK,
    &set_up_engine::HOOK,
    &draw_highlighted_object::HOOK,
    &draw_car_object::HOOK,
    &set_render_state::HOOK,
    &set_filters::HOOK,
    &begin_scene::HOOK,
    &end_scene::HOOK,
    &upload_plate::HOOK,
    &load_number_plates::HOOK,
    &reload_system_textures::HOOK,
    &place_object_points::HOOK,
    &sort_listed_faces::HOOK,
    &face_faces_away::HOOK,
    &face_sort_key::HOOK,
    &list_object_face::HOOK,
    &list_object_faces::HOOK,
    &set_object_material::HOOK,
    &set_object_fade::HOOK,
    &approximate_distance::HOOK,
    &add_point_lights::HOOK,
    &add_car_light::HOOK,
    &add_car_lights::HOOK,
    &set_object_light::HOOK,
    &restart_clock::HOOK,
    &set_texture_levels::HOOK,
    &init_texture_levels::HOOK,
    &find_object_of_kind::HOOK,
];
