//! `screens::draw_load_setup_list` (0x46F087): draws the text of the "load setup" screen: a
//! title (string 0x4DD), a label under each column of slots (strings 0x4E3 and 0x4E4) and the
//! slot names (`draw_setup_slot_names`).
//!
//! In:  nothing
//! Out: every register preserved; carry clear
//! Its address is in the draw list at 0x75915E. That it is the "load" screen is a reading:
//! by elimination from `draw_save_setup_list`. IDA had no function here until 2026-10-10.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::screens::draw_setup_slot_names::{
    COLUMNS, FONT, PAGES, STATE_RVAS, draw_setup_slot_names, names_end,
};
use crate::screens::font_is_loaded;
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable, push_dwords};
use crate::screens::stage_panel::label;
use crate::service::harness::{Region, at, capture, restore};
use crate::service::load_setup_slot::SLOT_PAGE_RVA;
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_load_setup_list",
    rva(0x46F087),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TITLE: u32 = 0x4DD;
const COLUMN_LABELS: [u32; 2] = [0x4E3, 0x4E4];
const LABEL_Y: i32 = 0x1F7;

// ---- Rust version ----

pub unsafe fn draw_load_setup_list(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        label(canvas, TITLE, FONT, 0x190, 0xE)?;
        for (id, x) in COLUMN_LABELS.into_iter().zip(COLUMNS) {
            label(canvas, id, FONT, x, LABEL_Y)?;
        }
        draw_setup_slot_names(canvas)
    }
}

// ---- hook ----

fn state() -> Vec<u8> {
    let mut s = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    push_dwords(&mut s, &STATE_RVAS);
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_load_setup_list(canvas) }?;
    Ok(Left::carry(false))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: the three pages of the owner's own slot list ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if blob().is_null()
        || !font_is_loaded(FONT)
        || !names_end()
        || ![TITLE, COLUMN_LABELS[0], COLUMN_LABELS[1]]
            .into_iter()
            .all(is_table_id)
    {
        return t.skip("strings or font not loaded, or a slot name has no end");
    }
    let mut regions: Vec<Region> = vec![at(SLOT_PAGE_RVA), (text() as u32, TEXT_LEN)];
    regions.extend(STATE_RVAS.map(at));
    let saved = capture(&regions);
    for page in 0..PAGES {
        check_draw(
            &mut t,
            &format!("page={page}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&regions, &saved);
                unsafe { game_write(SLOT_PAGE_RVA, page) };
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
