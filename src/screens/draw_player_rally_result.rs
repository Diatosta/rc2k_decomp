//! `screens::draw_player_rally_result` (0x4894AD): the player's result in a rally stage: the
//! labels (string 0x213 at (0xED, 0xE7), strings 0x20E .. 0x212 at x 0x155), the stage's
//! heading (`screens::draw_rally_times_heading`'s), the player's time (+0x88; string 0x513
//! for none, 0x980000) and top speed (+0x94), the rally's numbers and times
//! (`dword_758758` .. `dword_7587B8`), the car's sprite, the name and the flag's sprite (their
//! palettes ebx as the original leaves it: the last text past its NUL).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E76). Refuses (before anything) no current player or
//! name, no rally stage records, and what `strings::append_speed` refuses. Compared in shadow
//! mode (`dispatch_draw`). Self-test: the players, a time and none, nothing drawn (skipped
//! without the fonts or the records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading, times_page_ready};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::{font_at, line_buffer};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_player_rally_result",
    rva(0x4894AD),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

const NO_TIME: u32 = 0x98_0000;

/// (id, x, y, font) of the labels.
const LABELS: [(u32, i32, i32, usize); 7] = [
    (0x213, 0xED, 0xE7, 0x51E6C4),
    (0x20E, 0x155, 0x12B, 0x51E6E4),
    (0x20F, 0x155, 0x142, 0x51E6E4),
    (0x208, 0x155, 0x159, 0x51E6E4),
    (0x210, 0x155, 0x187, 0x51E6E4),
    (0x211, 0x155, 0x19F, 0x51E6E4),
    (0x212, 0x155, 0x1CD, 0x51E6E4),
];

/// A line of the page.
enum Value {
    Number(usize),
    Time(u32),
    Speed(Vec<u8>),
}

/// The Rust version.
pub unsafe fn draw_player_rally_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::text::draw_string_by_id::draw_string_by_id_at;
    unsafe {
        let p = crate::menu::player_settings::current()? as u32;
        let f = |o: u32| ((p + o) as *const u32).read_unaligned();
        let name = crate::menu::player_name::name()?;
        if (RALLY_STAGE_TIMES.records)() == 0 {
            return Err(crate::errors::BadInput("no rally stage records").into());
        }
        let speed = crate::strings::append_speed::append_speed(
            f(0x94),
            true,
            crate::strings::units::units(),
        )?;
        for (id, x, y, font) in LABELS {
            draw_string_by_id_at(id, font_at(font), x, y, canvas)?;
        }
        draw_times_heading(&RALLY_STAGE_TIMES, canvas)?;
        let font = font_at(0x51E6C4);
        let draw = |v: Value, x: i32, y: i32, canvas: &mut dyn Canvas| -> Result<u32, Refused> {
            let mut line = match v {
                Value::Number(a) => crate::strings::append_u32::append_u32(k(a)),
                Value::Time(t) => crate::strings::append_time::append_time(
                    t,
                    crate::strings::append_time::show_hours(),
                ),
                Value::Speed(s) => s,
            };
            line.push(0);
            std::ptr::copy_nonoverlapping(line.as_ptr(), line_buffer(), line.len());
            Ok(draw_text_at(font, line_buffer(), x, y, canvas)?.text as u32)
        };
        draw(Value::Number(0x758768), 0x18C, 0x12B, canvas)?;
        if f(0x88) == NO_TIME {
            draw_string_by_id_at(0x513, font, 0x1CF, 0xE7, canvas)?;
        } else {
            draw(Value::Time(f(0x88)), 0x23A, 0xE7, canvas)?;
        }
        draw(Value::Time(k(0x758758)), 0x18C, 0x142, canvas)?;
        draw(Value::Speed(speed), 0x18C, 0x159, canvas)?;
        let mut palette = 0;
        for (a, y) in [
            (0x758778usize, 0x12B),
            (0x758758, 0x142),
            (0x758788, 0x159),
            (0x758798, 0x187),
            (0x7587A8, 0x19F),
            (0x7587B8, 0x1CD),
        ] {
            palette = draw(Value::Number(a), 0x21C, y, canvas)?;
        }
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        canvas.blit_glyph(BlitCall::new(0x1C + f(4), 0x146, 0xE7, palette, sprites))?;
        let out = draw_text_at(font_at(0x51E584), name, 0x168, 0xE7, canvas)?;
        let mut flag = f(8);
        if (0x18..=0x19).contains(&flag) {
            flag ^= 1;
        }
        canvas.blit_glyph(BlitCall::new(
            flag.wrapping_add(0x44),
            0x1FE,
            0xE7,
            out.text as u32,
            sprites,
        ))?;
        Ok(())
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_player_rally_result(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (line_buffer() as u32, 0x100),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    if !unsafe { times_page_ready(&RALLY_STAGE_TIMES) } {
        return t.skip("no rally stage records");
    }
    let mut regions: Vec<Region> = vec![at(rva(0x75598C))];
    for n in 0..crate::menu::player::PLAYERS {
        let p = crate::menu::player::player_at(n);
        if p != 0 {
            regions.push((p + 0x88, 4));
        }
    }
    let saved = capture(&regions);
    for player in 0..crate::menu::player::PLAYERS {
        let p = crate::menu::player::player_at(player);
        // A top speed the original traps on (mid-race data in a stage): not a case.
        if p == 0
            || crate::strings::append_speed::traps(unsafe {
                ((p + 0x94) as *const u32).read_unaligned()
            })
        {
            continue;
        }
        for time in [None, Some(NO_TIME), Some(12345u32)] {
            check_draw(
                &mut t,
                &format!("player {player} time {time:x?}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &saved);
                    game_write(rva(0x75598C), player);
                    if let Some(v) = time {
                        ((p + 0x88) as *mut u32).write_unaligned(v);
                    }
                },
                &rust,
                &state,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
