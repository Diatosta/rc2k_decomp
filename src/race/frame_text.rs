//! The race frame's texts (`sub_4547AE`'s, IDA 2026-10-18): the frame rate and the credits
//! that roll over the race (`dword_71C3E0`), also used by the credits' screen.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::screens::font_at;
use crate::screens::harness::Left;
use crate::strings::text_from_ids::text;
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};

/// `race::draw_frame_rate` (0x4562B2): `dword_5C6604` 2; the frame rate (`flt_6214F0`,
/// rounded into `dword_62400C`), when not 0 and `dword_5EB100` is set, drawn at (0x190, 4)
/// in the font at 0x51DE90 as "fps: n" (its size; built in the text buffer). Gives what
/// `draw_text` leaves, or nothing.
pub unsafe fn draw_frame_rate(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe {
        game_write(rva(0x5C6604), 2u32);
        let rate = crate::math::x87::to_int(game_read(rva(0x6214F0))) as u32;
        game_write(rva(0x62400C), rate);
        if rate == 0 || game_read::<u32>(rva(0x5EB100)) == 0 {
            return Ok(Left::nothing());
        }
        let t = text();
        std::ptr::copy_nonoverlapping(b"fps:    ".as_ptr(), t, 8);
        let digits = crate::strings::format_u32::format_u32((rate as i32).wrapping_abs() as u32, 0);
        std::ptr::copy_nonoverlapping(digits.as_ptr(), t.add(5), digits.len());
        *t.add(5 + digits.len()) = 0;
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
    unsafe {
        if y <= -0x1B || y >= game_read::<u32>(rva(0x4F4950)) as i32 {
            return Ok(Left::nothing());
        }
        let out = draw_string_by_id_at(id, font as *mut crate::text::font::Font, x, y, canvas)?;
        Ok(Left::of_text(&out))
    }
}

/// What the credits leave: the registers (all kept by `pusha` .. `popa`) and the flags:
/// carry when they have rolled off and the music has stopped, else not and zero by eax.
pub struct Rolled {
    pub carry: bool,
    pub zero: bool,
}

/// `race::roll_credits` (0x415FC2): eax 0 starts them (the clock `dword_529850`, the top
/// `dword_52984C` the screen's height + 0x15E, `dword_529854` 0, tune 2 by `sound`); else
/// draws the script at 0x529858 (words: 1, 2, 7, 8 a string in fonts 0x51E8A4, 0x51E6A4,
/// 0x51E564 (centred) and 0x51E584 (at a sixteenth), each moving down a line; 3 / 4 a
/// string left / right of the middle (only 4 moves down); 5 three strings in columns; 6 a
/// signed move; 9 nothing; any other the end) from the top down, moves the top up a pixel
/// for each 0x12 ms passed, and when the last line is above -0xAF and the music has
/// stopped, `dword_529854` 1 and carry.
pub unsafe fn roll_credits(
    eax: u32,
    canvas: &mut dyn Canvas,
    sound: &mut dyn FnMut(&Regs),
    entry: &Regs,
) -> Result<Rolled, Refused> {
    use crate::system::tick_count::tick_count;
    unsafe {
        let width = game_read::<u32>(rva(0x4F494C)) as i32;
        let height: u32 = game_read(rva(0x4F4950));
        if eax == 0 {
            game_write(rva(0x529850), tick_count());
            game_write(rva(0x52984C), height.wrapping_add(0x15E));
            game_write(rva(0x529854), 0u32);
            // eax holds the top when the tune is sent, and is tested on the way out.
            let top = height.wrapping_add(0x15E);
            sound(&Regs { eax: top, ..*entry });
            return Ok(Rolled {
                carry: false,
                zero: top == 0,
            });
        }
        let mut at = crate::framework::game_addr(rva(0x529858)) as u32;
        let mut y = game_read::<u32>(rva(0x52984C)) as i32;
        let word = |at: &mut u32| -> u32 {
            let v = (*at as *const u16).read_unaligned() as u32;
            *at += 2;
            v
        };
        let line_height = |font: u32| {
            ((crate::framework::game_addr(rva(font as usize)) as u32 + 8) as *const i32)
                .read_unaligned()
        };
        loop {
            let op = word(&mut at);
            let font = match op {
                1 => Some((0x51E8A4u32, width >> 1, true)),
                2 => Some((0x51E6A4, width >> 1, true)),
                7 => Some((0x51E564, width >> 1, true)),
                8 => Some((0x51E584, width >> 4, true)),
                3 => Some((0x51E6E4, (width >> 1) - 0xF, false)),
                4 => Some((0x51E6C4, (width >> 1) + 0xF, true)),
                _ => None,
            };
            if let Some((f, x, moves)) = font {
                let id = word(&mut at);
                let line_y = y;
                if moves {
                    y = y.wrapping_add(line_height(f));
                }
                credit_line(
                    id,
                    crate::framework::game_addr(rva(f as usize)) as u32,
                    x,
                    line_y,
                    canvas,
                )?;
                continue;
            }
            match op {
                5 => {
                    let w = width as u32;
                    let edge = width >> 4;
                    let column = (w.wrapping_sub(edge as u32).wrapping_sub(edge as u32) / 3) as i32;
                    let f = crate::framework::game_addr(rva(0x51E6C4)) as u32;
                    for k in 0..3 {
                        let id = word(&mut at);
                        let line_y = y;
                        if k == 2 {
                            y = y.wrapping_add(line_height(0x51E6C4));
                        }
                        credit_line(
                            id,
                            f,
                            column.wrapping_mul(k).wrapping_add(edge),
                            line_y,
                            canvas,
                        )?;
                    }
                }
                6 => {
                    let step = (at as *const i16).read_unaligned() as i32;
                    at += 2;
                    y = y.wrapping_add(step);
                }
                9 => {}
                _ => break,
            }
        }
        let mut elapsed;
        loop {
            elapsed = tick_count().wrapping_sub(game_read(rva(0x529850)));
            if (elapsed.wrapping_sub(0x12) as i32) < 0 {
                break;
            }
            game_write(
                rva(0x52984C),
                game_read::<u32>(rva(0x52984C)).wrapping_sub(1),
            );
            game_write(
                rva(0x529850),
                game_read::<u32>(rva(0x529850)).wrapping_add(0x12),
            );
        }
        if y > -0xAF || game_read::<u32>(rva(0x506990)) == 1 {
            return Ok(Rolled {
                carry: false,
                zero: elapsed == 0,
            });
        }
        game_write(rva(0x529854), 1u32);
        Ok(Rolled {
            carry: true,
            zero: false,
        })
    }
}

/// `roll_credits`' tune in the game: `sound::play_tune_2`'s command, sent and waited for.
pub fn credits_tune(entry: &Regs) {
    use crate::sound::queue::{Queue, send_command, sent};
    use crate::sound::wait_for_commands::{Game, wait_for_commands};
    if let Some(with) = crate::sound::play_tune_2::play_tune_2(entry) {
        unsafe { send_command(&Queue::game(), 0xB, sent(&with)) };
        wait_for_commands(&mut Game(sent(&with)));
    }
}

/// `credit_line` for the hook of 0x4161BA (`race::credit_line`).
pub unsafe fn credit_line_at(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { credit_line(r.eax, r.ebx, r.ecx as i32, r.edx as i32, canvas) }
}
