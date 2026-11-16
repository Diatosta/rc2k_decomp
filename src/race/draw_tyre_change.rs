//! Draws the tyre change's texture (`texture`, then copied to the device's `device_texture`) for
//! the given device, the clip's left and right being kept round it. The texture is locked
//! (`d3d::lock_texture`); unless that failed or the texture was drawn before (`drawn` 1), the
//! strings 0x129, 0x12A and 0x12B are drawn in the game fonts (`FontBlock.game_fonts`) with
//! `text::draw_string_by_id`. Then (also after a failed lock) the best time's panels are filled
//! (`gfx::fill_rect`, `texture_rect`, palette `palette`) and drawn: string 0x12E and the record's
//! time (the tyre change's `record`) in the text buffer, three times, each 0x78 left and a line
//! down (`text::draw_text_keep_regs`), and the record's name (`record_name`); then the
//! device's own panel for player 0 (`player`) or the other half, with string 0x12D and the time
//! (+0x4F4) when the device's +0x4F0 is 1, else 0x12C. The texture is unlocked
//! (`d3d::unlock_texture`), copied to the device (`d3d::copy_to_device_texture`, the platform's)
//! and `drawn` set to 1. The text is built with `strings::append_string_by_id` and `append_time`.
//! Called twice by `tyre_change_gauge`. Refused: a text longer than the text buffer, and what its
//! calls refuse.

use crate::errors::{BadInput, Refused};
use crate::input::types::InputDevice;
use crate::memory::session::PlainDword;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

fn clip(edge: &PlainDword) -> u32 {
    edge.get()
}

fn set_clip(edge: &PlainDword, v: u32) {
    edge.set(v);
}

/// The font the tyre change's texts are drawn in (`FONTS.game_fonts[2]`).
fn font(gfx: &crate::gfx::owner::Gfx) -> &crate::gfx::typefaces::FontObject {
    &gfx.fonts.font_block.game_fonts[2]
}

/// A line's height (`FONTS.game_fonts[0]`'s).
fn line(gfx: &crate::gfx::owner::Gfx) -> u32 {
    gfx.fonts.font_block.game_fonts[0].get().line_height as u32
}

