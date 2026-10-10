//! `screens::draw_empty_savegame` (0x46E2DE): the load-game screen's preview for a slot with
//! no saved game: the headings and row numbers, "no entry" (string 0x578) as the title and as
//! every row's name, and "nothing" (string 0x57A) in the six other places of every row.
//!
//! In:  nothing
//! Out: every register preserved; carry as the last `draw_string_by_id` leaves it
//! Called by `draw_savegame_preview`. String 0x578 is "n/d" and string 0x57A is "-" (seen on
//! the rows of `draw_savegame_details` in the owner's screenshot of 2026-10-10); this panel
//! itself, for an empty slot, has not been seen.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable};
use crate::screens::savegame_preview::{
    FONT, FONTS, LEFT_X, NAME_X, NO_CAR_X, NO_ENTRY, NO_FLAG_X, NOTHING, NUMBER_COLUMNS, ROWS,
    STRINGS, TITLE_FONT, TITLE_Y, draw_frame,
};
use crate::screens::{font_at, font_is_loaded};
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_empty_savegame",
    rva(0x46E2DE),
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

// ---- Rust version ----

/// Returns the carry flag of the last text drawn.
pub unsafe fn draw_empty_savegame(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        draw_frame(canvas)?;
        let mut last =
            draw_string_by_id_at(NO_ENTRY, font_at(TITLE_FONT), LEFT_X, TITLE_Y, canvas)?;
        for y in ROWS {
            draw_string_by_id_at(NO_ENTRY, font_at(FONT), NAME_X, y, canvas)?;
            for x in [NO_FLAG_X, NO_CAR_X].into_iter().chain(NUMBER_COLUMNS) {
                last = draw_string_by_id_at(NOTHING, font_at(FONT), x, y, canvas)?;
            }
        }
        Ok(last.too_wide)
    }
}

// ---- hook ----

/// The shared text buffer and the line buffer, which the drawing leaves its last texts in.
pub fn state() -> Vec<u8> {
    let mut s = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    s.extend_from_slice(unsafe { from_raw_parts(crate::screens::line_buffer(), 0x80) });
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_empty_savegame(canvas) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test ----

/// Why the preview panels cannot be drawn in a test, or None.
pub fn preview_unavailable() -> Option<&'static str> {
    drawing_unavailable().or_else(|| {
        (blob().is_null()
            || !FONTS.iter().all(|&f| font_is_loaded(f))
            || !STRINGS.iter().all(|&id| is_table_id(id)))
        .then_some("the strings or the fonts are not loaded")
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = preview_unavailable() {
        return t.skip(why);
    }
    let saved = state();
    let put_back = || unsafe {
        std::ptr::copy_nonoverlapping(saved.as_ptr(), text(), TEXT_LEN);
        crate::screens::set_line(&saved[TEXT_LEN..]);
    };
    check_draw(
        &mut t,
        "the empty panel",
        call_original,
        Regs::SENTINEL,
        &put_back,
        &rust,
        &state,
    );
    put_back();
    t.finish()
}
