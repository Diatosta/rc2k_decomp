use crate::framework::GameHook;

pub mod blit_glyph;
pub mod surface;

pub static HOOKS: &[&dyn GameHook] = &[&blit_glyph::HOOK];
