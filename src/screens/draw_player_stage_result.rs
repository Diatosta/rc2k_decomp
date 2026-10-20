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
//! (`dispatch_draw`; the measuring font left out: see `drawn_state`). Self-test: the page's stage and the players, nothing drawn (skipped
//! without the fonts or the records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::best_scores::{STAGE_TIMES, draw_times_heading, times_page_ready};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::line_buffer;
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
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

/// The Rust version.
pub unsafe fn draw_player_stage_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
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
        // Converted: `rc2k_core::screens::player_result::stage_result`.
        use crate::screens::game_screen::PlayerResultScreen;
        speed.pop();
        let p = rc2k_core::screens::player_result::Player {
            name: name as u32,
            nation: f(4),
            car: f(8),
            time: f(0x88),
            speed,
        };
        rc2k_core::screens::player_result::stage_result(
            &mut PlayerResultScreen::new(canvas),
            &mut |s: &mut PlayerResultScreen| draw_times_heading(&STAGE_TIMES, s.canvas),
            &p,
            crate::strings::append_time::show_hours(),
        )?;
        Ok(())
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_player_stage_result(canvas) }?;
    Ok(Left::nothing())
}

/// What is compared: the two buffers, not the measuring font. The Rust version is compared
/// on a `RecordingCanvas` (in the self-test and in shadow mode), which leaves fonts alone,
/// while the original's measuring writes its count at the font's +0x14: compared, the font
/// mismatched on the first call after something had put a pointer back there (once a
/// session, 2026-10-20).
fn drawn_state() -> Vec<u8> {
    capture(&[
        (line_buffer() as u32, 0x100),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &drawn_state);
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
    // The measuring leaves its count in the font at 0x51E624's palette (+0x14), a pointer
    // until then: put back per case (left changed until 2026-10-18).
    let regions: Vec<Region> = vec![at(rva(0x75598C)), (game_addr(rva(0x51E624)) as u32, 0x20)];
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
            &|| unsafe {
                restore(&regions, &saved);
                game_write(rva(0x75598C), player)
            },
            &rust,
            &drawn_state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
