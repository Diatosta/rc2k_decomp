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
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading, times_page_ready};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::line_buffer;
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
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
/// The Rust version.
pub unsafe fn draw_player_rally_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
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
        // Converted: `rc2k_core::screens::player_result::rally_result`.
        use crate::screens::game_screen::PlayerResultScreen;
        let p = rc2k_core::screens::player_result::Player {
            name: name as u32,
            nation: f(4),
            car: f(8),
            time: f(0x88),
            speed,
        };
        let figures = rc2k_core::screens::player_result::RallyFigures {
            position: k(0x758768),
            best: k(0x758758),
            column: [0x758778, 0x758758, 0x758788, 0x758798, 0x7587A8, 0x7587B8].map(k),
        };
        rc2k_core::screens::player_result::rally_result(
            &mut PlayerResultScreen::new(canvas),
            &mut |s: &mut PlayerResultScreen| draw_times_heading(&RALLY_STAGE_TIMES, s.canvas),
            &p,
            &figures,
            crate::strings::append_time::show_hours(),
        )?;
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
