//! `screens::draw_menu_items` (0x471D0A): draws the items of a menu, each a picture or a
//! button with a text, the highlighted one with pulsing colours.
//!
//! In:  edi = the menu; ebx and ecx are used as they come (see below)
//! Out: every register preserved
//! Items are 0x5C bytes after a 4-byte head and end with -1; the first is drawn without
//! being checked for the end. An item is the highlighted one when its type (+0x10) equals
//! `dword_755525`; `dword_7559D0` is left at 1 or 0 for the last item accordingly.
//!
//! What goes behind the text depends on the item's +0x1C:
//!   0xFFFFFFA9  nothing
//!   0xFFFFFFA5  the button the item describes (`draw_item_button`)
//!   0xFFFFFFA7  a right-aligned button as wide as the text
//!   0xFFFFFFA6  a centred one
//!   0xFFFFFFAA  nothing (only looked at when the item is not highlighted)
//!   other       the sprite at +0x20 of `g_spriteTable`, at the item's +0 and +4
//! A highlighted item gets the highlighted buttons, and nothing at all if its +0x20 is
//! 0xFFFFFFAA. The sprite's palette is the sprite number itself for a plain item and, for
//! a highlighted one, whatever ebx holds: the font of the item before, or the caller's ebx
//! for the first item. The measuring of the text buttons is given ecx as its x: the text x
//! of the item before, or the caller's ecx.
//!
//! The text (string +0x14, `draw_menu_item_text` with the spacing flag at +0x58) goes at the
//! item's +0 and +4 minus 3 for the three button kinds (4 further left for the right-aligned
//! one), else at +8 and +0xC; the place is left in `g_markerX` and `g_markerY`. Its font is
//! the one at +0x18; for the highlighted item it is `text::blend_font` of that one and the
//! one at +0x50, by the time since the last key (0 while `dword_7556AC` is 1).
//!
//! One caller, `menu::draw_menu`. Manual patch with a private copy (0x18F bytes, ten calls
//! relocated): the loop comes back to the fifth byte, inside what the patch overwrites.
//! Refuses a menu without an end in its first 256 items.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::{MARKER_X_RVA, MARKER_Y_RVA, sprite_table};
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use crate::menu::update_blink::{BLINK_OFF_RVA, LAST_KEY_TIME_RVA};
use crate::menu::update_frame_clock::CLOCK_RVA;
use crate::screens::buttons::{
    ALIGN_CENTRE, ALIGN_RIGHT, ALT, HIGHLIGHTED, PLAIN, draw_item_button, draw_text_button, dword,
};
use crate::screens::draw_menu_item_text::{
    TEST_FONT, draw_menu_item_text, item_text_regions, text_unavailable,
};
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw_rewound};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::MENU_ITEM_RVA;
use crate::text::blend_font::{BLENDED_FONT_RVA, blend_font, blend_regions};
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_string_by_id::{MEASURING_FONT, TEST_STRINGS, measuring_unavailable};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_menu_items", rva(0x471D0A), thunk, &ORIGINAL)
        .private_copy(0x18F)
        .relocate_calls(&[0x82, 0x8A, 0x91, 0x98, 0xBE, 0xC5, 0xCC, 0xE2, 0x15D, 0x17B])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const HIGHLIGHTED_RVA: usize = rva(0x7559D0); // dword_7559D0: 1 while the highlighted item is drawn

// What an item's +0x1C says goes behind its text.
const NOTHING: u32 = 0xFFFF_FFA9;
const OWN_BUTTON: u32 = 0xFFFF_FFA5;
const RIGHT_BUTTON: u32 = 0xFFFF_FFA7;
const CENTRED_BUTTON: u32 = 0xFFFF_FFA6;
const NOTHING_EITHER: u32 = 0xFFFF_FFAA;

const END: u32 = u32::MAX;
const ITEM_LIMIT: u32 = 256;
const HALF_PULSE: u32 = 0x100;

// ---- Rust version ----

