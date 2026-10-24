//! The race frame's texts (`sub_4547AE`'s, IDA 2026-10-18): the frame rate and the credits
//! that roll over the race (`dword_71C3E0`), also used by the credits' screen.

use crate::audio::store::Slot;
use crate::errors::Refused;
use crate::framework::rva;
use crate::screens::font_at;
use crate::screens::harness::Left;
use crate::session::SCREEN;
use crate::store::{game_read, game_write};
use crate::strings::text_from_ids::text;
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};

/// `race::draw_frame_rate` (0x4562B2): `dword_5C6604` 2; the frame rate (`flt_6214F0`,
/// rounded into `dword_62400C`), when not 0 and `dword_5EB100` is set, drawn at (0x190, 4)
/// in the font at 0x51DE90 as "fps: n" (its size; built in the text buffer). Gives what
/// `draw_text` leaves, or nothing.
pub unsafe fn draw_frame_rate(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::frame_rate::{shown_rate, fps_text}`.
    use rc2k_core::race::frame_rate::{fps_text, shown_rate};
    unsafe {
        game_write(rva(0x5C6604), 2u32);
        let rate = shown_rate(f32::from_bits(game_read(rva(0x6214F0))));
        game_write(rva(0x62400C), rate as u32);
        if rate == 0 || game_read::<u32>(rva(0x5EB100)) == 0 {
            return Ok(Left::nothing());
        }
        let t = text();
        let bytes = fps_text(rate);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), t, bytes.len());
        let out = draw_text_at(font_at(0x51DE90), t, 0x190, 4, canvas)?;
        Ok(Left::of_text(&out))
    }
}

/// `sub_4161BA`: string `id` in `font` at (x, y), unless y is at or above the screen's
/// bottom (`g_screenHeight`) or at or below -0x1B. Gives what `draw_text` leaves, or
/// nothing.
unsafe fn credit_line(
    id: u32,
    font: u32,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::credits::shown`.
    unsafe {
        if !rc2k_core::race::credits::shown(y, SCREEN.height.get()) {
            return Ok(Left::nothing());
        }
        let out = draw_string_by_id_at(id, font as *mut crate::text::font::Font, x, y, canvas)?;
        Ok(Left::of_text(&out))
    }
}

/// `race::roll_credits` (0x415FC2): eax 0 starts them (the clock `dword_529850`, the top
/// `dword_52984C` the screen's height + 0x15E, `dword_529854` 0, tune 2 by `sound`); else
/// draws the script at 0x529858 (words: 1, 2, 7, 8 a string in fonts 0x51E8A4, 0x51E6A4,
/// 0x51E564 (centred) and 0x51E584 (at a sixteenth), each moving down a line; 3 / 4 a
/// string left / right of the middle (only 4 moves down); 5 three strings in columns; 6 a
/// signed move; 9 nothing; any other the end) from the top down, moves the top up a pixel
/// for each 0x12 ms passed, and when the last line is above -0xAF and the music has
/// stopped, `dword_529854` 1 (the original also leaves flags no caller reads).
pub unsafe fn roll_credits(
    eax: u32,
    canvas: &mut dyn Canvas,
    sound: &mut dyn FnMut(),
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::credits::{start_top, lay_out, catch_up, rolled_off}`.
    use crate::system::tick_count::tick_count;
    use rc2k_core::race::credits::{Font, catch_up, lay_out, rolled_off, start_top};
    unsafe {
        let width = SCREEN.width.get() as i32;
        let height: u32 = SCREEN.height.get();
        if eax == 0 {
            game_write(rva(0x529850), tick_count());
            let top = start_top(height);
            game_write(rva(0x52984C), top);
            game_write(rva(0x529854), 0u32);
            sound();
            return Ok(());
        }
        let font_of = |f: Font| {
            let at = match f.0 {
                1 => 0x51E8A4,
                2 => 0x51E6A4,
                3 => 0x51E6E4,
                4 => 0x51E6C4,
                7 => 0x51E564,
                _ => 0x51E584,
            };
            crate::framework::game_addr(rva(at)) as u32
        };
        let mut at = crate::framework::game_addr(rva(0x529858)) as u32;
        let (lines, below) = lay_out(
            || {
                let v = (at as *const u16).read_unaligned();
                at += 2;
                v
            },
            width,
            game_read::<u32>(rva(0x52984C)) as i32,
            |f| ((font_of(f) + 8) as *const i32).read_unaligned(),
        );
        for line in lines {
            credit_line(line.id, font_of(line.font), line.x, line.y, canvas)?;
        }
        let (top, clock, _) = catch_up(game_read(rva(0x52984C)), game_read(rva(0x529850)), || {
            tick_count()
        });
        game_write(rva(0x52984C), top);
        game_write(rva(0x529850), clock);
        if !rolled_off(below, game_read(Slot::at(0x506990))) {
            return Ok(());
        }
        game_write(rva(0x529854), 1u32);
        Ok(())
    }
}

/// `roll_credits`' tune in the game: `sound::play_tune_2`'s command, sent and waited for.
pub fn credits_tune() {
    crate::sound::play_tune_2::play_tune_2();
}
