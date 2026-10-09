use crate::framework::GameHook;

pub mod draw_string_by_id;
pub mod draw_text;
pub mod draw_text_keep_regs;
pub mod font;
pub mod glyph_kerning;
pub mod reset_line_state;

pub static HOOKS: &[&dyn GameHook] = &[
    &reset_line_state::HOOK,
    &glyph_kerning::HOOK,
    &draw_text::HOOK,
    &draw_text_keep_regs::HOOK,
    &draw_string_by_id::HOOK,
];
