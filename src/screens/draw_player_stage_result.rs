//! `screens::draw_player_stage_result` (0x486299): the player's result on a championship
//! stage: the three labels (strings 0x206 .. 0x208 at x 0x1EC, font 0x51E6E4), the stage's
//! heading (`screens::draw_stage_times_heading`'s), the player's time (+0x88) and top speed
//! (+0x94, with its unit) at x 0x1FE, the car's sprite, the name (+0x18) at x 0x221 and the
//! flag's sprite after it (its width by `text::measure_text` in the font at 0x51E624); the
//! sprites' palette ebx as the original leaves it (the speed line past its NUL, then the
//! measure's line count).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E7E). Refuses (before anything) no current player or
//! name, no stage records, and what `strings::append_speed` refuses. Compared in shadow mode
//! (`dispatch_draw`). Self-test: the page's stage and the players, nothing drawn (skipped
//! without the fonts or the records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::best_scores::{STAGE_TIMES, draw_times_heading, times_page_ready};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::{font_at, line_buffer};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_player_stage_result",
    rva(0x486299),
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

unsafe fn set_line(bytes: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len()) };
}

/// The Rust version.
pub unsafe fn draw_player_stage_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::text::draw_string_by_id::draw_string_by_id_at;
    unsafe {
        let player = crate::menu::player_settings::current()?;
        let p = player as u32;
        let f = |o: u32| ((p + o) as *const u32).read_unaligned();
        let name = crate::menu::player_name::name()?;
        let mut speed = crate::strings::append_speed::append_speed(
            f(0x94),
            true,
            crate::strings::units::units(),
        )?;
        speed.push(0);
        if (STAGE_TIMES.records)() == 0 {
            return Err(crate::errors::BadInput("no stage records").into());
        }
        let labels = font_at(0x51E6E4);
        for (n, id) in [0x206u32, 0x207, 0x208].into_iter().enumerate() {
            draw_string_by_id_at(id, labels, 0x1EC, 0xCF + 0x2D * n as i32, canvas)?;
        }
        draw_times_heading(&STAGE_TIMES, canvas)?;
        let font = font_at(0x51E6C4);
        let mut time = crate::strings::append_time::append_time(
            f(0x88),
            crate::strings::append_time::show_hours(),
        );
        time.push(0);
        set_line(&time);
        draw_text_at(font, line_buffer(), 0x1FE, 0xFC, canvas)?;
        set_line(&speed);
        let out = draw_text_at(font, line_buffer(), 0x1FE, 0x12A, canvas)?;
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        canvas.blit_glyph(BlitCall::new(
            0x1C + f(4),
            0x1FE,
            0xCF,
            out.text as u32,
            sprites,
        ))?;
        draw_text_at(font, name, 0x221, 0xCF, canvas)?;
        let m = crate::text::measure_text::measure_text_at(font_at(0x51E624), name, 0x221, canvas)?;
        let mut flag = f(8);
        if (0x18..=0x19).contains(&flag) {
            flag ^= 1;
        }
        let x = (m.width as i32).wrapping_add(0x221);
        canvas.blit_glyph(BlitCall::new(
            flag.wrapping_add(0x44),
            x,
            0xCF,
            m.lines,
            sprites,
        ))?;
        Ok(())
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_player_stage_result(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (line_buffer() as u32, 0x100),
        (crate::strings::text_from_ids::text() as u32, 0x100),
        (game_addr(rva(0x51E624)) as u32, 0x20),
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
    if !unsafe { times_page_ready(&STAGE_TIMES) } {
        return t.skip("no stage records");
    }
    let regions: Vec<Region> = vec![at(rva(0x75598C))];
    let saved = capture(&regions);
    for player in 0..crate::menu::player::PLAYERS {
        if crate::menu::player::player_at(player) == 0 {
            continue;
        }
        // A top speed the original's `miles_to_km` traps on (mid-race data while the page is
        // not shown): not a case.
        unsafe { game_write(rva(0x75598C), player) };
        let traps = crate::menu::player_settings::current().is_ok_and(|block| {
            let speed = unsafe { ((block as u32 + 0x94) as *const u32).read_unaligned() };
            crate::strings::append_speed::traps(speed)
        });
        if traps {
            continue;
        }
        check_draw(
            &mut t,
            &format!("player {player}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(0x75598C), player) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
