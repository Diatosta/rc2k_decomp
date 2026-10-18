//! `screens::draw_players_results_points` (0x4836CF): the players' stage results with their points: heads, the stage, and per player (three rows when `dword_755984` is 3, else four: two players get four) name, top speed, car, time or the gap to the first, flag, place and points (see `screens/players_results.rs`, `WITH_POINTS`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E8E). The sprites' palettes are ebx as the original leaves
//! it. Refuses (before anything) a row's offset past the four, a row's player without its
//! record or name or points, no stage records, a stage line past the line buffer, and a
//! speed `screens::draw_spaced_number` refuses. Compared in shadow mode (`dispatch_draw`;
//! the line buffer, `byte_A8BC54`, `dword_7559B8` / `dword_7559BC` as state). Self-test:
//! player counts 2, 3 and 4, two orders, the times as they are and with "no time" (0x36EE80
//! and above) in the first and third rows, nothing drawn (skipped without the fonts or four
//! players).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::player::player_at;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::line_buffer;
use crate::screens::players_results::{NO_TIME, WITH_POINTS, draw_players_results, refuses};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_players_results_points",
    rva(0x4836CF),
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

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_players_results(&WITH_POINTS, canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (line_buffer() as u32, 0x100),
        (game_addr(rva(0xA8BC54)) as u32, 0x100),
        (game_addr(rva(0x7559B8)) as u32, 8),
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
    if (0..4).any(|n| player_at(n) == 0) {
        return t.skip("fewer than four player records");
    }
    let mut regions: Vec<Region> = vec![
        at(rva(0x755984)),
        (game_addr(rva(0x758720)) as u32, 0x10),
        (game_addr(rva(0xA8BC54)) as u32, 0x100),
        (game_addr(rva(0x7559B8)) as u32, 8),
    ];
    for n in 0..4 {
        regions.push((player_at(n) + 0x88, 4));
    }
    let saved = capture(&regions);
    let times: [Option<[u32; 4]>; 3] = [
        None,
        Some([NO_TIME + 5, 7000, NO_TIME, 6500]),
        Some([5000, NO_TIME, 6000, NO_TIME]),
    ];
    for (count, order) in [
        (2u32, [0u32, 4, 8, 12]),
        (3, [8, 0, 4, 12]),
        (4, [12, 8, 4, 0]),
    ] {
        for (s, set) in times.iter().enumerate() {
            let apply = || unsafe {
                game_write(rva(0x755984), count);
                game_write(rva(0x758720), order);
                if let Some(v) = set {
                    for (n, x) in v.iter().enumerate() {
                        ((player_at(n as u32) + 0x88) as *mut u32).write_unaligned(*x);
                    }
                }
            };
            apply();
            if unsafe { refuses(&WITH_POINTS) } {
                restore(&regions, &saved);
                continue;
            }
            check_draw(
                &mut t,
                &format!("count {count} order {order:?} times {s}"),
                call_original,
                Regs::SENTINEL,
                &apply,
                &rust,
                &state,
            );
            restore(&regions, &saved);
        }
    }
    restore(&regions, &saved);
    t.finish()
}
