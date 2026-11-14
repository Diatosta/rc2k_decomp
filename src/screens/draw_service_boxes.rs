//! The service area's boxes, the layer behind `screens::draw_service_area`'s text: the
//! backdrop, the header and item boxes of the inspection (colour by the item's flags, a blinking
//! yellow highlight on the item under the cursor) and last the four frames round the setup cells.
//!
//! The boxes are called "items" (`loop_flags.item`): 2 to 0xD the twelve small boxes in two
//! columns, 0xE and 0xF the setup rows, 0x10 to 0x13 the four tall boxes, 0x14, 0x15, 0x16 and 1
//! the four bars at the bottom, and 0x63 the headers and the bottom panel.

use crate::errors::Refused;
use crate::menu::lists_store::lists;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::owners::Owners;
use crate::screens::service_boxes::{BLACK, Outline, quad};

/// The colour of the backdrop.
const BACKDROP: u32 = 0x11166A;
/// A box with nothing to report, the dark inside of a box, a box with its first flag set, and
/// one with both flags set.
const BLUE: u32 = 0x4FCF;
const DARK: u32 = 0x1F1F1F;
const GREEN: u32 = 0x7F00;
const RED: u32 = 0x7F0000;

/// The item number of the headers and the bottom panel.
const HEADER_ITEM: u32 = 0x63;
/// The item number of the bar below the last one numbered 0x16.
const LAST_BAR_ITEM: u32 = 1;

/// The backdrop's rectangles: x, y, width, height.
const BACKDROP_BOXES: [[u32; 4]; 30] = [
    [0x0, 0x0, 0x320, 0x45],
    [0x0, 0x45, 0x43, 0x258],
    [0x2DE, 0x45, 0x42, 0x213],
    [0x43, 0x224, 0x11F, 0x34],
    [0x162, 0x224, 0x17C, 0x5],
    [0x162, 0x23C, 0x17C, 0x1C],
    [0x2A8, 0x229, 0x36, 0x13],
    [0x43, 0x5E, 0x29B, 0x3],
    [0x43, 0x7B, 0x14C, 0x3],
    [0x192, 0x7B, 0x14C, 0x3],
    [0x43, 0x98, 0x14C, 0x3],
    [0x192, 0x98, 0x14C, 0x3],
    [0x43, 0xB5, 0x14C, 0x3],
    [0x192, 0xB5, 0x14C, 0x3],
    [0x43, 0xD2, 0x14C, 0x3],
    [0x192, 0xD2, 0x14C, 0x3],
    [0x43, 0xEF, 0x14C, 0x3],
    [0x192, 0xEF, 0x14C, 0x3],
    [0x43, 0x10C, 0x14C, 0x3],
    [0x192, 0x10C, 0x14C, 0x3],
    [0x43, 0x15F, 0x29B, 0x3],
    [0x43, 0x17B, 0x29B, 0x3],
    [0x43, 0x1B3, 0x14C, 0x3],
    [0x192, 0x1B3, 0x14C, 0x3],
    [0x43, 0x1EB, 0x14C, 0x3],
    [0x192, 0x1EB, 0x14C, 0x3],
    [0x43, 0x207, 0x14C, 0x3],
    [0x192, 0x207, 0x14C, 0x3],
    [0x18F, 0x61, 0x3, 0xFE],
    [0x18F, 0x17E, 0x3, 0xA6],
];

/// The see-through value shared by the boxes.
fn see_through(screens: &Screens) -> u32 {
    screens.state.service_ui.box_colour_a.get()
}

/// A quad in the colour of the item being drawn, see-through.
fn shade(o: Owners<'static>, x: u32, y: u32, w: u32, h: u32) -> Result<(), Refused> {
    let screens = &o.menus.screens;
    quad(
        o,
        x,
        y,
        w,
        h,
        screens.state.service_ui.box_colour_b.get(),
        see_through(screens),
    )
}

