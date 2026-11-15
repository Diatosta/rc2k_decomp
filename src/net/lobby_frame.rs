//! The frame of the lobby: background, the table's headings, the sorted players one row each
//! (flag, name, car, race time and total, or gaps to the first row when `show_gaps`), the chat's
//! eight lines and input, the four buttons and two lines. More than eight players are refused.

use crate::errors::{BadInput, Refused};
use crate::net::lobby::{FONT, blit, copy_picture, lock_surface, push_id, store_text};
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::players;
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
    let (net, strings, gfx) = (o.net, o.strings, o.gfx);
    let text = &strings.text;
    let vars = &net.vars;
    // The logic is in `rc2k_core::net::screens` (`shown_time`, `flag_sprite`, `car_sprite`,
    // `NO_TIME`, `lobby_buttons`), the ready bit and no time of `rc2k_core::net::players`.
    crate::net::drain_chat::drain_chat(net, gfx)?;
    if !lock_surface(o.gfx, o.system) {
        return Ok(());
    }
    if net.race_slot.players() > 8 {
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
    if net.race_slot.players() != 0 {
        let sorted = crate::net::sort_players::sort_players(net)?;
        let gaps = net.lobby().show_gaps.get() == 1;
        let (first_time, first_total) = (
            sorted[0].record.race_time.get(),
            sorted[0].record.total.get(),
        );
        for n in 0..net.race_slot.players() {
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
            let font = if block.record.state().is_ready() {
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
            use crate::strings::append_time::SHOW_HOURS;
            let kept_hours = SHOW_HOURS.get().get();
            SHOW_HOURS.get().set(1);
            let time = block.record.race_time.get();
            // The total is drawn at the y the drawing before it leaves: the text's, font offset
            // included, when there is no time (kept as is).
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
    {
        let _received = net.lock_received();
        crate::text::draw_text::draw_text_at(
            CHAT_FONT,
            &vars.chat.read(),
            0xA,
            y as i32,
            &mut canvas,
        )?;
    }
    y = y.wrapping_add(
        CHAT_FONT
            .object(&o.gfx.fonts.typefaces)
            .line_height
            .get()
            .wrapping_mul(8),
    );
    crate::net::chat_input_line::chat_input_line(o, &net.lobby().chat_input.text(), 3, y)?;
    let mut x = 0x310;
    let mut item = 0;
    for (n, (id, shown)) in
        screens::lobby_buttons(net.lobby().car_item.get(), vars.stage_item.get())
            .into_iter()
            .enumerate()
    {
        if shown {
            x = crate::net::lobby_button::lobby_button(
                crate::strings::append_string_by_id::StringOwners::of(o),
                net,
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
    crate::platform::show_frame_now(o);
    Ok(())
}
