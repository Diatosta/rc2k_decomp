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
pub mod components_to_4444_rows;
pub mod components_to_pixel;
pub mod convert_picture;
pub mod copy_picture_to_surface;
pub mod copy_pixel_rows;
pub mod decode_png;
pub mod describe_format;
pub mod fade_colour;
pub mod fill_row_table;
pub mod fill_thirds_table;
pub mod find_3d_cards;
pub mod fonts_store;
pub mod free_fonts;
pub mod free_glyph_tables;
pub mod free_picture_buffer;
pub mod glyph_builder;
pub mod glyph_store;
pub mod glyph_tables;
pub mod install_converters;
pub mod jade_store;
pub mod lay_out_pcx;
pub mod lay_out_png;
pub mod list_primary_modes;
pub mod list_secondary_modes;
pub mod load_jade;
pub mod load_picture;
pub mod load_picture_to_texture;
pub mod load_picture_with_overlay;
pub mod locked_pixels;
pub mod mark_3d_modes;
pub mod mmx_converters;
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
pub mod cards;
pub mod clear_back_surface;
pub mod converters;
pub mod decoder;
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

/// The test of every typed block of this module: the static's bytes are the image's.
#[cfg(test)]
pub(crate) fn first_values_are_the_games<T>(block: &'static T, start: usize, end: usize) {
    let _g = crate::store::test_lock();
    let len = end - start;
    let size = core::mem::size_of::<T>();
    assert!(
        len <= size && size <= len.next_multiple_of(4),
        "the struct's size"
    );
    // Read in place: a copy of a big block would not fit the test thread's stack.
    let mine =
        unsafe { core::slice::from_raw_parts(std::ptr::from_ref::<T>(block).cast::<u8>(), len) };
    let image = crate::moved::image_bytes(start, len);
    assert!(
        mine == &image[..],
        "the block's bytes differ from the image's"
    );
}

/// As `first_values_are_the_games`, the image's bytes first changed by `patch` (places
/// that hold a handle where the image has an address).
#[cfg(test)]
pub(crate) fn first_values_are_the_games_but<T>(
    block: &'static T,
    start: usize,
    end: usize,
    patch: impl Fn(&mut [u8]),
) {
    let _g = crate::store::test_lock();
    let len = end - start;
    let mine =
        unsafe { core::slice::from_raw_parts(std::ptr::from_ref::<T>(block).cast::<u8>(), len) };
    let mut image = crate::moved::image_bytes(start, len);
    patch(&mut image);
    assert!(
        mine == &image[..],
        "the block's bytes differ from the image's"
    );
}

/// An array of dwords with these first values (the blocks' tables).
pub(crate) const fn dwords<const N: usize>(values: [u32; N]) -> [crate::session::Dword; N] {
    let mut out = [const { crate::session::Dword::new(0) }; N];
    let mut i = 0;
    while i < N {
        out[i] = crate::session::Dword::new(values[i]);
        i += 1;
    }
    out
}

/// `dwords` for the tables only the game's thread reaches.
pub(crate) const fn plain_dwords<const N: usize>(
    values: [u32; N],
) -> [crate::session::PlainDword; N] {
    let mut out = [const { crate::session::PlainDword::new(0) }; N];
    let mut i = 0;
    while i < N {
        out[i] = crate::session::PlainDword::new(values[i]);
        i += 1;
    }
    out
}
