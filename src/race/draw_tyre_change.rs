//! `race::draw_tyre_change` (0x41C38C): the tyre change's texture (`dword_5BEBE0`, then
//! copied to the device's `dword_5BEBE4`), for the device at esi. ebp = esi; the clip's
//! left and right kept round it. Its lock (`d3d::lock_texture`); unless that failed or the
//! texture was drawn before (`dword_5BEBE8` 1), strings 0x129 (font `stru_51E304`, its
//! +0x1C 0xF8, at 4, 0x80), then with the clip's left 0x80 0x12A (`unk_51E2E4`, at 0xFC, 0)
//! and 0x12B (`unk_51E2C4`, at 0x84, a line down: `dword_51E2CC`), each by
//! `text::draw_string_by_id` inside `pusha` / `popa`. Then (also after a failed lock:
//! section 16) the clip's left 0x80; the rectangle (0x80, 0x32, 0x80, 0x4B) filled with
//! colour 0 (`gfx::fill_rect`, `dword_5C2A1C`, palette `unk_5BEC2C`); string 0x12E and the
//! record's time (`g_tyreChangeRecord`) in the text buffer, drawn three times (font +0x1C
//! 0x168, at 0x84, 0x32; each 0x78 left and a line down: `text::draw_text_keep_regs`);
//! the clip's left 0; (0, 0x99, 0x100, 0x19) filled; the record's name
//! (`g_tyreChangeRecordName`, +0x1C 0xF8, at 4, 0x99); for player 0 (`dword_5BEBEC`) the
//! clip's right 0x7F and x 0, else its left 0x80 and x 0x80; (x, 0xB2, 0x80, 0x4B) filled;
//! with the device's +0x4F0 1 string 0x12D and the time (+0x4F4), else 0x12C, drawn three
//! times (+0x1C 0x168, at x + 4, 0xB2, as before); the unlock (`d3d::unlock_texture`), the
//! copy to the device (`d3d::copy_to_device_texture` 0x41A8F9: the platform's, called in
//! the game), `dword_5BEBE8` 1.
//!
//! In:  esi the device
//! Out: ebp = esi; the rest kept; the flags of the copy (no caller reads them)
//!
//! Callers: `race::tyre_change_gauge` (twice), which calls the Rust version directly. The text
//! is built with the Rust versions of `strings::append_string_by_id` and `append_time`. Refuses
//! a text longer than the text buffer, and what its calls refuse.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::input::types::InputDevice;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::session::Dword;
use crate::strings::owner::strings;

fn clip(edge: &Dword) -> u32 {
    edge.get()
}

fn set_clip(edge: &Dword, v: u32) {
    edge.set(v);
}

/// The font the tyre change's texts are drawn in (`FONTS.game_fonts[2]`).
fn font(gfx: &crate::gfx::owner::Gfx) -> &crate::gfx::typefaces::FontObject {
    &gfx.fonts.menu_fonts.game_fonts[2]
}

/// A line's height (`FONTS.game_fonts[0]`'s).
fn line(gfx: &crate::gfx::owner::Gfx) -> u32 {
    gfx.fonts.menu_fonts.game_fonts[0].get().line_height as u32
}

/// The text buffer: string `id`, then (with `time`) a time, then a NUL.
fn build_text(
    strings: &crate::strings::owner::Strings,
    id: u32,
    time: Option<u32>,
) -> Result<(), Refused> {
    use crate::strings::append_time::{append_time, show_hours};
    let mut t = Vec::new();
    crate::strings::append_string_by_id::push_string(&mut t, id)?;
    if let Some(h) = time {
        t.extend(append_time(h, show_hours()));
    }
    t.push(0);
    if t.len() > crate::strings::text_from_ids::TEXT_LEN {
        return Err(BadInput("a tyre change text longer than the text buffer").into());
    }
    for (n, &b) in t.iter().enumerate() {
        strings.text.buffer.text.set(n, b);
    }
    Ok(())
}

/// The rectangle at `dword_5C2A1C` filled with colour 0 (`gfx::fill_rect`, palette
/// `unk_5BEC2C`).
fn fill(shared: &Shared, x: u32, y: u32, width: u32, height: u32) -> Result<(), Refused> {
    for (cell, v) in VIEWER_PANEL.texture_rect.iter().zip([x, y, width, height]) {
        cell.set(v);
    }
    crate::gfx::rect::fill_rect(
        crate::gfx::rect::rect_of(&VIEWER_PANEL.texture_rect),
        0,
        crate::gfx::rect::Palette::Of(&shared.tyre_change.palette),
    )
}

