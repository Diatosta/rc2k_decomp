//! `screens::draw_visual_preferences` (0x47A4C7): the values on the visual preferences page.
//!
//! In:  nothing
//! Out: every register preserved; carry of the last text
//!
//! For the current player (`dword_75598C`), at x = 0x1D1 from y = 0xC8 on every 0x1D, in the
//! font at 0x51E8C4: string 0x56B + the setting ("não", "sim") for the progress bar, the
//! driver's information, the pace notes, the rear mirror, the message bar, the rev
//! counter, the car's information and the local map, then string 0x373 + the units' setting
//! ("imperial", "métrico"). Each id is stored in `dword_7559EC` first. In the draw list
//! 0x758CAA (`menu::visual_preferences_loop`).
//! Refuses no current player and what `draw_string_by_id` refuses.
//! Compared in shadow mode (`dispatch_draw`). Self-test: the four players with the game's
//! settings and three patterns, nothing drawn.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::player::{PLAYERS, Player, player_at};
use crate::menu::player_settings::current;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw, push_dwords};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::PLAYER_INDEX_RVA;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_visual_preferences",
    rva(0x47A4C7),
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

// ---- game data ----

const ID_RVA: usize = rva(0x7559EC); // dword_7559EC
const NO: u32 = 0x56B;
const IMPERIAL: u32 = 0x373;

// ---- Rust version ----

/// The rows, in the order drawn: the setting and the string it counts from.
fn rows(p: &Player) -> [(u32, u32); 9] {
    [
        (p.progress_bar, NO),
        (p.driver_info, NO),
        (p.pace_notes, NO),
        (p.rear_mirror, NO),
        (p.message_display, NO),
        (p.rev_counter, NO),
        (p.car_info, NO),
        (p.map_area, NO),
        (p.metric_units, IMPERIAL),
    ]
}

/// Returns the carry.
pub unsafe fn draw_visual_preferences(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::settings_pages::visual_preferences`.
    let player = unsafe { current()?.read_unaligned() };
    let ids = rows(&player).map(|(value, first)| value.wrapping_add(first));
    rc2k_core::settings_pages::visual_preferences(
        &mut crate::screens::game_screen::SettingsScreen::new(canvas),
        ids,
    )
}

// ---- hook ----

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_visual_preferences(canvas) }?))
}

/// What it leaves behind: the last id and the text buffer the strings go through.
fn state() -> Vec<u8> {
    let mut state = capture(&[(text() as u32, TEXT_LEN)]);
    push_dwords(&mut state, &[ID_RVA]);
    state
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let players: Vec<Region> = (0..PLAYERS)
        .map(player_at)
        .filter(|&p| p != 0)
        .map(|p| (p, std::mem::size_of::<Player>()))
        .collect();
    let regions = [at(PLAYER_INDEX_RVA), at(ID_RVA), (text() as u32, TEXT_LEN)];
    let saved = (capture(&players), capture(&regions));
    for index in 0..PLAYERS {
        let p = player_at(index);
        if p == 0 {
            continue;
        }
        let fresh = unsafe { (p as *const Player).read_unaligned() };
        for pattern in [
            None,
            Some([0u32; 9]),
            Some([1; 9]),
            Some([1, 0, 1, 0, 1, 0, 1, 0, 0]),
        ] {
            check_draw(
                &mut t,
                &format!("player {index}, settings {pattern:?}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    game_write(PLAYER_INDEX_RVA, index);
                    let mut player = fresh;
                    if let Some(v) = pattern {
                        player.map_area = v[0];
                        player.rear_mirror = v[1];
                        player.rev_counter = v[2];
                        player.car_info = v[3];
                        player.message_display = v[4];
                        player.progress_bar = v[5];
                        player.driver_info = v[6];
                        player.pace_notes = v[7];
                        player.metric_units = v[8];
                    }
                    (p as *mut Player).write_unaligned(player);
                },
                &rust,
                &state,
            );
        }
    }
    restore(&players, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
