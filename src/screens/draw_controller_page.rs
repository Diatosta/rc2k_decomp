//! `screens::draw_controller_page` (0x46A454): the player's controller settings: the device,
//! its mode and strength, the sensitivity, two sliders and the stick's place now (see
//! `screens/controller.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Its other chunks are the shared fatal reporter (0x40763C) and its stub (0x4D9450). In four
//! controller pages' draw lists (0x758D8E ..). Refuses (before anything) no current player
//! record, or no input device for a player that has one. Compared in shadow mode
//! (`dispatch_draw`; the items, the clip and the work variables as state). Self-test: each
//! player, no controller and the first, slider values, nothing drawn (skipped without the
//! fonts, a menu or an input device).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::disable_item::item_region;
use crate::screens::controller::draw_controller_page;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_controller_page",
    rva(0x46A454),
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
    unsafe { draw_controller_page(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut regions: Vec<Region> = vec![
        (crate::screens::line_buffer() as u32, 0x20),
        g(0x7559AC, 0x48),
        g(0x755A0C, 4),
        g(0x4F503C, 8),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ];
    for n in 3..=8 {
        regions.extend(item_region(n));
    }
    capture(&regions)
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
    if crate::menu::enable_item::enable_item(3).is_err() {
        return t.skip("no current menu");
    }
    if unsafe { game_read::<u32>(rva(0x4F6EE8)) } == 0 {
        return t.skip("no input device");
    }
    let mut regions: Vec<Region> = vec![at(rva(0x75598C))];
    for n in 3..=8 {
        regions.extend(item_region(n));
    }
    for p in 0..crate::menu::player::PLAYERS {
        let a = crate::menu::player::player_at(p);
        if a != 0 {
            regions.push((a + 0x2C, 0x20));
        }
    }
    let saved = capture(&regions);
    for player in 0..crate::menu::player::PLAYERS {
        let p = crate::menu::player::player_at(player);
        if p == 0 {
            continue;
        }
        for (device, sliders, strength) in [
            (0u32, (0u32, 0u32), 0u32),
            (1, (50, 30), 75),
            (1, (0, 100), 0),
        ] {
            check_draw(
                &mut t,
                &format!("player {player} device {device} sliders {sliders:?} strength {strength}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &saved);
                    game_write(rva(0x75598C), player);
                    let w = |o: u32, v: u32| ((p + o) as *mut u32).write_unaligned(v);
                    w(0x2C, device);
                    w(0x40, sliders.0);
                    w(0x44, sliders.1);
                    w(0x48, strength);
                },
                &rust,
                &state,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
