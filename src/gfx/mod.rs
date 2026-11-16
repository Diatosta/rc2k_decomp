//! The game's 2D graphics: the picture in memory that the menus are drawn into, the picture
//! files' readers (PCX and PNG), the fonts' glyphs, copying and scaling pictures, the pixel
//! formats' converters, the choice of screen modes and the screenshots. Menus and text are drawn
//! here pixel by pixel; the platform puts the result on the window. Its state is the `Gfx` owner
//! (`owner.rs`).

pub mod allocate_picture_buffer;
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
pub mod choices_store;
pub mod clear_front_surface;
pub mod clear_picture;
pub mod components_to_pixel;
pub mod convert_picture;
pub mod copy_picture_to_surface;
pub mod decode_png;
pub mod describe_format;
pub mod fade_colour;
pub mod fill_row_table;
pub mod fill_thirds_table;
pub mod fonts_store;
pub mod free_fonts;
pub mod free_glyph_tables;
pub mod free_picture_buffer;
pub mod glyph_builder;
pub mod glyph_store;
pub mod glyph_tables;
pub mod jade_store;
pub mod lay_out_pcx;
pub mod lay_out_png;
pub mod list_primary_modes;
pub mod load_jade;
pub mod load_picture;
pub mod load_picture_to_texture;
pub mod load_picture_with_overlay;
pub mod locked_pixels;
pub mod mark_3d_modes;
pub mod owner;
pub mod palette_pixels_to_dwords;
pub mod palette_to_dwords;
pub mod pcx;
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
pub mod read_png;
pub mod read_sheet;
pub mod read_texture_picture;
pub mod rect;
pub mod rescale_picture;
pub mod rescale_to_surface;
pub mod rgb_pixels_to_dwords;
pub mod rows;
pub mod screen_state;
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

pub mod bar_store;
pub mod blit;
pub mod clear_back_surface;
pub mod decoder;
pub mod draw2d;
pub mod formats;
pub mod front;
pub mod locked;
pub mod modes;
pub mod pcx_planes;
pub mod picture_store;
pub mod png_file;
pub mod png_header;
pub mod reports;
pub mod shots;
pub mod strips;
pub mod texts;
pub mod typefaces;
pub mod upload;
pub mod windscreen_drop_batch;

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
