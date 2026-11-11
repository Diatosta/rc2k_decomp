//! `screens::draw_menu_items` (0x471D0A): draws the items of a menu, each a picture or a
//! button with a text, the highlighted one with pulsing colours.
//!
//! Takes the menu; the first and third values the original keeps in registers are handed on as
//! they come (see below).
//! Items are 0x5C bytes after a 4-byte head and end with -1; the first is drawn without
//! being checked for the end. An item is the highlighted one when its type (+0x10) equals
//! `loop_flags.item`; `draw_work.drawing_highlighted_item` is left at 1 or 0 for the last item accordingly.
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
//! of the item before, or the caller's ecx. Neither is looked at (the sprites are direct
//! colour, a measured width does not depend on its x), so neither is passed here.
//!
//! The text (string +0x14, `draw_menu_item_text` with the spacing flag at +0x58) goes at the
//! item's +0 and +4 minus 3 for the three button kinds (4 further left for the right-aligned
//! one), else at +8 and +0xC; the place is left in `g_markerX` and `g_markerY`. Its font is
//! the one at +0x18; for the highlighted item it is `text::blend_font` of that one and the
//! one at +0x50, by the time since the last key (0 while `option_work.redefining_key` is 1).
//!
//! One caller, `menu::draw_menu`. Refuses a menu without an end in its first 256 items.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::system::owner::system;
use crate::text::draw_text::Canvas;

// ---- game data ----

// `draw_work.drawing_highlighted_item`: 1 while the highlighted item is drawn

// What an item's +0x1C says goes behind its text.

const END: u32 = u32::MAX;
const ITEM_LIMIT: u32 = 256;

pub fn draw_menu_items(
    menus: &Menus,
    menu: crate::menu::menus_store::MenuId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let all = menu.items();
    if all.len() > ITEM_LIMIT as usize {
        return Err(BadInput("a menu without an end").into());
    }
    // Converted: `rc2k_core::screens::menus::items` (the items read here: the first
    // drawn without being checked for the end, as the original).
    use crate::screens::game_screen::{MenuFont, MenuScreen};
    use rc2k_core::screens::menus::Item;
    // The first item, then each until one whose x is the end mark (the -1 after the
    // menu's last item is one).
    let shown = 1 + all[1..]
        .iter()
        .take_while(|i| i.fixed().x as u32 != END)
        .count();
    let list: Vec<Item<MenuFont>> = all[..shown]
        .iter()
        .map(|i| Item {
            x: i.fixed().x as u32,
            y: i.fixed().y as u32,
            text_x: i.fixed().width_or_text_x as u32,
            text_y: i.fixed().alignment_or_text_y as u32,
            kind: i.fixed().id,
            id: i.string.get(),
            font: MenuFont::Object(i.font.get()),
            pulse_font: MenuFont::Object(i.highlight_font.get()),
            behind: i.behind.get(),
            sprite: i.sprites.get().raw(),
            spaced: i.fixed().spaced == 1,
        })
        .collect();
    let since_key = if menus.screens.option_work.redefining_key.get() == 1 {
        0
    } else {
        system()
            .session
            .clocks
            .frame
            .get()
            .wrapping_sub(menus.play.champ_view.last_key_time.get())
    };
    let mut screen = MenuScreen::new(canvas);
    screen.menu_items = all;
    let line = &crate::screens::text_buffers_store::TEXT_BUFFERS.line;
    rc2k_core::screens::menus::items(
        &mut screen,
        &list,
        menus.screens.loop_flags.item.get(),
        since_key,
        &|n| line.get(n),
    )
}
