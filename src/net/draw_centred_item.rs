//! `net::draw_centred_item` (0x416B53): an item's string drawn (`text::draw_string_by_id`)
//! at x ecx, y ebp, in `unk_51E744`, or `stru_51E684` when it is the highlighted item and
//! edited or in the blink's lit half (as `net::draw_button`); ebp moves down by the font's
//! line height (+8).
//!
//! In:  eax = the string, ecx = x, esi = the item, ebp = y
//! Out: ebx = the font, edx = the y, ebp lower; eax .. edx as the drawing leaves them; no
//!      `pusha`
//! Caller: `net::settings_frame` (its last item). Its call through `part`. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose call goes to a stand-in, against
//! the Rust version: highlighted or not, edited, blinking.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::draw_button::item_font;
use crate::net::lobby::{
    CHOICE, DRAW_STRING, EDITED, FRAME_CLOCK, MOVE_CLOCK, Part, Saved, dispatch_lobby, dword, p,
    run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::draw_centred_item", rva(0x416B53), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLAIN: usize = 0x51E744; // unk_51E744
const LIT: usize = 0x51E684; // stru_51E684

/// The Rust version: the registers it leaves.
pub fn draw_centred_item(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.ebx = item_font(r.esi, PLAIN, LIT);
    r.edx = r.ebp;
    r.ebp = r
        .ebp
        .wrapping_add(unsafe { ((r.ebx + 8) as *const u32).read_unaligned() });
    part(DRAW_STRING, &mut r)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, draw_centred_item);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // lea ebx, unk_51E744
    let copy = match crate::net::lobby::copy(
        0x416B53,
        0x3B,
        &[0x8D, 0x1D, 0x44, 0xE7, 0x51, 0x00],
        &[0x35],
        &[DRAW_STRING],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(CHOICE),
        dword(EDITED),
        dword(FRAME_CLOCK),
        dword(MOVE_CLOCK),
    ];
    let saved = Saved::new(&all);
    for (choice, edited, clock) in [
        (4u32, u32::MAX, 0u32),
        (4, u32::MAX, 8),
        (4, 4, 8),
        (1, 4, 0),
    ] {
        run_both(
            &mut t,
            &format!("highlighted {choice}, edited {edited:#x}, clock {clock}"),
            copy.addr(),
            Regs {
                eax: 0x16F,
                ecx: 0x2A0,
                esi: 4,
                ebp: 0x21D,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                p(CHOICE, choice);
                p(EDITED, edited);
                p(FRAME_CLOCK, 0x2000 + clock);
                p(MOVE_CLOCK, 0x2000);
            },
            &draw_centred_item,
        );
    }
    drop(saved);
    t.finish()
}
