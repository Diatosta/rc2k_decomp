//! `net::lobby_frame` (0x417D95): the frame of the lobby (`net::lobby_loop`): on the locked
//! surface the background; the table's headings (strings 0x15F at 0x21A and 0x160 at 0x2C0,
//! `stru_51E564`, y 0x6F); the players sorted (`net::sort_players`) and one row each from y
//! 0x8C: the nationality's flag (sprite +0x27 + 0x1C at 0x18), the name (`stru_51E784` when
//! ready, else `stru_51E6C4`), the car's sprite (+0x25 + 0x44, 0x18 and 0x19 swapped), the
//! race's time and the total (`text::draw_time_text` with the hours always shown, or string
//! 0x161 for no time; with gaps shown, `dword_52AC30` 1, each as its gap to the first row's
//! with a '+' over its first character when within 0x36EE7F); the chat's eight lines (under
//! the received messages' lock) and its input (`net::chat_input_line`, item 3); the four
//! buttons right to left from 0x310 at 0x223 (`net::lobby_button`: ready 0x15D, the car 0x15B
//! and the stage 0x15C when they can be chosen, the chat 0x15E); two lines (`gfx::draw_hline`
//! at y 0x156 and 0x217, 0x320 long, colour 7); unlocked and shown.
//!
//! In:  nothing
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! More than eight players would read past the sorted copy: refused. Called by
//! `screens::run_frame` for `net::lobby_loop`.

use crate::errors::{BadInput, Refused};
use crate::gfx::fonts_store::FONTS;
use crate::net::lobby::{Cursor, FONT, blit, copy_picture, lock_surface};
use crate::net::store::NET;
use crate::strings::text_buffer::TEXT_BUFFER;
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::players::{self, READY};
use rc2k_core::net::screens;

const HEADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::TIME;
const NAME_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::VALUES;
const READY_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::READY;
const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NAME;

/// A time, or its gap to `first` with a '+' over its first character, written at the
/// cursor.
fn time_text(cursor: &mut Cursor, time: u32, gaps: bool, first: u32) -> Result<(), Refused> {
    let (value, plus) = screens::shown_time(time, first, gaps);
    let start = cursor.0;
    cursor.bytes(&crate::strings::append_time::append_time(
        value,
        crate::strings::append_time::show_hours(),
    ))?;
    if plus {
        start.write(b"+", "a text line")?;
    }
    Ok(())
}

