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
//! In two draw lists (the car specification screens).
//! Refuses a page past the four, what the record's lookup refuses, and a line longer
//! than the line buffer.
//! Self-test: records of the test's on the four pages, both unit settings, for car 0x21
//! and another, with notice kind 3 and another; nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::car_description::{CarDescription, read, shown_description};
use crate::scene::carousel_pictures::ENTRIES_RVA;
use crate::screens::LINE_BUFFER_RVA;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::car_units::UNITS_RVA;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_car_figures", rva(0x47CEB4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
const CAR_RVA: usize = rva(0x755990); // dword_755990
pub const NOTICE_KIND_RVA: usize = rva(0x4E07A0); // g_noticeKind
pub const TEXT_KEPT_RVA: usize = rva(0x7559EC); // dword_7559EC
pub const LINE_LEN: usize = 0x100;
const PAGES: u32 = 4;

// ---- Rust version ----

/// Returns the carry of the last text drawn.
pub unsafe fn draw_car_figures(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let page: u32 = unsafe { game_read(PAGE_RVA) };
    if page >= PAGES {
        return Err(BadInput("a page past the four").into());
    }
    unsafe { draw_car_page(page, shown_description()?, canvas) }
}

/// One page (`sub_469C40`, `sub_469F12`, `sub_46A193`, `sub_469CA0` for 0 .. 3) of the
/// record at `record`: the carry of the last text drawn. Converted:
/// `rc2k_core::car_pages::page` (the record written back when it changed it: the Clio's
/// strings).
pub unsafe fn draw_car_page(
    page: u32,
    record: *mut CarDescription,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    use crate::screens::game_screen::GameScreen;
    use rc2k_core::car_pages::{Font, Work};
    unsafe {
        let before = read(record);
        let mut d = before;
        let notice: u8 = game_read(NOTICE_KIND_RVA);
        let carry = rc2k_core::car_pages::page(
            &mut GameScreen::<Font, Work>::new(canvas),
            page,
            &mut d,
            notice,
            crate::strings::units::units(),
        );
        if d != before {
            record.write_unaligned(d);
        }
        carry
    }
}

// ---- hook ----

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_car_figures(canvas) }?))
}

pub fn state() -> Vec<u8> {
    capture(&[
        (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
        (text() as u32, TEXT_LEN),
        at(TEXT_KEPT_RVA),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test ----

/// A record of the test's: string ids where the pages take strings, figures elsewhere.
pub fn test_record(car: u32, seed: u32) -> [u32; 0x1D] {
    let mut lot = seed;
    let mut next = |below: u32| {
        lot = lot.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (lot >> 8) % below
    };
    let mut record = [0u32; 0x1D];
    record[0] = car;
    for value in record.iter_mut().skip(3) {
        *value = match next(5) {
            0 => 0,
            1 => 0xFF00 | next(0x100),
            _ => next(30_000),
        };
    }
    // The strings.
    for offset in [0x10, 0x2C, 0x30, 0x34, 0x38, 0x44, 0x48, 0x4C, 0x50, 0x70] {
        record[offset / 4] = 0x249 + next(0x18);
    }
    record
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let inputs: [Region; 5] = [
        at(PAGE_RVA),
        at(CAR_RVA),
        at(ENTRIES_RVA),
        at(UNITS_RVA),
        (game_addr(NOTICE_KIND_RVA) as u32, 1),
    ];
    let saved = (capture(&inputs), state());
    for page in 0..PAGES {
        for units in [0u32, 1] {
            for (case, (car, notice)) in [(0x21u32, 3u8), (0x21, 2), (5, 3), (0x10, 0)]
                .iter()
                .enumerate()
            {
                let record =
                    std::cell::RefCell::new(test_record(*car, page * 31 + units * 7 + case as u32));
                let fresh = *record.borrow();
                let table = [record.as_ptr() as u32];
                check_draw(
                    &mut t,
                    &format!("page {page}, units {units}, car {car:#x}, notice kind {notice}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        *record.borrow_mut() = fresh;
                        game_write(PAGE_RVA, page);
                        game_write(CAR_RVA, 0u32);
                        game_write(ENTRIES_RVA, table.as_ptr() as u32);
                        game_write(UNITS_RVA, units);
                        game_write(NOTICE_KIND_RVA, *notice);
                    },
                    &rust,
                    &|| {
                        let mut s = state();
                        s.extend(record.borrow().iter().flat_map(|v| v.to_le_bytes()));
                        s
                    },
                );
            }
        }
    }
    restore(&inputs, &saved.0);
    restore(
        &[
            (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
            (text() as u32, TEXT_LEN),
            at(TEXT_KEPT_RVA),
        ],
        &saved.1,
    );
    t.finish()
}
