//! `net::draw_number_value` (0x416B8E): a number setting's value: for the item being edited
//! the number's text as typed (`byte_95E8F8`); else the number written into the game's text
//! buffer (`strings::format_u32`, or string 0x16E for 0); then on into
//! `net::draw_setting_value`.
//!
//! In:  eax = the number, esi = the item, ebp = y
//! Out: as `net::draw_setting_value` leaves them; no `pusha`
//! Caller: `net::settings_frame` (the most players). Its calls through `part`
//! (`strings::append_nul` inline). Shadow mode runs the original alone (it draws).
//! Self-test: a copy of it and `net::draw_setting_value` after it, whose calls go to
//! stand-ins, against the Rust version: edited, 0, 4.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::draw_setting_value::{draw_setting_value, string_answers};
use crate::net::lobby::{
    APPEND_STRING, DRAW_TEXT, EDITED, FORMAT_U32, NUMBER_TEXT, Part, STR_APPEND, Saved, TEXT, a,
    answer, dispatch_lobby, dword, g, p, put_text, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::draw_number_value", rva(0x416B8E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The first 0x3C bytes as in the file: both its own first bytes and those of
/// `net::draw_setting_value` at +0x36 are a hook's jump in memory.
const ENTRY: [u8; 0x3C] = [
    0x3B, 0x35, 0xB8, 0xA4, 0x52, 0x00, 0x74, 0x28, 0x8D, 0x1D, 0x30, 0xE4, 0x93, 0x00, 0x53, 0x51,
    0x33, 0xC9, 0x85, 0xC0, 0x75, 0x0C, 0xB8, 0x6E, 0x01, 0x00, 0x00, 0xE8, 0xF5, 0xB3, 0xFE, 0xFF,
    0xEB, 0x05, 0xE8, 0x95, 0xB4, 0xFE, 0xFF, 0xE8, 0x5C, 0xB3, 0xFE, 0xFF, 0x59, 0x58, 0xEB, 0x06,
    0x8D, 0x05, 0xF8, 0xE8, 0x95, 0x00, 0x3B, 0x35, 0xB8, 0xA4, 0x52, 0x00,
];

/// The Rust version: the registers it leaves.
pub fn draw_number_value(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if r.esi == g(EDITED) {
        r.eax = a(NUMBER_TEXT);
    } else {
        r.ebx = a(TEXT);
        let kept_ecx = r.ecx;
        r.ecx = 0;
        if r.eax == 0 {
            r.eax = 0x16E;
            part(APPEND_STRING, &mut r)?;
        } else {
            part(FORMAT_U32, &mut r)?;
        }
        unsafe { (r.ebx as *mut u8).write(0) };
        r.ebx = r.ebx.wrapping_add(1);
        r.ecx = kept_ecx;
        r.eax = a(TEXT);
    }
    draw_setting_value(&r, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, draw_number_value);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp esi, dword_52A4B8 (then sub_416BC4 after it)
    let copy = match crate::net::lobby::copy(
        0x416B8E,
        0x98,
        &ENTRY,
        &[0x1B, 0x22, 0x27, 0x44, 0x4E, 0x53, 0x70, 0x75, 0x92],
        &[APPEND_STRING, FORMAT_U32, STR_APPEND, DRAW_TEXT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(EDITED), region(TEXT, 0x40), region(NUMBER_TEXT, 0x10)];
    let saved = Saved::new(&all);
    for (what, edited, number) in [("edited", 2u32, 4u32), ("0", u32::MAX, 0), ("4", 3, 4)] {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs {
                eax: number,
                esi: 2,
                ebp: 0xD2,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                p(EDITED, edited);
                crate::net::lobby::fill(TEXT, 0x40, 4);
                put_text(NUMBER_TEXT, b"6");
                answer(string_answers());
            },
            &draw_number_value,
        );
    }
    drop(saved);
    t.finish()
}
