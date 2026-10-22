use crate::framework::GameHook;

pub mod allocate_picture_buffer;
pub mod alpha_from_kept;
pub mod blend_palette;
pub mod blit_bar;
pub mod blit_bar_marker;
pub mod blit_components;
pub mod blit_glyph;
pub mod blit_marker_pair;
pub mod build_fonts;
pub mod build_game_fonts;
pub mod build_glyphs;
pub mod build_menu_sprites;
pub mod clear_front_surface;
pub mod clear_picture;
pub mod components_to_4444_rows;
pub mod components_to_pixel;
pub mod convert_picture;
pub mod copy_picture_to_surface;
pub mod copy_pixel_rows;
pub mod decode_png;
pub mod describe_format;
pub mod draw_hline;
pub mod draw_vline;
pub mod fade_colour;
pub mod fill_rect;
pub mod fill_row_table;
pub mod fill_thirds_table;
pub mod find_3d_cards;
pub mod frame_rect;
pub mod free_fonts;
pub mod free_glyph_tables;
pub mod free_picture_buffer;
pub mod glyph_builder;
pub mod glyph_tables;
pub mod grade_texture;
pub mod halve_texture;
pub mod harness;
pub mod install_converters;
pub mod key_texture_colours;
pub mod lay_out_pcx;
pub mod lay_out_png;
pub mod least_colour;
pub mod list_primary_modes;
pub mod list_secondary_modes;
pub mod load_jade;
pub mod load_picture;
pub mod load_picture_to_texture;
pub mod load_picture_with_overlay;
pub mod mark_3d_modes;
pub mod mmx_converters;
pub mod palette_pixels_to_dwords;
pub mod palette_to_dwords;
pub mod pcx;
pub mod pcx_reader;
pub mod picture;
pub mod picture_as_surface;
pub mod pixel_format;
pub mod pixel_to_components;
pub mod pixels_to_dwords;
pub mod png;
pub mod put_surface_pixel;
pub mod read_pcx;
pub mod read_pcx_24bit;
pub mod read_pcx_8bit;
pub mod read_picture_file;
pub mod read_pixel;
pub mod read_png;
pub mod read_sheet;
pub mod read_texture_picture;
pub mod rect;
pub mod rescale_picture;
pub mod rescale_to_surface;
pub mod rgb_pixels_to_dwords;
pub mod set_clip;
pub mod set_component_format;
pub mod show_picture_file;
pub mod surface;
pub mod texture_pixels;
pub mod texture_upload;
pub mod upload_texture_picture;

pub mod keep_race_screen_mode;
pub mod screen_mode_choices;
pub mod take_screenshot;
pub mod try_screen_mode;
pub mod write_screen_pcx;

pub mod clear_back_surface;

pub static HOOKS: &[&dyn GameHook] = &[
    &blit_glyph::HOOK,
    &fill_rect::HOOK,
    &frame_rect::HOOK,
    &draw_hline::HOOK,
    &draw_vline::HOOK,
    &blit_marker_pair::HOOK,
    &blit_bar::HOOK,
    &blit_bar_marker::HOOK,
    &pixel_to_components::HOOK,
    &read_pixel::HOOK,
    &components_to_pixel::HOOK,
    &blend_palette::HOOK,
    &describe_format::HOOK,
    &clear_picture::HOOK,
    &convert_picture::HOOK,
    &copy_picture_to_surface::HOOK,
    &copy_pixel_rows::HOOK,
    &components_to_4444_rows::HOOK,
    &palette_to_dwords::HOOK,
    &lay_out_pcx::HOOK,
    &read_pcx_8bit::HOOK,
    &read_pcx_24bit::HOOK,
    // Before `read_pcx`: its private copy takes that function along, and has to take it
    // as it is in the image, not with the jump a hook puts at its start.
    &read_picture_file::HOOK,
    &read_pcx::HOOK,
    &decode_png::HOOK,
    &lay_out_png::HOOK,
    &palette_pixels_to_dwords::HOOK,
    &rgb_pixels_to_dwords::HOOK,
    &read_png::HOOK,
    &fill_row_table::HOOK,
    &rescale_picture::HOOK,
    &rescale_to_surface::HOOK,
    &show_picture_file::HOOK,
    &set_clip::HOOK,
    &picture_as_surface::HOOK,
    &load_picture::HOOK,
    &build_glyphs::HOOK,
    &build_fonts::HOOK,
    &free_fonts::HOOK,
    &build_menu_sprites::HOOK,
    &build_game_fonts::HOOK,
    &least_colour::HOOK,
    &alpha_from_kept::HOOK,
    &key_texture_colours::HOOK,
    &grade_texture::HOOK,
    &halve_texture::HOOK,
    &read_texture_picture::HOOK,
    &blit_components::HOOK,
    &upload_texture_picture::HOOK,
    &load_picture_to_texture::HOOK,
    &load_picture_with_overlay::HOOK,
    &put_surface_pixel::HOOK,
    &fade_colour::HOOK,
    &mark_3d_modes::HOOK,
    &free_picture_buffer::HOOK,
    &write_screen_pcx::HOOK,
    &take_screenshot::HOOK,
    &keep_race_screen_mode::HOOK,
    &screen_mode_choices::HOOK,
    &try_screen_mode::HOOK,
    &clear_back_surface::HOOK,
    &free_glyph_tables::HOOK,
];
