//! `screens::draw_car_figures` (0x47CEB4): the car specification's figures for the page
//! shown.
//!
//! In:  nothing
//! Out: every register preserved; carry of the last text drawn
//!
//! The page `dword_755998` picks the function from `funcs_47CED1` (at 0x759AE6), which
//! gets edi = the shown car's record (the entry `dword_755990` through `dword_75A65C`)
//! and eax = its first dword, the car. The four have this one caller and are inlined.
//! Labels are in the font at 0x51E5A4, figures in the one at 0x51E584; a figure line is
//! built in `g_lineBuffer`; a figure of 0 is string 0x578 ("n/d").
//! - 0 (`sub_469C40`): the description at (0x1A4, 0xD5): for car 0x21 the string at
//!   +0x70 (made 0x103F first when `g_noticeKind` is 3), else the seven strings from
//!   +0x70 on (`strings::text_from_ids`, the text's address kept in `dword_7559EC`).
//! - 1 (`sub_469F12`), x = 0x244: capacity +0xC and "cc"; type, the string +0x10 (0x103A
//!   for car 0x21 with notice kind 3); power +0x14 (`append_power`) and, if +0x18 is not
//!   0, "@", it and "rpm"; torque +0x1C (`append_torque`) with +0x20 the same way; bore
//!   +0x24 and stroke +0x28 (`append_length`); compression, the string +0x2C. Rows
//!   0x10A, 0x121, 0x14F, 0x166, 0x17D, 0x194, 0x1AB.
//! - 2 (`sub_46A193`), x = 0x220: transmission +0x30, drive +0x34 (0x103D for car 0x21
//!   with notice kind 3), steering +0x38, at 0xF3, 0x10A, 0x121; brakes at 0x138, two
//!   lines: "frente - " +0x3C (`append_length`), a space and the string +0x44, then
//!   "traseira - " +0x40 and +0x48; suspension at 0x18C: the strings +0x4C and +0x50
//!   after the same two words.
//! - 3 (`sub_469CA0`), x = 0x28B: length, width, height, wheelbase, front and rear track
//!   +0x54 .. +0x68 (`append_length`) at 0x10A .. 0x17D, 0x17 apart; weight +0x6C
//!   (`append_weight`) at 0x194.
//!
//! In two draw lists (the car specification screens). Refuses a page past the four, what the
//! record's lookup refuses, and a line longer than the line buffer. The menus and draw lists run
//! its `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::car_description::{read, shown_description};
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;

// ---- game data ----

// dword_755990
// dword_7559EC
const PAGES: u32 = 4;

// ---- Rust version ----

/// Returns the carry of the last text drawn.
pub fn draw_car_figures(menus: &Menus, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let page: u32 = menus.screens.event.car_page.get();
    if page >= PAGES {
        return Err(BadInput("a page past the four").into());
    }
    draw_car_page(page, shown_description(menus)?, canvas)
}

/// One page (`sub_469C40`, `sub_469F12`, `sub_46A193`, `sub_469CA0` for 0 .. 3) of the
/// record at `record`: the carry of the last text drawn. Converted:
/// `rc2k_core::screens::car_pages::page` (the record written back when it changed it: the Clio's
/// strings).
pub fn draw_car_page(page: u32, record: u32, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    use crate::screens::game_screen::GameScreen;
    use rc2k_core::screens::car_pages::{Font, Work};
    {
        let before = read(record)?;
        let mut d = before;
        let notice = crate::strings::load_language::notice_kind(); // g_noticeKind
        let carry = rc2k_core::screens::car_pages::page(
            &mut GameScreen::<Font, Work>::new(canvas),
            page,
            &mut d,
            notice,
            crate::strings::units::units(),
        );
        if d != before {
            crate::menu::cars_store::description(record)?.set(d);
        }
        carry
    }
}
