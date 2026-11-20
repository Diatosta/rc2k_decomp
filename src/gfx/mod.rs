//! The game's 2D graphics: the picture in memory that the menus are drawn into, the picture
//! files' readers (PCX and PNG), the fonts' glyphs, copying and scaling pictures, the pixel
//! formats' converters, the choice of screen modes and the screenshots. Menus and text are drawn
//! here pixel by pixel; the platform puts the result on the window. Its state is the `Gfx` owner
//! (`owner.rs`).

pub(crate) mod allocate_picture_buffer;
pub(crate) mod blend_palette;
pub(crate) mod blit_bar;
pub(crate) mod blit_bar_marker;
pub(crate) mod blit_components;
pub(crate) mod blit_glyph;
pub(crate) mod blit_marker_pair;
pub(crate) mod build_fonts;
pub(crate) mod build_game_fonts;
mod build_glyphs;
pub(crate) mod build_menu_sprites;
pub(crate) mod button_hints;
pub(crate) mod button_pictures;
mod choices_store;
pub(crate) mod clear_front_surface;
mod clear_picture;
pub(crate) mod components_to_pixel;
pub(crate) mod convert_picture;
pub(crate) mod copy_picture_to_surface;
mod decode_png;
pub(crate) mod describe_format;
pub(crate) mod fade_colour;
mod fill_row_table;
pub(crate) mod fill_thirds_table;
mod fonts_store;
pub(crate) mod free_fonts;
pub(crate) mod free_glyph_tables;
pub(crate) mod free_picture_buffer;
mod glyph_builder;
pub(crate) mod glyph_store;
mod glyph_tables;
mod jade_store;
mod lay_out_pcx;
mod lay_out_png;
pub(crate) mod list_primary_modes;
pub(crate) mod load_jade;
pub(crate) mod load_picture;
pub(crate) mod load_picture_to_texture;
pub(crate) mod load_picture_with_overlay;
pub(crate) mod locked_pixels;
mod mark_3d_modes;
pub(crate) mod notice_text;
pub(crate) mod owner;
mod palette_pixels_to_dwords;
pub(crate) mod palette_to_dwords;
mod pcx;
pub(crate) mod picture;
pub(crate) mod picture_as_surface;
pub(crate) mod pixel_format;
pub(crate) mod pixel_to_components;
mod pixels_to_dwords;
mod png;
pub(crate) mod put_surface_pixel;
mod read_pcx;
mod read_pcx_24bit;
mod read_pcx_8bit;
pub(crate) mod read_picture_file;
mod read_png;
pub(crate) mod read_sheet;
pub(crate) mod read_texture_picture;
pub(crate) mod rect;
mod rescale_picture;
mod rescale_to_surface;
mod rgb_pixels_to_dwords;
pub(crate) mod rows;
pub(crate) mod scaled_region;
pub(crate) mod screen_state;
pub(crate) mod set_clip;
pub(crate) mod set_component_format;
pub(crate) mod show_picture_file;
pub(crate) mod surface;
pub(crate) mod texture_pixels;
pub(crate) mod texture_upload;
pub(crate) mod ui_scale;
mod upload_texture_picture;

mod keep_race_screen_mode;
pub(crate) mod screen_mode_choices;
pub(crate) mod take_screenshot;
mod try_screen_mode;
pub(crate) mod write_screen_pcx;

mod bar_store;
mod blit;
pub(crate) mod clear_back_surface;
mod decoder;
pub(crate) mod draw2d;
mod formats;
mod front;
mod locked;
pub(crate) mod modes;
pub(crate) mod monitor_sizes;
mod pcx_planes;
mod picture_store;
mod png_file;
mod png_header;
mod reports;
pub(crate) mod shots;
pub(crate) mod strips;
pub(crate) mod texts;
pub(crate) mod typefaces;
pub(crate) mod upload;
pub(crate) mod windscreen_drop_batch;
pub(crate) mod with_locked_surface;

/// An array of dwords with these first values (the blocks' tables).
pub(crate) const fn dwords<const N: usize>(values: [u32; N]) -> [crate::memory::session::Dword; N] {
    let mut out = [const { crate::memory::session::Dword::new(0) }; N];
    let mut i = 0;
    while i < N {
        out[i] = crate::memory::session::Dword::new(values[i]);
        i += 1;
    }
    out
}

/// `dwords` for the tables only the game's thread reaches.
pub(crate) const fn plain_dwords<const N: usize>(
    values: [u32; N],
) -> [crate::memory::session::PlainDword; N] {
    let mut out = [const { crate::memory::session::PlainDword::new(0) }; N];
    let mut i = 0;
    while i < N {
        out[i] = crate::memory::session::PlainDword::new(values[i]);
        i += 1;
    }
    out
}
