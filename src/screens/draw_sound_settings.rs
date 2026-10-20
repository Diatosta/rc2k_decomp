//! `screens::draw_sound_settings` (0x47022A): the sound page's items 4 .. 6 (3D sound, EAX,
//! A3D 2) choosable only when the card has them (`g_audioOptions.has_3d`, `has_eax`,
//! `has_a3d2`), and the six settings' names (strings 0x54B + quality, 0x554 + channels,
//! 0x56D + DS3D, EAX, A3D, Dolby; each kept in `dword_7559EC`) at x 0x1D1, y 0x10B and on
//! every 0x1B, in the font at 0x51E8C4.
//!
//! In:  nothing
//! Out: as the last `draw_string_by_id` leaves them (carry, eax 0, ebx, ecx, edx)
//!
//! In a menu loop's draw list (0x758C96). Refuses what `menu::enable_item` refuses (no
//! current menu), before anything. Compared in shadow mode (`dispatch_draw`; the items and
//! `dword_7559EC` as state). Self-test: the page's work values and the card's switches
//! varied, nothing drawn (skipped without the fonts or a menu).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::disable_item::{disable_item, item_region};
use crate::menu::enable_item::enable_item;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_sound_settings",
    rva(0x47022A),
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

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

/// (the switch, its item) and (the setting, its first string).
const SWITCHES: [(u32, u32); 3] = [(0x506CBC, 4), (0x506CC4, 5), (0x506CC0, 6)];
const NAMES: [(u32, u32); 6] = [
    (0x75566C, 0x54B),
    (0x755680, 0x554),
    (0x755670, 0x56D),
    (0x755674, 0x56D),
    (0x755678, 0x56D),
    (0x75567C, 0x56D),
];

/// The Rust version: the registers the last name leaves.
pub unsafe fn draw_sound_settings(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    // Converted: `rc2k_core::screens::settings_pages::{sound_items, sound}`.
    use crate::screens::game_screen::{SettingsScreen, left_of};
    let [a, b, c] = SWITCHES.map(|(switch, _)| k(switch) == 1);
    for (item, on) in rc2k_core::screens::settings_pages::sound_items(a, b, c) {
        if on {
            enable_item(item)?;
        } else {
            disable_item(item)?;
        }
    }
    let ids = NAMES.map(|(setting, first)| k(setting).wrapping_add(first));
    let out = rc2k_core::screens::settings_pages::sound(&mut SettingsScreen::new(canvas), ids)?;
    Ok(left_of(out))
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_sound_settings(canvas) }
}

fn state() -> Vec<u8> {
    let mut regions: Vec<Region> = vec![(text() as u32, TEXT_LEN), at(rva(0x7559EC))];
    for n in 4..=6 {
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
    // Only looked up: enabling it here, before the capture, left the current menu changed
    // (the stage selection's items, 2026-10-18).
    if crate::menu::item_at::item_at(4).is_err() {
        return t.skip("no current menu");
    }
    let mut kept: Vec<Region> = vec![
        (game_addr(rva(0x75566C)) as u32, 0x18),
        (game_addr(rva(0x506CBC)) as u32, 0xC),
        at(rva(0x7559EC)),
    ];
    for n in 4..=6 {
        kept.extend(item_region(n));
    }
    let saved = capture(&kept);
    for (seed, switches) in [(0u32, [1u32, 1, 1]), (1, [0, 1, 0]), (2, [1, 0, 2])] {
        check_draw(
            &mut t,
            &format!("seed {seed} switches {switches:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x75566C), [seed, 0, 1, seed & 1, 0]);
                game_write(rva(0x755680), seed);
                game_write(rva(0x506CBC), switches);
            },
            &rust,
            &state,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