/// `ebx` and `ecx` are those registers at entry.
pub unsafe fn draw_menu_items(
    menu: u32,
    mut ebx: u32,
    mut ecx: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        let first = menu.wrapping_add(MENU_HEAD_LEN);
        if menu == 0 || !(1..=ITEM_LIMIT).any(|n| dword(first + n * ITEM_LEN) == END) {
            return Err(BadInput("a menu without an end").into());
        }
        let mut item = first;
        loop {
            let (x, y) = (dword(item), dword(item + 4));
            let (behind, sprite) = (dword(item + 0x1C), dword(item + 0x20));
            let highlighted = game_read::<u32>(MENU_ITEM_RVA) == dword(item + 0x10);
            game_write(HIGHLIGHTED_RVA, highlighted as u32);

            if !highlighted {
                if behind != NOTHING {
                    game_write(MARKER_X_RVA, x);
                    game_write(MARKER_Y_RVA, y);
                }
                match behind {
                    NOTHING | NOTHING_EITHER => {}
                    OWN_BUTTON => draw_item_button(item, canvas)?,
                    RIGHT_BUTTON => draw_text_button(&ALT, ALIGN_RIGHT, item, ecx as i32, canvas)?,
                    CENTRED_BUTTON => {
                        draw_text_button(&PLAIN, ALIGN_CENTRE, item, ecx as i32, canvas)?
                    }
                    _ => canvas.blit_glyph(BlitCall::new(
                        sprite,
                        x as i32,
                        y as i32,
                        sprite,
                        sprite_table(),
                    ))?,
                }
            } else if sprite != NOTHING_EITHER {
                match behind {
                    NOTHING => {}
                    OWN_BUTTON => draw_item_button(item, canvas)?,
                    RIGHT_BUTTON => {
                        draw_text_button(&HIGHLIGHTED, ALIGN_RIGHT, item, ecx as i32, canvas)?
                    }
                    CENTRED_BUTTON => {
                        draw_text_button(&HIGHLIGHTED, ALIGN_CENTRE, item, ecx as i32, canvas)?
                    }
                    _ => canvas.blit_glyph(BlitCall::new(
                        sprite,
                        x as i32,
                        y as i32,
                        ebx,
                        sprite_table(),
                    ))?,
                }
            }

            let is_button = matches!(behind, OWN_BUTTON | RIGHT_BUTTON | CENTRED_BUTTON);
            let (mut text_x, text_y) = if is_button {
                (x, y.wrapping_sub(3))
            } else {
                (dword(item + 8), dword(item + 0xC))
            };
            if behind == RIGHT_BUTTON {
                text_x = text_x.wrapping_sub(4);
            }
            game_write(MARKER_X_RVA, text_x);
            game_write(MARKER_Y_RVA, text_y);

            let mut font = dword(item + 0x18);
            if highlighted {
                let since_key = if game_read::<u32>(BLINK_OFF_RVA) == 1 {
                    0
                } else {
                    game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(LAST_KEY_TIME_RVA))
                };
                blend_font(
                    since_key,
                    HALF_PULSE,
                    font as *const Font,
                    dword(item + 0x50) as *const Font,
                )?;
                font = game_addr(BLENDED_FONT_RVA) as u32;
            }
            draw_menu_item_text(
                dword(item + 0x14),
                font as *mut Font,
                text_x as i32,
                text_y as i32,
                dword(item + 0x58) == 1,
                canvas,
            )?;
            (ebx, ecx) = (font, text_x);

            item = item.wrapping_add(ITEM_LEN);
            if dword(item) == END {
                return Ok(());
            }
        }
    }
}

// ---- hook ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_menu_items(r.edi, r.ebx, r.ecx, canvas) }?;
    Ok(Left::nothing())
}

/// Everything the function and the ones under it leave behind, but the font objects that
/// measuring writes.
pub fn regions() -> Vec<Region> {
    [
        &[at(HIGHLIGHTED_RVA), at(MARKER_X_RVA), at(MARKER_Y_RVA)][..],
        &PLAIN.regions(),
        &ALT.regions(),
        &item_text_regions(),
        &blend_regions(),
    ]
    .concat()
}

pub fn state() -> Vec<u8> {
    capture(&regions())
}

extern "C" fn dispatch(r: &mut Regs) {
    // The blended font and its palette are put back before the original runs: the palette's
    // top bytes are carried over from one blend to the next.
    dispatch_draw_rewound(&HOOK, r, call_original, &rust, &state, &blend_regions());
}

// ---- self-test: a menu of its own with one item of each kind, each of them highlighted in
// turn, with the game's strings and fonts ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable().or_else(measuring_unavailable) {
        return t.skip(why);
    }
    let font = font_at(TEST_FONT);
    if unsafe { font.read_unaligned() }.palette == 0 {
        return t.skip("the test font has no palette to blend");
    }
    let measuring: *mut Font = game_ptr(rva(MEASURING_FONT));
    let all = [
        &regions()[..],
        &[at(MENU_ITEM_RVA), (measuring as u32, 0x20)],
    ]
    .concat();
    let saved = capture(&all);

    let sprites = [0x40u32, 0x41, 0x42, 0x43, 0x44];
    // (what is behind, +0x20, spaced)
    let kinds = [
        (NOTHING, 0, 0),
        (OWN_BUTTON, sprites.as_ptr() as u32, 0),
        (RIGHT_BUTTON, sprites.as_ptr() as u32, 0),
        (CENTRED_BUTTON, sprites.as_ptr() as u32, 0),
        (NOTHING_EITHER, 0, 0),
        (0, 0x45, 1),
        (5, NOTHING_EITHER, 0),
    ];
    let mut menu = vec![0xAAAA_AAAAu32; 1 + kinds.len() * 0x5C / 4 + 1];
    for (n, (behind, at_20, spaced)) in kinds.into_iter().enumerate() {
        let item = &mut menu[1 + n * 0x5C / 4..][..0x5C / 4];
        let n = n as u32;
        item[0] = 300 + 20 * n; // x
        item[1] = 60 + 50 * n; // y
        item[2] = if behind == OWN_BUTTON {
            70
        } else {
            120 + 20 * n
        }; // width, or the text's x
        item[3] = if behind == OWN_BUTTON { 2 } else { 64 + 50 * n }; // alignment, or the text's y
        item[4] = n + 1; // type
        item[5] = TEST_STRINGS[n as usize % 3];
        item[6] = font as u32;
        item[7] = behind;
        item[8] = at_20;
        item[0x4C / 4] = measuring as u32;
        item[0x50 / 4] = font as u32;
        item[0x58 / 4] = spaced;
    }
    *menu.last_mut().unwrap() = END;

    for highlighted in 0..=kinds.len() as u32 {
        let before = Regs {
            ebx: 0x1234,
            ecx: 7,
            edi: menu.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        check_draw(
            &mut t,
            &format!("highlighted item {highlighted}"),
            call_original,
            before,
            &|| {
                restore(&all, &saved);
                unsafe { game_write(MENU_ITEM_RVA, highlighted) };
            },
            &rust,
            &state,
        );
    }
    restore(&all, &saved);
    t.finish()
}