/// A quad in the dark colour of the inside of a box, see-through.
fn dark(o: Owners<'static>, x: u32, y: u32, w: u32, h: u32) -> Result<(), Refused> {
    quad(o, x, y, w, h, DARK, see_through(&o.menus.screens))
}

/// Sets the colour of an item's boxes from its two inspection flags.
fn set_item_colour(menus: &Menus, item: u32) {
    let colour = if lists().inspection_flags_a[item as usize].get() != 1 {
        BLUE
    } else if lists().inspection_flags_b[item as usize].get() != 1 {
        GREEN
    } else {
        RED
    };
    menus.screens.state.service_ui.box_colour_b.set(colour);
}

/// Whether the cursor is on `item` and its highlight blinks on.
fn blinking(screens: &Screens, item: u32) -> bool {
    screens.loop_flags.item.get() == item
        && !screens.state.menus.service_flag.is_set()
        && screens.draw_work.blink.is_set()
}

/// Like [`blinking`], and also whenever the item's service box is turned over.
fn blinking_or_turned_over(screens: &Screens, item: u32) -> bool {
    screens.loop_flags.item.get() == item
        && (screens.state.service_ui.box_shown.is_set()
            || (!screens.state.menus.service_flag.is_set() && screens.draw_work.blink.is_set()))
}

/// A bar or panel: its filled box and the black outline round it, highlighted when the cursor
/// is on `item`.
fn panel(
    o: Owners<'static>,
    item: u32,
    [x, y, w, h]: [u32; 4],
    colour: u32,
) -> Result<(), Refused> {
    let screens = &o.menus.screens;
    quad(o, x, y, w, h, colour, see_through(screens))?;
    let outline = Outline::around(x, y, w, h);
    outline.draw(o, BLACK)?;
    if blinking(screens, item) {
        outline.draw_highlighted(o)?;
    }
    Ok(())
}

/// The left (even item) or right (odd item) column's x of a box.
fn column_x(item: u32) -> u32 {
    if item.is_multiple_of(2) { 0x42 } else { 0x191 }
}

/// One of the twelve small boxes, items 2 .. 0xD: a label cell and a wide cell.
fn small_item(o: Owners<'static>, item: u32) -> Result<(), Refused> {
    let menus = o.menus;
    let x = column_x(item);
    let y = 0x60 + 0x1D * ((item - 2) / 2);
    set_item_colour(menus, item);
    shade(o, x, y, 0xAA, 0x1C)?;
    shade(o, x + 0xAA, y, 0xA4, 4)?;
    shade(o, x + 0x14A, y + 4, 4, 0x14)?;
    shade(o, x + 0xAA, y + 0x18, 0xA4, 4)?;
    dark(o, x + 0xAA, y + 4, 0xA0, 0x14)?;
    Outline::around(x, y, 0x24, 0x1C).draw(o, BLACK)?;
    let big = Outline::around(x, y, 0x14E, 0x1C);
    big.draw(o, BLACK)?;
    Outline::around(x + 0xA9, y + 3, 0xA2, 0x16).draw(o, BLACK)?;
    if blinking(&menus.screens, item) {
        big.draw_highlighted(o)?;
    }
    Ok(())
}

