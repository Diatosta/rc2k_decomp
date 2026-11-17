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

/// The text buffer: string `id`, then (with `time`) a time, then a NUL; the text without the NUL.
fn build_text(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    strings: &crate::strings::owner::Strings,
    id: u32,
    time: Option<u32>,
) -> Result<Vec<u8>, Refused> {
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
    t.pop();
    Ok(t)
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

/// A button's picture drawn after a text.
#[derive(Clone, Copy)]
struct Picture {
    family: rc2k_core::input::icons::Family,
    icon: rc2k_core::input::icons::Icon,
}

/// The picture of the button that ends the gauge for the player's controller, when the player
/// has a controller with pictures and the pictures are on (the advanced page's switch for the
/// pictures in questions); none for the keyboard.
fn stop_picture(o: crate::owners::Owners<'static>, player: u32) -> Option<Picture> {
    if !crate::drive::quit_question::pictures_on() {
        return None;
    }
    let (pictures, bindings) = crate::input::profiles::race_pictures(o.input, player as usize)?;
    let digital = rc2k_core::input::profile::tyre_change_button(&bindings)?;
    let icon = pictures.digital(digital)?;
    crate::gfx::button_pictures::has(pictures.family, icon).then_some(Picture {
        family: pictures.family,
        icon,
    })
}

/// A text's width in the tyre change's font, measured without drawing.
fn text_width(gfx: &'static crate::gfx::owner::Gfx, text: &[u8]) -> i32 {
    let mut copy = font(gfx).get();
    copy.mode = rc2k_core::strings::text::MODE_MEASURE;
    let mut nul = text.to_vec();
    nul.push(0);
    let mut canvas = crate::text::draw_text::GameCanvas::new(gfx);
    crate::text::measure_text::measure_text(&mut copy, &nul[..], 0, &mut canvas)
        .map_or(0, |m| m.width as i32)
}

/// Picture `picture` drawn into the texture's pixels (32 bits, the alpha in the top byte) with its
/// top left at (`x`, `y`), `size` pixels high and wide, clipped as the text is: white over what
/// is there, the picture's alpha added to the pixel's own.
fn put_picture(
    gfx: &'static crate::gfx::owner::Gfx,
    picture: Picture,
    x: i32,
    y: i32,
    size: i32,
) -> Option<()> {
    let alpha = crate::gfx::button_pictures::alpha_of(picture.family, picture.icon, size as usize)?;
    if crate::gfx::surface::surface(gfx).bits_per_pixel != 32 {
        return None;
    }
    let rows = crate::gfx::rows::Rows::surface(gfx).ok()?;
    let clip = crate::gfx::surface::clip(gfx);
    let (left, right) = (x.max(clip.left), (x + size - 1).min(clip.right));
    if left > right {
        return Some(());
    }
    for row in 0..size {
        let py = y + row;
        if py < clip.top || py > clip.bottom {
            continue;
        }
        let coverage = &alpha[row as usize * size as usize..][..size as usize];
        let at = (left * 4) as u32;
        let mut pixels = rows
            .read(
                py as u32,
                at,
                4 * (right - left + 1) as usize,
                "a picture's row",
            )
            .ok()?;
        for (pixel, &a) in pixels
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(&coverage[(left - x) as usize..=(right - x) as usize])
        {
            if a == 0 {
                continue;
            }
            let was = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
            let (a, under) = (u32::from(a), was >> 24);
            // Alpha over alpha; the colour is white where the picture covers, the pixel's own
            // where it does not.
            let seen = a + under * (255 - a) / 255;
            let colour = if under == 0 {
                0x00FF_FFFF
            } else {
                crate::gfx::button_pictures::blend_32(was & 0x00FF_FFFF, a as u8)
            };
            pixel.copy_from_slice(&(colour | seen << 24).to_le_bytes());
        }
        rows.write(py as u32, at, &pixels, "a picture's row").ok()?;
    }
    Some(())
}

/// The prompt (`prompt`, in the text buffer) drawn three times as `three_times` does, with the
/// button's picture after it, the two centred together. The picture is as tall as the prompt's
/// capitals and centred on them.
fn three_times_with_picture(
    o: crate::owners::Owners<'static>,
    prompt: &[u8],
    picture: Picture,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    let gfx = o.gfx;
    let strings = o.strings;
    let canvas = crate::text::draw_text::GameCanvas::new(gfx);
    let font_now = font(gfx).get();
    let (ink_top, size) =
        crate::gfx::button_hints::capital_rows(&font_now, prompt, &canvas).unwrap_or((0, 25));
    let gap = (text_width(gfx, b" ") / 2).max(2) * 2;
    // The three strips hold a box of 0x168: a prompt and picture wider than that have their
    // letters drawn closer together by the least whole pixels that make them fit.
    let spacing = font_now.spacing;
    let mut closer = 0;
    while closer < 3 && text_width(gfx, prompt) + gap + size > 0x168 {
        closer += 1;
        gfx.fonts.font_block.game_fonts[2].update(|f| f.spacing = spacing - closer);
    }
    let width = text_width(gfx, prompt);
    let total = width + gap + size;
    // Each copy's own box is the text's width, so the text starts where it is put.
    gfx.fonts.font_block.game_fonts[2].update(|f| f.box_width = width as u32);
    let (from, to) = crate::gfx::button_pictures::outline_rows(picture.family, picture.icon)
        .unwrap_or((0.0, 1.0));
    let cell = (size as f32 / (to - from).max(0.1)).round() as i32;
    let mut y = y;
    let mut start = x as i32 + (0x168 - total) / 2;
    let mut drawn = Ok(());
    for step in 0..3 {
        if step > 0 {
            y = y.wrapping_add(line(gfx));
            start -= 0x78;
        }
        drawn = text(font(gfx), &strings.text.buffer.text, start as u32, y, gfx);
        if drawn.is_err() {
            break;
        }
        let middle = y as i32 + ink_top + size / 2;
        let top = middle - (cell as f32 * (from + to) / 2.0).round() as i32;
        let left = start + width + gap / 2 + (size - cell) / 2;
        put_picture(gfx, picture, left, top, cell);
    }
    gfx.fonts.font_block.game_fonts[2].update(|f| f.spacing = spacing);
    drawn
}

/// The title drawn into its own texture (`title_texture`) with the font's own letter spacing,
/// then copied to the device. The tyre change texture's strips are 256 wide and the Portuguese
/// "MUDANCA DE PNEUS" is wider than their box, so a title that does not fit one strip is drawn
/// across two (the texture's first row of strips holds its left half, the second its right half)
/// and shown as two quads side by side (`rc2k_core::drive::tyre_screen::title_parts`). The
/// width drawn is kept for those quads.
fn draw_title(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    gfx: &'static crate::gfx::owner::Gfx,
    shared: &RaceShared,
) -> Result<(), Refused> {
    use rc2k_core::drive::tyre_screen::{TITLE_BOX, TITLE_WIDE_BOX};
    let tyre = shared.tyre_change();
    let mut title = crate::text::draw_string_by_id::string_text(so, 0x129)?;
    title.pop();
    let width = text_width(gfx, &title).max(0) as u32;
    let wide = width > TITLE_BOX;
    if crate::d3d::lock_texture::lock_texture_in(gfx, tyre.title_texture.get())? {
        let clips = (
            clip(&gfx.drawing.blit.clip.left),
            clip(&gfx.drawing.blit.clip.right),
        );
        set_clip(&gfx.drawing.blit.clip.left, 0);
        set_clip(&gfx.drawing.blit.clip.right, 0xFF);
        let box_width = if wide { TITLE_WIDE_BOX } else { TITLE_BOX };
        gfx.fonts.font_block.game_fonts[2].update(|f| f.box_width = box_width);
        title.push(0);
        let drawn = (|| -> Result<(), Refused> {
            let mut canvas = crate::text::draw_text::GameCanvas::new(gfx);
            crate::text::draw_text::draw_text_at(font(gfx), &title[..], 4, 0, &mut canvas)?;
            if wide {
                crate::text::draw_text::draw_text_at(
                    font(gfx),
                    &title[..],
                    4 - 0x100,
                    0x19,
                    &mut canvas,
                )?;
            }
            Ok(())
        })();
        set_clip(&gfx.drawing.blit.clip.left, clips.0);
        set_clip(&gfx.drawing.blit.clip.right, clips.1);
        drawn?;
        crate::d3d::unlock_texture::unlock_texture(gfx, tyre.title_texture.get())?;
        crate::d3d::copy_to_device_texture::copy_to_device_texture(
            tyre.title_texture.get(),
            tyre.title_device_texture.get(),
        );
    }
    tyre.title_width.set(width);
    Ok(())
}

/// Draws the tyre change's texture for the device `device`.
pub fn draw_tyre_change(
    o: crate::owners::Owners<'static>,
    race: &'static Race,
    shared: &RaceShared,
    device: &InputDevice,
) -> Result<(), Refused> {
    let (gfx, strings) = (o.gfx, o.strings);
    let so = crate::strings::append_string_by_id::StringOwners::of(o);
    let clips = (
        clip(&gfx.drawing.blit.clip.left),
        clip(&gfx.drawing.blit.clip.right),
    );
    let done = (|| -> Result<(), Refused> {
        if shared.tyre_change().drawn.get() != 1 {
            draw_title(so, gfx, shared)?;
        }
        let locked =
            crate::d3d::lock_texture::lock_texture_in(gfx, shared.tyre_change().texture.get())?;
        if locked && shared.tyre_change().drawn.get() != 1 {
            set_clip(&gfx.drawing.blit.clip.left, 0x80);
            let mut y = 0u32;
            for (n, (id, font, x)) in [
                (0x12Au32, &gfx.fonts.font_block.game_fonts[1], 0xFCu32),
                (0x12B, &gfx.fonts.font_block.game_fonts[0], 0x84),
            ]
            .into_iter()
            .enumerate()
            {
                // The first text is at the top, the second a line below it.
                y = if n == 0 { 0 } else { y.wrapping_add(line(gfx)) };
                crate::text::draw_string_by_id::draw_string_by_id_at(
                    so,
                    id,
                    font,
                    x as i32,
                    y as i32,
                    &mut crate::text::draw_text::GameCanvas::new(gfx),
                )?;
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
            three_times(gfx, strings, x.wrapping_add(4), 0xB2)?;
        } else {
            let prompt = build_text(so, strings, 0x12C, None)?;
            match stop_picture(o, shared.tyre_change().player.get()) {
                Some(picture) => {
                    three_times_with_picture(o, &prompt, picture, x.wrapping_add(4), 0xB2)?;
                }
                None => three_times(gfx, strings, x.wrapping_add(4), 0xB2)?,
            }
        }
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
