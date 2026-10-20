//! `screens::draw_highlighted_caption` (0x4695C6): the name of the highlighted entry of a
//! menu whose entries are turning objects, on a button at the bottom of the screen.
//!
//! In:  ecx (the x the caption is measured with)
//! Out: every register preserved; carry of the text drawn
//! `dword_75AA74` says which menu (0 .. 6; anything else: nothing is drawn) and so
//! which list of string numbers; the highlighted item picks the string, which is put
//! at `byte_A8BE54`, where string 0x1004 is read from. The button is as wide as the
//! caption measured with the font at 0x51E824 plus 12, centred on x = 0x197 at y =
//! 0x1FE: sprite 0xF, pieces of 32 of sprite 0xB (the last one drawn over the one
//! before, so that the width comes out), sprite 0x10; their palette is what the
//! measuring leaves in ebx, the caption's number of lines. A width of 0 is fatal error
//! 0x80701010. Then the caption at (0x195, 0x1FB) in the font blended between those at
//! 0x51E964 and 0x51E844 by the time since the last key.
//! Leaves the button's work variables (0x7559AC .. 0x7559C4), its two ends in
//! 0x755A20 .. 0x755A2C, and `g_markerX` / `g_markerY`.
//! In 12 lists (the main menu's among them: its menu is number 0).
//! Refuses an item of 0 or above 32, a caption of 0x100 bytes or more, and a caption
//! narrower than 20 (the original then draws 2^32 middle pieces).
//! Screen: not confirmed; the name is a reading of the code.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_marker_pair::{MARKER_X_RVA, MARKER_Y_RVA, sprite_table};
use crate::menu::update_blink::LAST_KEY_TIME_RVA;
use crate::menu::update_frame_clock::CLOCK_RVA;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw_rewound};
use crate::screens::{font_at, font_is_loaded};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::MENU_ITEM_RVA;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::blend_font::blend_regions;
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_string_by_id::measuring_unavailable;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_highlighted_caption",
    rva(0x4695C6),
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

const MENU_RVA: usize = rva(0x75AA74); // dword_75AA74
/// The lists of string numbers, by menu.
const LISTS: [usize; 7] = [
    0x758A12, 0x758A2A, 0x758A86, 0x758AA2, 0x758AB2, 0x758AC2, 0x758AE2,
];
const CAPTION_RVA: usize = rva(0xA8BE54); // byte_A8BE54
const CAPTION_LEN: usize = 0x100;
const CAPTION_STRING: u32 = 0x1004;
/// seconds, dword_7559B0, dword_7559B4, glyph, dword_7559C4.
const WIDTH_RVA: usize = rva(0x7559AC);
const X_RVA: usize = rva(0x7559B0);
const Y_RVA: usize = rva(0x7559B4);
const LEFT_RVA: usize = rva(0x7559C0);
const RIGHT_RVA: usize = rva(0x7559C4);
/// dword_755A20 .. dword_755A2C: where the two ends were drawn.
const ENDS_RVA: usize = rva(0x755A20);
const MEASURING_FONT: usize = 0x51E824;
const FONT_A: usize = 0x51E964;
const FONT_B: usize = 0x51E844;
const ITEMS: u32 = 32;

// ---- Rust version ----

pub unsafe fn draw_highlighted_caption(
    measured_at: u32,
    canvas: &mut dyn Canvas,
) -> Result<Option<bool>, Refused> {
    unsafe {
        let menu: u32 = game_read(MENU_RVA);
        let Some(&list) = LISTS.get(menu as usize) else {
            return Ok(None);
        };
        let item: u32 = game_read(MENU_ITEM_RVA);
        if !(1..=ITEMS).contains(&item) {
            return Err(BadInput("a highlighted item outside the lists of captions").into());
        }
        let id: u32 = game_read(rva(list) + (item as usize - 1) * 4);
        let caption = string_text(id)?;
        if caption.len() > CAPTION_LEN {
            return Err(BadInput("a caption longer than its place").into());
        }
        std::ptr::copy_nonoverlapping(
            caption.as_ptr(),
            game_addr(CAPTION_RVA) as *mut u8,
            caption.len(),
        );
        // Converted: `rc2k_core::screens::highlighted_caption`.
        let since_key = game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(LAST_KEY_TIME_RVA));
        let too_wide = rc2k_core::screens::highlighted_caption::highlighted_caption(
            &mut crate::screens::game_screen::GameScreen::<
                rc2k_core::screens::highlighted_caption::Font,
                rc2k_core::screens::highlighted_caption::Work,
            >::new(canvas),
            measured_at as i32,
            since_key,
        )?;
        Ok(Some(too_wide))
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    [
        &[
            at(WIDTH_RVA),
            at(X_RVA),
            at(Y_RVA),
            at(LEFT_RVA),
            at(RIGHT_RVA),
            (game_addr(ENDS_RVA) as u32, 16),
            at(MARKER_X_RVA),
            at(MARKER_Y_RVA),
            (game_addr(CAPTION_RVA) as u32, CAPTION_LEN),
            (text() as u32, TEXT_LEN),
        ][..],
        &blend_regions(),
    ]
    .concat()
}

fn state() -> Vec<u8> {
    capture(&regions())
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(match unsafe { draw_highlighted_caption(r.ecx, canvas) }? {
        Some(carry) => Left::carry(carry),
        None => Left::nothing(),
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw_rewound(&HOOK, r, call_original, &rust, &state, &blend_regions());
}

// ---- self-test: each menu's list with the game's strings and fonts ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable().or_else(measuring_unavailable) {
        return t.skip(why);
    }
    if ![MEASURING_FONT, FONT_A, FONT_B]
        .into_iter()
        .all(font_is_loaded)
        || sprite_table() == 0
        || unsafe { string_text(CAPTION_STRING) }.is_err()
    {
        return t.skip("the fonts, the sprites or the caption's string are not there");
    }
    let regions = regions();
    // The measuring leaves the widest line in the measuring font's box width, which the
    // menus' buttons are measured with: left changed, the stage selection's selectors were
    // drawn white (2026-10-18, found with `selftest screens:: leftovers`).
    let inputs = [
        at(MENU_RVA),
        at(MENU_ITEM_RVA),
        (font_at(MEASURING_FONT) as u32, size_of::<Font>()),
    ];
    let saved = (capture(&regions), capture(&inputs));
    for (menu, item) in [
        (0u32, 1u32),
        (0, 3),
        (0, 5),
        (1, 1),
        (1, 6),
        (2, 2),
        (3, 1),
        (4, 2),
        (5, 4),
        (6, 1),
        (6, 4),
        (7, 1),
        (0xFFFF_FFFF, 1),
    ] {
        check_draw(
            &mut t,
            &format!("menu {menu:#x}, item {item}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved.0);
                restore(&inputs, &saved.1);
                game_write(MENU_RVA, menu);
                game_write(MENU_ITEM_RVA, item);
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
