//! The car specification's four figure pages as functions of their own
//! (`screens::car_page_description` 0x469C40, `car_page_engine` 0x469F12,
//! `car_page_transmission` 0x46A193, `car_page_dimensions` 0x469CA0), called through
//! `funcs_47CED1` (0x759AE6) by `screens::draw_car_figures`, whose Rust version draws the
//! same pages (`draw_car_figures::draw_car_page`) without calling them: these hooks only
//! take calls when its hook is off. What each draws: `screens/draw_car_figures.rs`.
//!
//! In:  edi = the car's record (eax its first dword, not read)
//! Out: every register kept (`pusha` .. `popa`); the carry of the last text drawn
//! Compared in shadow mode (`dispatch_draw`; the line buffer, the text buffer,
//! `dword_7559EC` and the record as state). Refuses a line longer than the line buffer and
//! what the text functions refuse. Self-test: records of the test's, both unit settings,
//! for car 0x21 and another, with notice kind 3 and another; nothing drawn.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::car_description::CarDescription;
use crate::screens::draw_car_figures::draw_car_page;
use crate::screens::harness::Left;
use crate::text::draw_text::Canvas;

fn rust(r: &Regs, page: u32, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe {
        draw_car_page(page, r.edi as *mut CarDescription, canvas)
    }?))
}

/// `dispatch_page` without its hook (typed tables).
pub fn page_entry(name: &str, r: &mut Regs, page: u32) -> Result<(), Refused> {
    crate::screens::harness::draw_entry(name, r, &|r, c| rust(r, page, c))
}
