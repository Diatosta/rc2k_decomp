//! The two frames of `install::install_files` (code inside it, given to `screens::run_frame`
//! as the frame callback). Each
//! starts with `call $+5`: its body runs twice, the second time
//! from the pushed return, so both buffers of the flip get the frame:
//! - `install::progress_frame`: the surface locked (`gfx::lock_surface`;
//!   failing: carry, nothing more this time round); the bar's place (`bar`: x, y,
//!   width, height) filled with colour 0 (`gfx::fill_rect`, with `g_blendedPalette`); the
//!   bar's width `g_filesCopied * 0x190 / g_filesToCopy`; the bar's glyphs
//!   (`install::blit_glyph_list`, two from `bar_glyphs_list`, with `g_installBarGlyphs`) clipped
//!   at x + that + 1 (`g_clipRight`, put back); a quarter of it as a percentage (`text`)
//!   drawn in `font` (0x51E6A4) at (x + 0xC8, y + 2); unlocked, shown;
//! - `install::copying_frame`: locked; the backdrop (`gfx::copy_picture_to_surface`);
//!   two texts (`install::draw_string_list`, `texts`); a frame (`gfx::frame_rect`,
//!   colour 0x2A, `frame`); unlocked, shown.
//!
//!
//! Every call by `at(..)`: the image is elsewhere in the standalone (a bare IDA address
//! there faulted). Callers: `screens::run_frame` from `install_files`.
//! Refuses (before anything) no files to copy, and a bar width past 32 bits (the original's
//! `div` faults).

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::install::installer::{copy_picture, draw_text, lock, show, unlock};
use crate::strings::owner::strings;

const FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING_23;

/// The bar's width, or the refusal of a count the original's `div` faults on.
fn bar_width(install: &'static crate::install::owner::Install) -> Result<u32, Refused> {
    let to_copy = install.run.files_to_copy.get();
    if to_copy == 0 {
        return Err(BadInput("no files to copy: the original divides by 0").into());
    }
    let width = install.run.files_copied.get() as u64 * 0x190 / to_copy as u64;
    u32::try_from(width)
        .map_err(|_| BadInput("a bar width past 32 bits: the original's div faults").into())
}

/// The progress frame: its carry (the last lock's or show's).
pub fn progress_frame(install: &'static crate::install::owner::Install) -> Result<bool, Refused> {
    let strings = strings();
    let drawing = &gfx().drawing;
    let width = bar_width(install)?;
    let mut carry = false;
    for _ in 0..2 {
        if !lock() {
            carry = true;
            continue;
        }
        // gfx::fill_rect: colour 0
        crate::gfx::rect::fill_rect(
            crate::gfx::rect::rect_in(&install.pages.bar),
            0,
            crate::gfx::rect::Palette::Of(crate::text::blend_font::BLENDED_PALETTE.get()),
        )?;
        let x = install.pages.bar.get_u32(0);
        let kept_clip: u32 = drawing.blit.clip.right.get();
        drawing
            .blit
            .clip
            .right
            .set(width.wrapping_add(x).wrapping_add(1));
        crate::install::blit_glyph_list::blit_glyph_list(
            &install.pages.bar_glyphs_list,
            0,
            0,
            install.run.bar_glyphs.get(),
        )?;
        drawing.blit.clip.right.set(kept_clip);
        let mut text = crate::strings::append_percent::append_percent(width >> 2);
        text.push(0);
        strings
            .text
            .buffer
            .text
            .write_at(0, &text, "the text buffer")?;
        draw_text(
            FONT,
            &text,
            x.wrapping_add(0xC8),
            install.pages.bar.get_u32(4).wrapping_add(2),
        )?;
        unlock();
        carry = !show();
    }
    Ok(carry)
}

/// The copying frame: its carry (the last lock's or show's).
pub fn copying_frame(install: &'static crate::install::owner::Install) -> Result<bool, Refused> {
    let mut carry = false;
    for _ in 0..2 {
        if !lock() {
            carry = true;
            continue;
        }
        copy_picture()?;
        crate::install::draw_string_list::draw_string_list(&install.pages.texts.each_ref(), 0, 0)?;
        // gfx::frame_rect: colour 0x2A
        crate::gfx::rect::frame_rect(
            crate::gfx::rect::rect_in(&install.pages.frame),
            0,
            0,
            0x2A,
            crate::gfx::rect::Palette::Of(crate::text::blend_font::BLENDED_PALETTE.get()),
        );
        unlock();
        carry = !show();
    }
    Ok(carry)
}
