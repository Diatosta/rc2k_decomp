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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::game_read;
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use crate::menu::update_blink::{BLINK_OFF_RVA, LAST_KEY_TIME_RVA};
use crate::menu::update_frame_clock::CLOCK_RVA;
use crate::screens::buttons::dword;
use crate::service::state::MENU_ITEM_RVA;
use crate::text::draw_text::Canvas;

// ---- game data ----

// dword_7559D0: 1 while the highlighted item is drawn

// What an item's +0x1C says goes behind its text.

const END: u32 = u32::MAX;
const ITEM_LIMIT: u32 = 256;

// ---- Rust version ----

/// `ebx` and `ecx` are those registers at entry.
pub unsafe fn draw_menu_items(
    menu: u32,
    ebx: u32,
    ecx: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        let first = menu.wrapping_add(MENU_HEAD_LEN);
        if menu == 0 || !(1..=ITEM_LIMIT).any(|n| dword(first + n * ITEM_LEN) == END) {
            return Err(BadInput("a menu without an end").into());
        }
        // Converted: `rc2k_core::screens::menus::items` (the items read here: the first
        // drawn without being checked for the end, as the original).
        use crate::screens::game_screen::{FontAt, MenuScreen};
        use rc2k_core::screens::menus::Item;
        let mut list = Vec::new();
        let mut item = first;
        loop {
            list.push(Item {
                x: dword(item),
                y: dword(item + 4),
                text_x: dword(item + 8),
                text_y: dword(item + 0xC),
                kind: dword(item + 0x10),
                id: dword(item + 0x14),
                font: FontAt(dword(item + 0x18)),
                pulse_font: FontAt(dword(item + 0x50)),
                behind: dword(item + 0x1C),
                sprite: dword(item + 0x20),
                spaced: dword(item + 0x58) == 1,
            });
            item = item.wrapping_add(ITEM_LEN);
            if dword(item) == END {
                break;
            }
        }
        let since_key = if game_read::<u32>(BLINK_OFF_RVA) == 1 {
            0
        } else {
            game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(LAST_KEY_TIME_RVA))
        };
        let mut screen = MenuScreen::new(canvas);
        screen.menu_items = first;
        let line: *const u8 = crate::framework::game_ptr(crate::screens::LINE_BUFFER_RVA);
        rc2k_core::screens::menus::items(
            &mut screen,
            &list,
            game_read(MENU_ITEM_RVA),
            since_key,
            ebx,
            ecx,
            &|n| *line.add(n),
        )
    }
}

// ---- self-test: a menu of its own with one item of each kind, each of them highlighted in
// turn, with the game's strings and fonts ----
