//! The two frames of `install::install_files`, given to `screens::run_frame` as the frame
//! callback; each runs twice so both buffers of the flip get the frame.
//! - `install::progress_frame`: the bar's place filled, its width `files_copied * 0x190 /
//!   files_to_copy`, the bar's glyphs clipped at that width, a percentage drawn on it;
//! - `install::copying_frame`: the backdrop, two texts and a frame.
//!
//! Refuses (before anything) no files to copy, and a bar width past 32 bits.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{copy_picture, draw_text, lock, show, unlock};

const FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING_23;

/// The bar's width, or the refusal of a count it cannot divide by.
fn bar_width(install: &crate::install::owner::Install) -> Result<u32, Refused> {
    let to_copy = install.run.files_to_copy.get();
    if to_copy == 0 {
        return Err(BadInput("no files to copy: the original divides by 0").into());
    }
    let width = install.run.files_copied.get() as u64 * 0x190 / to_copy as u64;
    u32::try_from(width)
        .map_err(|_| BadInput("a bar width past 32 bits: the original's div faults").into())
}

/// The progress frame; true when the last lock or show failed.
pub fn progress_frame(o: crate::owners::Owners<'static>) -> Result<bool, Refused> {
    let (install, strings, gfx, system) = (o.install, o.strings, o.gfx, o.system);
    let drawing = &gfx.drawing;
    let width = bar_width(install)?;
    let mut carry = false;
    for _ in 0..2 {
        if !lock(gfx, system) {
            carry = true;
            continue;
        }
        // gfx::fill_rect: colour 0
        crate::gfx::rect::fill_rect(
            crate::gfx::rect::rect_in(&install.pages.bar),
            0,
            crate::gfx::rect::Palette::Of(&gfx.fonts.blended_palette),
            gfx,
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
            gfx,
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
            gfx,
        )?;
        unlock(gfx);
        carry = !show(o);
    }
    Ok(carry)
}

/// The copying frame; true when the last lock or show failed.
pub fn copying_frame(o: crate::owners::Owners<'static>) -> Result<bool, Refused> {
    let (install, gfx, system) = (o.install, o.gfx, o.system);
    let mut carry = false;
    for _ in 0..2 {
        if !lock(gfx, system) {
            carry = true;
            continue;
        }
        copy_picture(gfx, system)?;
        crate::install::draw_string_list::draw_string_list(
            crate::strings::append_string_by_id::StringOwners::of(o),
            &install.pages.texts.each_ref(),
            0,
            0,
            gfx,
        )?;
        // gfx::frame_rect: colour 0x2A
        crate::gfx::rect::frame_rect(
            crate::gfx::rect::rect_in(&install.pages.frame),
            0,
            0,
            0x2A,
            crate::gfx::rect::Palette::Of(&gfx.fonts.blended_palette),
            gfx,
        );
        unlock(gfx);
        carry = !show(o);
    }
    Ok(carry)
}
