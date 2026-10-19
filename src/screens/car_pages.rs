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
use crate::framework::{RawHook, Regs, SelfTest, game_write};
use crate::menu::car_description::CarDescription;
use crate::screens::draw_car_figures::{
    LINE_LEN, NOTICE_KIND_RVA, TEXT_KEPT_RVA, draw_car_page, state, test_record,
};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{CallOriginal, Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::car_units::UNITS_RVA;
use crate::text::draw_text::Canvas;

fn rust(r: &Regs, page: u32, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe {
        draw_car_page(page, r.edi as *mut CarDescription, canvas)
    }?))
}

/// What the page writes besides the drawing: the buffers and the record.
fn state_with(record: u32) -> Vec<u8> {
    let mut s = state();
    s.extend(capture(&[(record, 0x74)]));
    s
}

pub fn dispatch_page(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, page: u32) {
    let record = r.edi;
    dispatch_draw(hook, r, call_original, &|r, c| rust(r, page, c), &|| {
        state_with(record)
    });
}

pub fn test_page(hook: &RawHook, call_original: CallOriginal, page: u32) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let inputs: [Region; 2] = [
        at(UNITS_RVA),
        (crate::framework::game_addr(NOTICE_KIND_RVA) as u32, 1),
    ];
    let buffers: [Region; 3] = [
        (
            crate::framework::game_addr(crate::screens::LINE_BUFFER_RVA) as u32,
            LINE_LEN,
        ),
        (
            crate::strings::text_from_ids::text() as u32,
            crate::strings::text_from_ids::TEXT_LEN,
        ),
        at(TEXT_KEPT_RVA),
    ];
    let saved = (capture(&inputs), capture(&buffers));
    for units in [0u32, 1] {
        for (case, (car, notice)) in [(0x21u32, 3u8), (0x21, 2), (5, 3), (0x10, 0)]
            .iter()
            .enumerate()
        {
            let record =
                std::cell::RefCell::new(test_record(*car, page * 31 + units * 7 + case as u32));
            let fresh = *record.borrow();
            let at_record = record.as_ptr() as u32;
            check_draw(
                &mut t,
                &format!("units {units}, car {car:#x}, notice kind {notice}"),
                call_original,
                Regs {
                    edi: at_record,
                    eax: *car,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    *record.as_ptr() = fresh;
                    game_write(UNITS_RVA, units);
                    game_write(NOTICE_KIND_RVA, *notice);
                },
                &|r, c| rust(r, page, c),
                &|| state_with(at_record),
            );
        }
    }
    restore(&inputs, &saved.0);
    restore(&buffers, &saved.1);
    t.finish()
}
