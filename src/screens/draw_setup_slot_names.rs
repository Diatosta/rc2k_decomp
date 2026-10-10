//! `screens::draw_setup_slot_names` (0x46F0DD): draws the names of the 30 setup slots of the
//! current page in two columns of 15.
//!
//! In:  nothing
//! Out: every register preserved; carry clear
//! The page is `dword_755244`; a name is 0x20 bytes of `byte_527880`. Columns at x = 233 and
//! 568, rows from y = 68 in steps of 29, font 0x51E8A4. It keeps its position in
//! `g_markerX`/`g_markerY` and its row counter in `dword_7559EC`, which are left at 568, 503
//! and 15. Refuses a page past the third: the table has 100 names.
//! Called by `draw_save_setup_list` and `draw_load_setup_list`.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable, push_dwords};
use crate::screens::{font_at, font_is_loaded};
use crate::service::harness::{at, capture, restore};
use crate::service::load_setup_slot::SLOT_PAGE_RVA;
use crate::service::read_setup_slot::{NAME_LEN, slot_name};
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_setup_slot_names",
    rva(0x46F0DD),
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

const X_RVA: usize = rva(0x755A18); // g_markerX
const Y_RVA: usize = rva(0x755A1C); // g_markerY
const ROW_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable, here the row counter

pub const FONT: usize = 0x51E8A4;
pub const COLUMNS: [i32; 2] = [0xE9, 0x238];
const FIRST_Y: i32 = 0x44;
const ROW_STEP: i32 = 0x1D;
const ROWS: u32 = 15;
pub const PAGES: u32 = 3;

// ---- Rust version ----

pub unsafe fn draw_setup_slot_names(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let page: u32 = unsafe { game_read(SLOT_PAGE_RVA) };
    if page >= PAGES {
        return Err(BadInput("slot page past the name table").into());
    }
    let mut slot = page * ROWS * 2;
    for x in COLUMNS {
        unsafe {
            game_write(X_RVA, x);
            game_write(Y_RVA, FIRST_Y);
            game_write(ROW_RVA, 0u32);
            for row in 0..ROWS {
                let y = FIRST_Y + ROW_STEP * row as i32;
                draw_text_at(font_at(FONT), slot_name(slot), x, y, canvas)?;
                slot += 1;
                game_write(Y_RVA, y + ROW_STEP);
                game_write(ROW_RVA, row + 1);
            }
        }
    }
    Ok(())
}

// ---- hook ----

pub const STATE_RVAS: [usize; 3] = [X_RVA, Y_RVA, ROW_RVA];

fn state() -> Vec<u8> {
    let mut s = Vec::new();
    push_dwords(&mut s, &STATE_RVAS);
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_setup_slot_names(canvas) }?;
    Ok(Left::carry(false))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: the three pages of the owner's own slot list ----

/// True if every name of the list ends within its 0x20 bytes, so that drawing stays inside it.
pub fn names_end() -> bool {
    (0..PAGES * ROWS * 2)
        .all(|slot| (0..NAME_LEN).any(|i| unsafe { slot_name(slot).add(i).read() } == 0))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if !font_is_loaded(FONT) || !names_end() {
        return t.skip("the font is not loaded or a slot name has no end");
    }
    let regions = [at(SLOT_PAGE_RVA), at(X_RVA), at(Y_RVA), at(ROW_RVA)];
    let saved = capture(&regions);
    for page in 0..PAGES {
        check_draw(
            &mut t,
            &format!("page={page}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(SLOT_PAGE_RVA, page);
                game_write(X_RVA, 0x5EEDi32);
                game_write(Y_RVA, 0x5EEDi32);
                game_write(ROW_RVA, 0x5EEDu32);
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
