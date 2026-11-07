//! `text::draw_text_keep_regs` (0x402303): `draw_text` between `pusha` and `popa`.
//!
//! In:  eax = font, ebx = text, ecx = x, edx = y
//! Out: carry as `draw_text` leaves it; every register preserved
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::text::draw_text::{Canvas, draw_text};
use crate::text::font::Font;

/// Returns whether a line was too wide: the carry flag.
pub fn draw_text_keep_regs(
    font: &mut Font,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    Ok({ draw_text(font, text, x, y, canvas) }?.too_wide)
}
