//! `net::lobby_frame` (0x417D95): the frame of the lobby (`net::lobby_loop`): on the locked
//! surface the background; the table's headings (strings 0x15F at 0x21A and 0x160 at 0x2C0,
//! `typefaces::TIME`, y 0x6F); the players sorted (`net::sort_players`) and one row each from y
//! 0x8C: the nationality's flag (sprite +0x27 + 0x1C at 0x18), the name (`typefaces::READY` when
//! ready, else `typefaces::VALUES`), the car's sprite (+0x25 + 0x44, 0x18 and 0x19 swapped), the
//! race's time and the total (`text::draw_time_text` with the hours always shown, or string
//! 0x161 for no time; with gaps shown, `show_gaps` 1, each as its gap to the first row's
//! with a '+' over its first character when within 0x36EE7F); the chat's eight lines (under
//! the received messages' lock) and its input (`net::chat_input_line`, item 3); the four
//! buttons right to left from 0x310 at 0x223 (`net::lobby_button`: ready 0x15D, the car 0x15B
//! and the stage 0x15C when they can be chosen, the chat 0x15E); two lines (`gfx::draw_hline`
//! at y 0x156 and 0x217, 0x320 long, colour 7); unlocked and shown.
//!
//! More than eight players would read past the sorted copy: refused. Called by
//! `screens::run_frame` for `net::lobby_loop`.

use crate::errors::{BadInput, Refused};
use crate::net::lobby::{FONT, blit, copy_picture, lock_surface, push_id, store_text};
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::players::{self, READY};
use rc2k_core::net::screens;

const HEADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::TIME;
const NAME_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::VALUES;
const READY_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::READY;
const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NAME;

/// A time, or its gap to `first` with a '+' over its first character.
fn time_text(time: u32, gaps: bool, first: u32) -> Vec<u8> {
    let (value, plus) = screens::shown_time(time, first, gaps);
    let mut text =
        crate::strings::append_time::append_time(value, crate::strings::append_time::show_hours());
    if plus && let Some(first) = text.first_mut() {
        *first = b'+';
    }
    text
}

/// Called through
/// `screens::run_frame` by `net::lobby_loop`.
pub fn lobby_frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (net, install, strings, gfx) = (o.net, o.install, o.strings, o.gfx);
    let text = &strings.text;
    let vars = &net.vars;
    // Converted: `rc2k_core::net::screens` (`shown_time`, `flag_sprite`, `car_sprite`,
    // `NO_TIME`, `lobby_buttons`), the ready bit and no time of `rc2k_core::net::players`.
    if !lock_surface(o.gfx, o.system) {
        return Ok(());
    }
    if vars.players.get() > 8 {
        return Err(BadInput("more than eight players").into());
    }
    copy_picture(o.gfx, o.system)?;
    let mut canvas = GameCanvas::new(o.gfx);
    let mut y = 0x6F;
    for (id, x) in [(0x15F, 0x21A), (0x160, 0x2C0)] {
        crate::text::draw_string_by_id::draw_string_by_id_at(
            crate::strings::append_string_by_id::StringOwners::of(o),
            id,
            HEADING_FONT,
            x,
            y as i32,
            &mut canvas,
        )?;
    }
    y += 0x1D;
    if vars.players.get() != 0 {
        let sorted = crate::net::sort_players::sort_players(&net.vars)?;
        let gaps = vars.show_gaps.get() == 1;
        let (first_time, first_total) = (
            sorted[0].record.race_time.get(),
            sorted[0].record.total.get(),
        );
        for n in 0..vars.players.get() {
            let block = &sorted[n as usize];
            let mut x = 0x18;
            blit(
                screens::flag_sprite(block.record.head.get(3)),
                x,
                y,
                o.gfx,
                o.menus,
            )?;
            x += 0x28;
            let name = block.name.text();
            if name.len() == block.name.read().len() {
                return Err(BadInput("a text without its end in its buffer").into());
            }
            let name = store_text(strings, name)?;
            let font = if block.record.state() & READY != 0 {
                READY_FONT
            } else {
                NAME_FONT
            };
            crate::text::draw_text::draw_text_at(font, &name, x as i32, y as i32, &mut canvas)?;
            x += 0x14A;
            blit(
                screens::car_sprite(block.record.head.get(1)),
                x,
                y + 1,
                o.gfx,
                o.menus,
            )?;
            x += 0x48;
            use crate::strings::append_time::SHOW_HOURS; // g_timeShowHours
            let kept_hours = SHOW_HOURS.get().get();
            SHOW_HOURS.get().set(1);
            let time = block.record.race_time.get();
            // The total is drawn at the y the drawing before it leaves: the text's,
            // font offset included, when there is no time (the original's slip).
            let mut total_y = y;
            if time == players::NO_TIME {
                let mut none = Vec::new();
                push_id(o, &mut none, screens::NO_TIME)?;
                let none = store_text(strings, none)?;
                let out = {
                    crate::text::draw_text::draw_text_at(
                        NAME_FONT,
                        &none,
                        x as i32,
                        y as i32,
                        &mut canvas,
                    )
                }?;
                total_y = out.y as u32;
            } else {
                store_text(strings, time_text(time, gaps, first_time))?;
                {
                    crate::text::draw_time_text::draw_time_text(
                        FONT,
                        &text.buffer.text.read(),
                        x,
                        y,
                        o.gfx,
                    )
                }?;
            }
            x += 0xA5;
            let timed = time != players::NO_TIME;
            store_text(
                strings,
                time_text(block.record.total.get(), gaps && timed, first_total),
            )?;
            {
                crate::text::draw_time_text::draw_time_text(
                    FONT,
                    &text.buffer.text.read(),
                    x,
                    total_y,
                    o.gfx,
                )
            }?;
            SHOW_HOURS.get().set(kept_hours);
            y = y.wrapping_add(NAME_FONT.object(&o.gfx.fonts.typefaces).line_height.get());
        }
    }
    y = 0x158;
    crate::net::lock_received::lock_received(&net.vars);
    {
        crate::text::draw_text::draw_text_at(
            CHAT_FONT,
            &vars.chat.read(),
            0xA,
            y as i32,
            &mut canvas,
        )
    }?;
    crate::net::unlock_received::unlock_received(&net.vars);
    y = y.wrapping_add(
        CHAT_FONT
            .object(&o.gfx.fonts.typefaces)
            .line_height
            .get()
            .wrapping_mul(8),
    );
    crate::net::chat_input_line::chat_input_line(o, &net.edit().chat_input.text(), 3, y)?;
    let mut x = 0x310;
    let mut item = 0;
    for (n, (id, shown)) in
        screens::lobby_buttons(net.setup().car_item.get(), vars.stage_item.get())
            .into_iter()
            .enumerate()
    {
        if shown {
            x = crate::net::lobby_button::lobby_button(
                crate::strings::append_string_by_id::StringOwners::of(o),
                net,
                install,
                id,
                item,
                x,
                0x223,
                o.gfx,
                o.menus,
            )?;
        }
        if n != 3 {
            item += 1;
        }
    }
    for line_y in [0x156, 0x217] {
        crate::gfx::rect::draw_hline(
            0,
            line_y,
            0x320,
            7,
            crate::gfx::rect::Palette::Held(gfx.fonts.font_block.palettes.get()),
            gfx,
        );
    }
    crate::platform::unlock_surface(o.gfx);
    crate::platform::show_frame_now();
    Ok(())
}
