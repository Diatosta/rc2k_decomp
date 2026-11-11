//! `screens::draw_menu_labels` (0x471EF0): draws the fixed labels of a menu.
//!
//! Takes the menu.
//! The menu's first dword points at a list of labels, 0x10 bytes each: x, y, font, string id. The
//! list ends with -1, but the first label is drawn without being checked for it. Each is drawn with
//! `draw_string_by_id`. One caller, the menu's drawing step `menu::draw_menu`. Refuses a menu
//! without a label list, a list without an end in its first 256 labels, and a label whose font
//! field is off the lists' grid (the fonts are handles: `menus_store::label_font`).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;

const LABEL_LEN: u32 = 0x10;
const END: u32 = u32::MAX;
const LABEL_LIMIT: u32 = 256;

/// Whether a label among the first `LABEL_LIMIT` after the first has the end mark.
fn has_end(dword: &impl Fn(u32) -> Result<u32, BadInput>) -> Result<bool, BadInput> {
    for n in 1..=LABEL_LIMIT {
        if dword(n * LABEL_LEN)? == END {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn draw_menu_labels(
    menus: &Menus,
    menu: crate::menu::menus_store::MenuId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let (Some(labels), Some(first)) = (
        menu.labels_place()?,
        crate::menu::menus_store::labels_slot(menus, menu, 0)?,
    ) else {
        return Err(BadInput("a menu without a label list").into());
    };
    // The dword `at` bytes into the list (bounded to the menus' store).
    let dword = |at: u32| first.offset(at).read::<u32>("a menu label");
    if !has_end(&dword)? {
        return Err(BadInput("a label list without an end").into());
    }
    // Converted: `rc2k_core::screens::menus::labels` (the list read here: the first label
    // drawn without being checked for the end, as the original).
    use crate::screens::game_screen::{MenuFont, MenuScreen};
    let mut list = Vec::new();
    let mut label = 0;
    loop {
        list.push(rc2k_core::screens::menus::Label {
            x: dword(label)? as i32,
            y: dword(label + 4)? as i32,
            font: MenuFont::Object(crate::menu::menus_store::label_font(labels + label + 8)?),
            id: dword(label + 0xC)?,
        });
        label += LABEL_LEN;
        if dword(label)? == END {
            break;
        }
    }
    rc2k_core::screens::menus::labels(&mut MenuScreen::new(canvas), &list)
}
