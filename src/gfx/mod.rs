use crate::framework::GameHook;

pub mod blend_palette;
pub mod blit_bar;
pub mod blit_bar_marker;
pub mod blit_glyph;
pub mod blit_marker_pair;
pub mod components_to_pixel;
pub mod pixel_format;
pub mod pixel_to_components;
pub mod surface;

pub static HOOKS: &[&dyn GameHook] = &[
    &blit_glyph::HOOK,
    &blit_marker_pair::HOOK,
    &blit_bar::HOOK,
    &blit_bar_marker::HOOK,
    &pixel_to_components::HOOK,
    &components_to_pixel::HOOK,
    &blend_palette::HOOK,
];
