//! `screens::draw_menu_labels` (0x471EF0): draws the fixed labels of a menu.
//!
//! In:  edi = the menu
//! Out: every register preserved; carry clear
//! The menu's first dword points at a list of labels, 0x10 bytes each: x, y, font, string id. The
//! list ends with -1, but the first label is drawn without being checked for it. Each is drawn with
//! `draw_string_by_id`. One caller, the menu's drawing step `sub_4746B7`. Refuses a menu without a
//! label list, a list without an end in its first 256 labels, and a label whose font
//! field is off the lists' grid (the fonts are handles: `menus_store::label_font`).

use crate::errors::{BadInput, Refused};
use crate::text::draw_text::Canvas;

const LABEL_LEN: u32 = 0x10;
const END: u32 = u32::MAX;
const LABEL_LIMIT: u32 = 256;

fn dword(at: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(at).read::<u32>("a game value")
}

// ---- Rust version ----

/// Whether a label among the first `LABEL_LIMIT` after the first has the end mark.
fn has_end(labels: u32) -> Result<bool, BadInput> {
    for n in 1..=LABEL_LIMIT {
        if dword(labels + n * LABEL_LEN)? == END {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn draw_menu_labels(
    menu: crate::menu::menus_store::MenuId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let labels = menu.labels().get();
    if labels == 0 {
        return Err(BadInput("a menu without a label list").into());
    }
    if !has_end(labels)? {
        return Err(BadInput("a label list without an end").into());
    }
    // Converted: `rc2k_core::screens::menus::labels` (the list read here: the first label
    // drawn without being checked for the end, as the original).
    use crate::screens::game_screen::{FontAt, MenuScreen};
    let mut list = Vec::new();
    let mut label = labels;
    loop {
        list.push(rc2k_core::screens::menus::Label {
            x: dword(label)? as i32,
            y: dword(label + 4)? as i32,
            font: FontAt(crate::menu::menus_store::label_font(label + 8)?.addr()),
            id: dword(label + 0xC)?,
        });
        label += LABEL_LEN;
        if dword(label)? == END {
            break;
        }
    }
    rc2k_core::screens::menus::labels(&mut MenuScreen::new(canvas), &list)
}