/// The Rust version (the original keeps every register). Called through
/// `screens::run_frame` by `net::lobby_loop`.
pub fn lobby_frame() -> Result<(), Refused> {
    // Converted: `rc2k_core::net::screens` (`shown_time`, `flag_sprite`, `car_sprite`,
    // `NO_TIME`, `lobby_buttons`), the ready bit and no time of `rc2k_core::net::players`.
    if !lock_surface() {
        return Ok(());
    }
    if NET.players.get() > 8 {
        return Err(BadInput("more than eight players").into());
    }
    copy_picture()?;
    let mut canvas = GameCanvas;
    let mut y = 0x6F;
    // The blits are handed the palette (ebx) the drawing before them leaves: the text
    // pointer a heading or a name leaves, or the text buffer.
    let mut palette = 0;
    for (id, x) in [(0x15F, 0x21A), (0x160, 0x2C0)] {
        let out = unsafe {
            crate::text::draw_string_by_id::draw_string_by_id_at(
                id,
                HEADING_FONT.as_ptr(),
                x,
                y as i32,
                &mut canvas,
            )
        }?;
        palette = crate::pointers::game_address(out.text);
    }
    y += 0x1D;
    if NET.players.get() != 0 {
        crate::net::sort_players::sort_players()?;
        let gaps = NET.show_gaps.get() == 1;
        let (first_time, first_total) = (
            NET.sorted[0].record.race_time.get(),
            NET.sorted[0].record.total.get(),
        );
        for n in 0..NET.players.get() {
            let block = &NET.sorted[n as usize];
            let mut x = 0x18;
            blit(
                screens::flag_sprite(block.record.head.get(3)),
                x,
                y,
                palette,
            )?;
            x += 0x28;
            let mut cursor = Cursor::text();
            cursor.string(block.name.addr());
            cursor.nul()?;
            let font = if block.record.state() & READY != 0 {
                READY_FONT
            } else {
                NAME_FONT
            };
            let name = {
                crate::text::draw_text::draw_text_at(
                    font.as_ptr(),
                    TEXT_BUFFER.text.as_ptr() as *const u8,
                    x as i32,
                    y as i32,
                    &mut canvas,
                )
            }?;
            x += 0x14A;
            blit(
                screens::car_sprite(block.record.head.get(1)),
                x,
                y + 1,
                crate::pointers::game_address(name.text),
            )?;
            x += 0x48;
            use crate::strings::append_time::SHOW_HOURS; // g_timeShowHours
            use std::sync::atomic::Ordering::Relaxed;
            let kept_hours = SHOW_HOURS.swap(1, Relaxed);
            let time = block.record.race_time.get();
            let mut cursor = Cursor::text();
            // The total is drawn at the y the drawing before it leaves in edx: the text's,
            // font offset included, when there is no time (the original's slip).
            let mut total_y = y;
            if time == players::NO_TIME {
                cursor.string_id(screens::NO_TIME);
                cursor.nul()?;
                let out = {
                    crate::text::draw_text::draw_text_at(
                        NAME_FONT.as_ptr(),
                        TEXT_BUFFER.text.as_ptr() as *const u8,
                        x as i32,
                        y as i32,
                        &mut canvas,
                    )
                }?;
                total_y = out.y as u32;
            } else {
                time_text(&mut cursor, time, gaps, first_time)?;
                cursor.nul()?;
                {
                    crate::text::draw_time_text::draw_time_text(
                        FONT.as_ptr(),
                        TEXT_BUFFER.text.addr(),
                        x,
                        y,
                    )
                }?;
            }
            x += 0xA5;
            let timed = time != players::NO_TIME;
            let mut cursor = Cursor::text();
            time_text(
                &mut cursor,
                block.record.total.get(),
                gaps && timed,
                first_total,
            )?;
            cursor.nul()?;
            {
                crate::text::draw_time_text::draw_time_text(
                    FONT.as_ptr(),
                    TEXT_BUFFER.text.addr(),
                    x,
                    total_y,
                )
            }?;
            SHOW_HOURS.store(kept_hours, Relaxed);
            // The next row's blit gets the text buffer's address.
            palette = TEXT_BUFFER.text.addr();
            y = y.wrapping_add(NAME_FONT.object().line_height.get());
        }
    }
    y = 0x158;
    crate::net::lock_received::lock_received();
    {
        crate::text::draw_text::draw_text_at(
            CHAT_FONT.as_ptr(),
            NET.chat.as_ptr() as *const u8,
            0xA,
            y as i32,
            &mut canvas,
        )
    }?;
    crate::net::unlock_received::unlock_received();
    y = y.wrapping_add(CHAT_FONT.object().line_height.get().wrapping_mul(8));
    crate::net::chat_input_line::chat_input_line(NET.chat_input.addr(), 3, y)?;
    let mut x = 0x310;
    let mut item = 0;
    for (n, (id, shown)) in screens::lobby_buttons(NET.car_item.get(), NET.stage_item.get())
        .into_iter()
        .enumerate()
    {
        if shown {
            x = crate::net::lobby_button::lobby_button(id, item, x, 0x223)?;
        }
        if n != 3 {
            item += 1;
        }
    }
    for line_y in [0x156, 0x217] {
        unsafe { crate::gfx::rect::draw_hline(0, line_y, 0x320, 7, FONTS.palettes.get()) };
    }
    crate::platform::unlock_surface();
    crate::platform::show_frame_now();
    Ok(())
}
