use crate::framework::GameHook;

pub mod glyph_kerning;
pub mod reset_line_state;

pub static HOOKS: &[&dyn GameHook] = &[&reset_line_state::HOOK, &glyph_kerning::HOOK];
