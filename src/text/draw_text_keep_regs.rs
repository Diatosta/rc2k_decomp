//! `draw_text` for callers that need only whether a line was too wide.

use crate::errors::Refused;
use crate::text::draw_text::{Canvas, draw_text};
use crate::text::font::Font;

/// Returns whether a line was too wide.
pub fn draw_text_keep_regs(
    font: &mut Font,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    Ok({ draw_text(font, text, x, y, canvas) }?.too_wide)
}