/// The text buffer: string `id`, then (with `time`) a time, then a NUL.
fn build_text(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    strings: &crate::strings::owner::Strings,
    id: u32,
    time: Option<u32>,
) -> Result<(), Refused> {
    use crate::strings::append_time::{append_time, show_hours};
    let mut t = Vec::new();
    crate::strings::append_string_by_id::push_string(so, &mut t, id)?;
    if let Some(h) = time {
        t.extend(append_time(h, show_hours(strings)));
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

/// The rectangle at `texture_rect` filled with colour 0 (`gfx::fill_rect`, palette `palette`).
fn fill(
    race: &Race,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let panel = race.viewer_panel();
    for (cell, v) in panel.texture_rect.iter().zip([x, y, width, height]) {
        cell.set(v);
    }
    crate::gfx::rect::fill_rect(
        panel.texture_rect.each_ref().map(|c| c.get() as i32),
        0,
        crate::gfx::rect::Palette::Of(&race.shared.tyre_change().palette),
        gfx,
    )
}

/// A text (`text::draw_text_keep_regs` on the game's surface) in `font` at (`x`, `y`).
fn text(
    font: &crate::gfx::typefaces::FontObject,
    text: impl Into<crate::memory::place::BytePlace>,
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    { crate::race::name_plates::draw_text_in_game(font, text, x, y, gfx) }?;
    Ok(())
}

/// The text buffer drawn three times, each 0x78 to the left and a line down.
fn three_times(
    gfx: &'static crate::gfx::owner::Gfx,
    strings: &'static crate::strings::owner::Strings,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    let text_stores = &strings.text;
    gfx.fonts.font_block.game_fonts[2].update(|f| f.box_width = 0x168);
    let (mut x, mut y) = (x, y);
    text(font(gfx), &text_stores.buffer.text, x, y, gfx)?;
    for _ in 0..2 {
        x = x.wrapping_sub(0x78);
        y = y.wrapping_add(line(gfx));
        text(font(gfx), &text_stores.buffer.text, x, y, gfx)?;
    }
    Ok(())
}

/// Draws the tyre change's texture for the device `device`.
pub fn draw_tyre_change(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    race: &'static Race,
    shared: &RaceShared,
    device: &InputDevice,
    gfx: &'static crate::gfx::owner::Gfx,
    strings: &'static crate::strings::owner::Strings,
) -> Result<(), Refused> {
    let clips = (
        clip(&gfx.drawing.blit.clip.left),
        clip(&gfx.drawing.blit.clip.right),
    );
    let done = (|| -> Result<(), Refused> {
        let locked =
            crate::d3d::lock_texture::lock_texture_in(gfx, shared.tyre_change().texture.get())?;
        if locked && shared.tyre_change().drawn.get() != 1 {
            let mut y = 0u32;
            for (n, (id, font, x)) in [
                (0x129u32, &gfx.fonts.font_block.game_fonts[2], 4u32),
                (0x12A, &gfx.fonts.font_block.game_fonts[1], 0xFC),
                (0x12B, &gfx.fonts.font_block.game_fonts[0], 0x84),
            ]
            .into_iter()
            .enumerate()
            {
                match n {
                    0 => {
                        gfx.fonts.font_block.game_fonts[2].update(|f| f.box_width = 0xF8);
                        y = 0x80;
                    }
                    1 => y = 0,
                    _ => y = y.wrapping_add(line(gfx)),
                }
                crate::text::draw_string_by_id::draw_string_by_id_at(
                    so,
                    id,
                    font,
                    x as i32,
                    y as i32,
                    &mut crate::text::draw_text::GameCanvas::new(gfx),
                )?;
                if n == 0 {
                    set_clip(&gfx.drawing.blit.clip.left, 0x80);
                }
            }
        }
        set_clip(&gfx.drawing.blit.clip.left, 0x80);
        fill(race, 0x80, 0x32, 0x80, 0x4B, gfx)?;
        { build_text(so, strings, 0x12E, Some(shared.tyre_change().record.get())) }?;
        three_times(gfx, strings, 0x84, 0x32)?;
        set_clip(&gfx.drawing.blit.clip.left, 0);
        fill(race, 0, 0x99, 0x100, 0x19, gfx)?;
        gfx.fonts.font_block.game_fonts[2].update(|f| f.box_width = 0xF8);
        text(
            font(gfx),
            &race.shared.tyre_change().record_name,
            4,
            0x99,
            gfx,
        )?;
        let x = if shared.tyre_change().player.get() == 0 {
            set_clip(&gfx.drawing.blit.clip.right, 0x7F);
            0
        } else {
            set_clip(&gfx.drawing.blit.clip.left, 0x80);
            0x80
        };
        fill(race, x, 0xB2, 0x80, 0x4B, gfx)?;
        if device.stage.get() == 1 {
            let time = device.clock.get();
            { build_text(so, strings, 0x12D, Some(time)) }?;
        } else {
            { build_text(so, strings, 0x12C, None) }?;
        }
        three_times(gfx, strings, x.wrapping_add(4), 0xB2)?;
        { crate::d3d::unlock_texture::unlock_texture(gfx, shared.tyre_change().texture.get()) }?;
        // The platform's (Direct3D).
        crate::d3d::copy_to_device_texture::copy_to_device_texture(
            shared.tyre_change().texture.get(),
            shared.tyre_change().device_texture.get(),
        );
        shared.tyre_change().drawn.set(1);
        Ok(())
    })();
    set_clip(&gfx.drawing.blit.clip.left, clips.0);
    set_clip(&gfx.drawing.blit.clip.right, clips.1);
    done
}
