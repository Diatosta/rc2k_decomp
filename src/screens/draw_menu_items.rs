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
//! 0xFFFFFFA9  nothing
//! 0xFFFFFFA5  the button the item describes (`draw_item_button`)
//! 0xFFFFFFA7  a right-aligned button as wide as the text
//! 0xFFFFFFA6  a centred one
//! 0xFFFFFFAA  nothing (only looked at when the item is not highlighted)
//! other       the sprite at +0x20 of `g_spriteTable`, at the item's +0 and +4
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
//! One caller, `menu::draw_menu`. Refuses a menu without an end in its first 256 items.

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::session::CLOCKS;
use crate::text::draw_text::Canvas;

// ---- game data ----

// dword_7559D0: 1 while the highlighted item is drawn

// What an item's +0x1C says goes behind its text.

const END: u32 = u32::MAX;
const ITEM_LIMIT: u32 = 256;

// ---- Rust version ----

/// `ebx` and `ecx` are those registers at entry.
pub fn draw_menu_items(
    menu: crate::menu::menus_store::MenuId,
    ebx: u32,
    ecx: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let all = menu.items();
    if all.len() > ITEM_LIMIT as usize {
        return Err(BadInput("a menu without an end").into());
    }
    // Converted: `rc2k_core::screens::menus::items` (the items read here: the first
    // drawn without being checked for the end, as the original).
    use crate::screens::game_screen::{FontAt, MenuScreen};
    use rc2k_core::screens::menus::Item;
    // The first item, then each until one whose x is the end mark (the -1 after the
    // menu's last item is one).
    let shown = 1 + all[1..].iter().take_while(|i| i.x.get() != END).count();
    let list: Vec<Item<FontAt>> = all[..shown]
        .iter()
        .map(|i| Item {
            x: i.x.get(),
            y: i.y.get(),
            text_x: i.text_x.get(),
            text_y: i.text_y.get(),
            kind: i.id.get(),
            id: i.string.get(),
            font: FontAt(i.font.get()),
            pulse_font: FontAt(i.highlight_font.get()),
            behind: i.behind.get(),
            sprite: i.sprites.get(),
            spaced: i.spaced.get() == 1,
        })
        .collect();
    let since_key = if OPTION_WORK.redefining_key.get() == 1 {
        0
    } else {
        CLOCKS
            .frame
            .get()
            .wrapping_sub(CHAMP_VIEW.last_key_time.get())
    };
    let mut screen = MenuScreen::new(canvas);
    screen.menu_items = all;
    let line = &crate::screens::text_buffers_store::TEXT_BUFFERS.line;
    rc2k_core::screens::menus::items(
        &mut screen,
        &list,
        LOOP_FLAGS.item.get(),
        since_key,
        ebx,
        ecx,
        &|n| line.get(n),
    )
}