/// A row of setup cells under a label, item 0xE (five cells) or 0xF (six cells).
fn setup_row(o: Owners<'static>, item: u32) -> Result<(), Refused> {
    let menus = o.menus;
    let (cells, step, cell_w, label_h) = if item == 0xE {
        (5, 0x42, 0x40, 0x16)
    } else {
        (6, 0x37, 0x35, 0x1C)
    };
    let x = column_x(item);
    let y = 0x10E;
    set_item_colour(menus, item);
    shade(o, x, y, 0xAA, 0x1F)?;
    shade(o, x + 0xAA, y, 0xA4, 4)?;
    shade(o, x + 0x14A, y, 4, 0x45)?;
    shade(o, x + 0xAA, y + 0x18, 0xA1, 7)?;
    shade(o, x, 0x12C, 4, 0x27)?;
    shade(o, x, 0x152, 0x14E, 0xE)?;
    for n in 1..cells {
        shade(o, x + step * n, 0x12D, 4, 0x25)?;
    }
    // The label's inside is drawn after the cells' on the right, before them on the left.
    let label = |x: u32| dark(o, x + 0xA9, 0x111, 0xA2, label_h);
    if item == 0xE {
        label(x)?;
    }
    for n in 0..cells {
        dark(o, x + 3 + step * n, 0x12C, cell_w, 0x26)?;
    }
    if item != 0xE {
        label(x)?;
    }
    Outline::around(x, y, 0x24, 0x1C).draw(o, BLACK)?;
    let big = Outline::around(x, y, 0x14E, 0x52);
    big.draw(o, BLACK)?;
    Outline::around(x + 0xA9, y + 3, 0xA2, 0x16).draw(o, BLACK)?;
    for n in 0..cells {
        Outline::around(x + 3 + step * n, 0x12C, cell_w, 0x26).draw(o, BLACK)?;
    }
    if blinking_or_turned_over(&menus.screens, item) {
        big.draw_highlighted(o)?;
    }
    Ok(())
}

/// One of the four tall boxes, items 0x10 .. 0x13.
fn tall_item(o: Owners<'static>, item: u32) -> Result<(), Refused> {
    let menus = o.menus;
    let x = column_x(item);
    let y = 0x17D + 0x38 * ((item - 0x10) / 2);
    set_item_colour(menus, item);
    shade(o, x, y, 0x14E, 0x1F)?;
    shade(o, x + 0x14A, y + 0x1F, 4, 0x13)?;
    shade(o, x, y + 0x1F, 4, 0x13)?;
    shade(o, x, y + 0x32, 0x14E, 5)?;
    dark(o, x + 4, y + 0x1F, 0x146, 0x13)?;
    Outline::around(x, y, 0x24, 0x1C).draw(o, BLACK)?;
    let big = Outline::around(x, y, 0x14E, 0x37);
    big.draw(o, BLACK)?;
    Outline::around(x + 3, y + 0x1E, 0x148, 0x15).draw(o, BLACK)?;
    if blinking_or_turned_over(&menus.screens, item) {
        big.draw_highlighted(o)?;
    }
    Ok(())
}

/// The service area's boxes, in drawing order.
pub fn draw_service_boxes(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    for [x, y, w, h] in BACKDROP_BOXES {
        quad(
            o,
            x,
            y,
            w,
            h,
            BACKDROP,
            menus.screens.state.service_ui.fade_full.get(),
        )?;
    }
    panel(o, HEADER_ITEM, [0x42, 0x44, 0x29D, 0x1B], BLUE)?;
    for item in 2..=0xD {
        small_item(o, item)?;
    }
    setup_row(o, 0xE)?;
    setup_row(o, 0xF)?;
    panel(o, HEADER_ITEM, [0x42, 0x161, 0x29D, 0x1B], BLUE)?;
    for item in 0x10..=0x13 {
        tall_item(o, item)?;
    }
    panel(o, 0x14, [0x42, 0x1ED, 0x14E, 0x1B], BLUE)?;
    panel(o, 0x15, [0x191, 0x1ED, 0x14E, 0x1B], BLUE)?;
    panel(o, 0x16, [0x42, 0x209, 0x14E, 0x1B], BLUE)?;
    panel(o, LAST_BAR_ITEM, [0x191, 0x209, 0x14E, 0x1B], BLUE)?;
    panel(o, HEADER_ITEM, [0x161, 0x228, 0x148, 0x15], DARK)?;
    crate::screens::service_box_frame_1::service_box_frame_1(o)?;
    crate::screens::service_box_frame_2::service_box_frame_2(o)?;
    crate::screens::service_box_frame_3::service_box_frame_3(o)?;
    crate::screens::service_box_frame_4::service_box_frame_4(o)
}
