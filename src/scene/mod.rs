//! The 3D scene: the game's own code above the Direct3D layer (`d3d`).

use crate::framework::GameHook;

pub mod add_car_light;
pub mod add_car_lights;
pub mod add_object_shape;
pub mod add_point_lights;
pub mod approximate_distance;
pub mod assign_kind_textures;
pub mod assign_menu_textures;
pub mod cached_reflection_coordinates;
pub mod carousel_pictures;
pub mod choose_carousel_pictures;
pub mod clear_depth_buffer;
pub mod clip;
pub mod clip_asm;
pub mod clip_polygon_by_depth;
pub mod clip_polygon_to_screen;
pub mod clip_test;
pub mod copy_matrix_with_place;
pub mod draw_carousel;
pub mod draw_carousel_entry;
pub mod draw_clipped_polygon;
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
pub mod face_faces_away;
pub mod face_sort_key;
pub mod fog_polygon;
pub mod light_dot;
pub mod list_object_face;
pub mod list_object_faces;
pub mod list_object_textures;
pub mod load_front_end_objects;
pub mod load_object;
pub mod load_slot_texture;
pub mod load_slot_textures;
pub mod make_object_matrix;
pub mod map_texture_places;
pub mod matrix;
pub mod measure_object;
pub mod multiply_matrices;
pub mod objects;
pub mod place_object_points;
pub mod polygon;
pub mod project_point;
pub mod project_polygon;
pub mod reflect_vector;
pub mod reflection_coordinates;
pub mod reset_objects;
pub mod scale_texture_coordinates;
pub mod set_light_direction;
pub mod set_object_fade;
pub mod set_object_file_names;
pub mod set_object_light;
pub mod set_object_material;
pub mod set_place;
pub mod set_turn_x;
pub mod set_turn_y;
pub mod sort_listed_faces;
pub mod submit_polygon;
pub mod textures;
pub mod update_carousel_pictures;
pub mod vertex_light;

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
];
