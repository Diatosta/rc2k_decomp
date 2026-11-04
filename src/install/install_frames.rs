//! The two frames of `install::install_files` (code inside it, given to `screens::run_frame`
//! in eax; ported 2026-10-20 for the standalone, which has no game code to run them). Each
//! starts with `call $+5`: its body (`pusha` .. `popa; retn`) runs twice, the second time
//! from the pushed return, so both buffers of the flip get the frame:
//! - `install::progress_frame` (0x413295): the surface locked (`gfx::lock_surface`;
//!   failing: carry, nothing more this time round); the bar's place (`dword_51D4A3`: x, y,
//!   width, height) filled with colour 0 (`gfx::fill_rect`, esi `g_blendedPalette`); the
//!   bar's width `g_filesCopied * 0x190 / g_filesToCopy`; the bar's glyphs
//!   (`install::blit_glyph_list`, two at `unk_51D483`, esi `g_installBarGlyphs`) clipped
//!   at x + that + 1 (`g_clipRight`, put back); a quarter of it as a percentage (`text`)
//!   drawn in `font` (0x51E6A4) at (x + 0xC8, y + 2); unlocked, shown;
//! - `install::copying_frame` (0x413340): locked; the backdrop (`gfx::copy_picture_to_surface`);
//!   two texts (`install::draw_string_list`, `unk_51D433`); a frame (`gfx::frame_rect`,
//!   colour 0x2A, `unk_51D4B3`); unlocked, shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of the last call (the show, or
//!      the lock that failed)
//!
//! Every call by `at(..)`: the image is elsewhere in the standalone (a bare IDA address
//! there faulted, 2026-10-20). Callers: `screens::run_frame` from `install_files` (0x41318D, 0x413248, 0x413255).
//! Refuses (before anything) no files to copy, and a bar width past 32 bits (the original's
//! `div` faults).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit::BLIT;
use crate::install::installer::{copy_picture, draw_text, lock, show, unlock};
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_RUN;
use crate::strings::text_buffer::TEXT_BUFFER;

const FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING_23;

/// The bar's width, or the refusal of a count the original's `div` faults on.
fn bar_width() -> Result<u32, Refused> {
    let to_copy = INSTALLER_RUN.files_to_copy.get();
    if to_copy == 0 {
        return Err(BadInput("no files to copy: the original divides by 0").into());
    }
    let width = INSTALLER_RUN.files_copied.get() as u64 * 0x190 / to_copy as u64;
    u32::try_from(width)
        .map_err(|_| BadInput("a bar width past 32 bits: the original's div faults").into())
}

/// `install::progress_frame`'s Rust version: its carry (the last lock's or show's).
pub fn progress_frame() -> Result<bool, Refused> {
    let width = bar_width()?;
    let mut carry = false;
    for _ in 0..2 {
        if !lock() {
            carry = true;
            continue;
        }
        // gfx::fill_rect: colour 0
        crate::gfx::rect::fill_rect(
            crate::gfx::rect::rect_in(&INSTALLER_PAGES.bar),
            0,
            crate::gfx::rect::Palette::Of(crate::pointers::GamePtr::to(
                &crate::text::blend_font::BLENDED_PALETTE,
            )),
        )?;
        let x = INSTALLER_PAGES.bar.get_u32(0);
        let kept_clip: u32 = BLIT.clip.right.get();
        BLIT.clip.right.set(width.wrapping_add(x).wrapping_add(1));
        crate::install::blit_glyph_list::blit_glyph_list(
            2,
            &INSTALLER_PAGES.bar_glyphs_list,
            0,
            0,
            INSTALLER_RUN.bar_glyphs.get(),
        )?;
        BLIT.clip.right.set(kept_clip);
        let mut text = crate::strings::append_percent::append_percent(width >> 2);
        text.push(0);
        TEXT_BUFFER.text.write_at(0, &text, "the text buffer")?;
        draw_text(
            FONT,
            &TEXT_BUFFER.text,
            x.wrapping_add(0xC8),
            INSTALLER_PAGES.bar.get_u32(4).wrapping_add(2),
        )?;
        unlock();
        carry = !show();
    }
    Ok(carry)
}

/// `install::copying_frame`'s Rust version: its carry (the last lock's or show's).
pub fn copying_frame() -> Result<bool, Refused> {
    let mut carry = false;
    for _ in 0..2 {
        if !lock() {
            carry = true;
            continue;
        }
        copy_picture()?;
        crate::install::draw_string_list::draw_string_list(2, &INSTALLER_PAGES.texts, 0, 0)?;
        // gfx::frame_rect: colour 0x2A
        crate::gfx::rect::frame_rect(
            crate::gfx::rect::rect_in(&INSTALLER_PAGES.frame),
            0,
            0,
            0x2A,
            crate::gfx::rect::Palette::Of(crate::pointers::GamePtr::to(
                &crate::text::blend_font::BLENDED_PALETTE,
            )),
        );
        unlock();
        carry = !show();
    }
    Ok(carry)
}
