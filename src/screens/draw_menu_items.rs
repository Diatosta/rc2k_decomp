//! Draws the items of a menu, each a picture or a button with a text, the highlighted one with
//! pulsing colours. Items follow a 4-byte head, are 0x5C bytes each and end with -1; the first is
//! drawn without being checked for the end. An item is the highlighted one when its type equals
//! `loop_flags.item`; `draw_work.drawing_highlighted_item` is left at 1 or 0 for the last item.
//!
//! What goes behind the text depends on the item's kind: nothing, the button the item describes
//! (`draw_item_button`), a right-aligned button as wide as the text, a centred one, or a sprite of
//! the sprite table at the item's place. A highlighted item gets the highlighted buttons, and
//! nothing at all for the "nothing" kind. The sprites are direct colour, so the palette and the
//! text's x passed to the measuring do not matter.
//!
//! The text goes at the item's place (3 further left for the three button kinds, 4 for the
//! right-aligned one), which is left in the marker position; its font is the item's, blended
//! with its highlight font by the time since the last key (0 while a key is being redefined) for
//! the highlighted item. One caller, `menu::draw_menu`. Refuses a menu without an end in its first
//! 256 items.

use crate::errors::{BadInput, Refused};
use crate::menu::item_sprites_store::ItemSprite;
use crate::text::draw_text::Canvas;

// ---- game data ----

// `draw_work.drawing_highlighted_item`: 1 while the highlighted item is drawn

// What an item's +0x1C says goes behind its text.

const END: u32 = u32::MAX;
const ITEM_LIMIT: u32 = 256;

pub fn draw_menu_items(
    o: crate::owners::Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let all = menu.items(&menus.tables.menus);
    if all.len() > ITEM_LIMIT as usize {
        return Err(BadInput("a menu without an end").into());
    }
    // The logic is in `rc2k_core::screens::menus::items` (the first item is drawn without being
    // checked for the end).
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
            sprite: match i.sprites.get() {
                ItemSprite::Sprite(n) => rc2k_core::screens::menus::ItemSprite::Number(n),
                ItemSprite::List(n) => rc2k_core::screens::menus::ItemSprite::List(n),
            },
            spaced: i.fixed().spaced == 1,
        })
        .collect();
    let since_key = if menus.screens.option_work.redefining_key.is_set() {
        0
    } else {
        o.system
            .clocks()
            .frame
            .get()
            .wrapping_sub(menus.play.champ_view.last_key_time.get())
    };
    let mut screen = MenuScreen::new(canvas, o);
    screen.menu_items = all;
    let line = &o.race.text_buffers().line;
    rc2k_core::screens::menus::items(
        &mut screen,
        &list,
        menus.screens.loop_flags.item.get(),
        since_key,
        &|n| line.get(n),
    )
}