/// A text (`text::draw_text_keep_regs` on the game's surface) in `font` at (`x`, `y`).
fn text(
    font: &crate::gfx::typefaces::FontObject,
    text: impl Into<crate::place::Place>,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    { crate::race::name_plates::draw_text_in_game(font, text, x, y) }?;
    Ok(())
}

/// The text buffer drawn three times, each 0x78 to the left and a line down.
fn three_times(
    gfx: &crate::gfx::owner::Gfx,
    strings: &'static crate::strings::owner::Strings,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    let text_stores = &strings.text;
    gfx.fonts.menu_fonts.game_fonts[2].update(|f| f.box_width = 0x168);
    let (mut x, mut y) = (x, y);
    text(font(gfx), &text_stores.buffer.text, x, y)?;
    for _ in 0..2 {
        x = x.wrapping_sub(0x78);
        y = y.wrapping_add(line(gfx));
        text(font(gfx), &text_stores.buffer.text, x, y)?;
    }
    Ok(())
}

/// The Rust version, for the device `device` (esi). The registers it leaves (ebp = esi,
/// the copy's flags) are read by no caller.
pub fn draw_tyre_change(
    race: &'static Race,
    shared: &Shared,
    device: &InputDevice,
) -> Result<(), Refused> {
    let gfx = gfx();
    let strings = strings();
    let clips = (
        clip(&gfx.drawing.blit.clip.left),
        clip(&gfx.drawing.blit.clip.right),
    );
    let done = (|| -> Result<(), Refused> {
        let failed =
            { crate::d3d::lock_texture::lock_texture(shared.tyre_change.texture.get()) }?.carry;
        if !failed && shared.tyre_change.drawn.get() != 1 {
            let mut y = 0u32;
            for (n, (id, font, x)) in [
                (0x129u32, &gfx.fonts.menu_fonts.game_fonts[2], 4u32),
                (0x12A, &gfx.fonts.menu_fonts.game_fonts[1], 0xFC),
                (0x12B, &gfx.fonts.menu_fonts.game_fonts[0], 0x84),
            ]
            .into_iter()
            .enumerate()
            {
                match n {
                    0 => {
                        gfx.fonts.menu_fonts.game_fonts[2].update(|f| f.box_width = 0xF8);
                        y = 0x80;
                    }
                    1 => y = 0,
                    _ => y = y.wrapping_add(line(gfx)),
                }
                crate::text::draw_string_by_id::draw_string_by_id_at(
                    id,
                    font,
                    x as i32,
                    y as i32,
                    &mut crate::text::draw_text::GameCanvas,
                )?;
                if n == 0 {
                    set_clip(&gfx.drawing.blit.clip.left, 0x80);
                }
            }
        }
        set_clip(&gfx.drawing.blit.clip.left, 0x80);
        fill(shared, 0x80, 0x32, 0x80, 0x4B)?;
        { build_text(strings, 0x12E, Some(shared.tyre_change.record.get())) }?;
        three_times(gfx, strings, 0x84, 0x32)?;
        set_clip(&gfx.drawing.blit.clip.left, 0);
        fill(shared, 0, 0x99, 0x100, 0x19)?;
        gfx.fonts.menu_fonts.game_fonts[2].update(|f| f.box_width = 0xF8);
        text(font(gfx), &race.shared.tyre_change.record_name, 4, 0x99)?;
        let x = if shared.tyre_change.player.get() == 0 {
            set_clip(&gfx.drawing.blit.clip.right, 0x7F);
            0
        } else {
            set_clip(&gfx.drawing.blit.clip.left, 0x80);
            0x80
        };
        fill(shared, x, 0xB2, 0x80, 0x4B)?;
        if device.stage.get() == 1 {
            let time = device.clock.get();
            { build_text(strings, 0x12D, Some(time)) }?;
        } else {
            { build_text(strings, 0x12C, None) }?;
        }
        three_times(gfx, strings, x.wrapping_add(4), 0xB2)?;
        { crate::d3d::unlock_texture::unlock_texture(shared.tyre_change.texture.get()) }?;
        // The platform's (Direct3D).
        crate::d3d::copy_to_device_texture::copy_to_device_texture(
            shared.tyre_change.texture.get(),
            shared.tyre_change.device_texture.get(),
        );
        shared.tyre_change.drawn.set(1);
        Ok(())
    })();
    set_clip(&gfx.drawing.blit.clip.left, clips.0);
    set_clip(&gfx.drawing.blit.clip.right, clips.1);
    done
}
