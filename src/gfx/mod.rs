use crate::framework::GameHook;

pub mod blend_palette;
pub mod blit_bar;
pub mod blit_bar_marker;
pub mod blit_glyph;
pub mod blit_marker_pair;
pub mod build_fonts;
pub mod build_game_fonts;
pub mod build_glyphs;
pub mod build_menu_sprites;
pub mod clear_picture;
pub mod components_to_4444_rows;
pub mod components_to_pixel;
pub mod convert_picture;
pub mod copy_picture_to_surface;
pub mod copy_pixel_rows;
pub mod decode_png;
pub mod fill_row_table;
pub mod free_fonts;
pub mod glyph_builder;
pub mod glyph_tables;
pub mod harness;
pub mod install_converters;
pub mod lay_out_pcx;
pub mod lay_out_png;
pub mod load_picture;
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
pub mod read_pcx;
pub mod read_pcx_24bit;
pub mod read_pcx_8bit;
pub mod read_picture_file;
pub mod read_png;
pub mod read_sheet;
pub mod rescale_picture;
pub mod rgb_pixels_to_dwords;
pub mod set_clip;
pub mod surface;

pub static HOOKS: &[&dyn GameHook] = &[
    &blit_glyph::HOOK,
    &blit_marker_pair::HOOK,
    &blit_bar::HOOK,
    &blit_bar_marker::HOOK,
    &pixel_to_components::HOOK,
    &components_to_pixel::HOOK,
    &blend_palette::HOOK,
    &clear_picture::HOOK,
    &convert_picture::HOOK,
    &copy_picture_to_surface::HOOK,
    &copy_pixel_rows::HOOK,
    &components_to_4444_rows::HOOK,
    &install_converters::HOOK,
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
    &set_clip::HOOK,
    &picture_as_surface::HOOK,
    &load_picture::HOOK,
    &build_glyphs::HOOK,
    &build_fonts::HOOK,
    &free_fonts::HOOK,
    &build_menu_sprites::HOOK,
    &build_game_fonts::HOOK,
];
