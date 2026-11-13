//! Draws the fixed labels of a menu: a list of x, y, font and string id, ended by -1 (the first
//! is drawn without being checked for it). Refuses a menu without a label list, a list without an
//! end in its first 256 labels and a label whose font is off the lists' grid (fonts are handles:
//! `menus_store::label_font`).

use crate::errors::{BadInput, Refused};
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
    o: crate::owners::Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let (Some(labels), Some(first)) = (
        menu.labels_place()?,
        crate::menu::menus_store::labels_slot(menu, 0)?,
    ) else {
        return Err(BadInput("a menu without a label list").into());
    };
    // The dword `at` bytes into the list (bounded to the label lists).
    let dword = |at: u32| menus.tables.menus.labels.dword(first + at as usize);
    if !has_end(&dword)? {
        return Err(BadInput("a label list without an end").into());
    }
    // The logic is in `rc2k_core::screens::menus::labels` (the first label is drawn without
    // being checked for the end).
    use crate::screens::game_screen::{MenuFont, MenuScreen};
    let mut list = Vec::new();
    let mut label = 0;
    loop {
        list.push(rc2k_core::screens::menus::Label {
            x: dword(label)? as i32,
            y: dword(label + 4)? as i32,
            font: MenuFont::Object(crate::menu::menus_store::label_font(
                &menus.tables.menus.labels,
                labels + label + 8,
            )?),
            id: dword(label + 0xC)?,
        });
        label += LABEL_LEN;
        if dword(label)? == END {
            break;
        }
    }
    rc2k_core::screens::menus::labels(&mut MenuScreen::new(canvas, o), &list)
}
