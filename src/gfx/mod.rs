use crate::framework::GameHook;

pub mod blit_bar;
pub mod blit_bar_marker;
pub mod blit_glyph;
pub mod blit_marker_pair;
pub mod surface;

pub static HOOKS: &[&dyn GameHook] = &[
    &blit_glyph::HOOK,
    &blit_marker_pair::HOOK,
    &blit_bar::HOOK,
    &blit_bar_marker::HOOK,
];
