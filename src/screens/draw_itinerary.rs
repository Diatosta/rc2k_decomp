//! `screens::draw_itinerary` (0x472C8C): a rally's itinerary: its stages and services with
//! numbers, lengths and times of day (see `screens/itinerary.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! It has two chunks besides its body: 0x40763C (the fatal reporter it jumps into, shared)
//! and 0x4D9563 (its fatal stub), so no private copy is made. In a menu loop's draw list
//! (0x758D56). Refuses (before anything) a section past the record table. Compared in shadow
//! mode (`dispatch_draw`). Self-test: the rows `index` 1 .. 3 highlighted, nothing drawn
//! (skipped without the fonts or the records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::itinerary::draw_itinerary;
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_itinerary", rva(0x472C8C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_itinerary(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    capture(&[
        (crate::screens::line_buffer() as u32, 0x100),
        g(0x7559AC, 0xC),
        g(0x755A08, 8),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    if !crate::screens::best_scores::section_there() {
        return t.skip("no record table section");
    }
    let regions: Vec<Region> = vec![at(rva(0x755958))];
    let saved = capture(&regions);
    for row in [1u32, 2, 3] {
        check_draw(
            &mut t,
            &format!("row {row}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(0x755958), row) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
