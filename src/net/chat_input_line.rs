//! `net::chat_input_line` (0x418083): the lobby's chat input drawn (`text::draw_text` with
//! `stru_51E584` at x 0xA, y ebp; ebp 0x1E lower), with string 4 (the cursor) after it while
//! it is being edited (`dword_52A4B8` the item), built in the game's text buffer.
//!
//! In:  eax = the text, esi = the item, ebp = y
//! Out: as `text::draw_text` leaves them; ebp + 0x1E; no `pusha`
//! Caller: `net::lobby_frame`. Its calls through `part` (`strings::append_nul` inline).
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version, edited or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    APPEND_STRING, DRAW_TEXT, EDITED, Part, STR_APPEND, Saved, TEXT, a, answer, dispatch_lobby,
    dword, g, p, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::chat_input_line", rva(0x418083), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CHAT_FONT: usize = 0x51E584; // stru_51E584

/// The Rust version: the registers it leaves.
pub fn chat_input_line(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if r.esi == g(EDITED) {
        r.ebx = a(TEXT);
        part(STR_APPEND, &mut r)?;
        r.eax = 4;
        part(APPEND_STRING, &mut r)?;
        unsafe { (r.ebx as *mut u8).write(0) };
        r.ebx = r.ebx.wrapping_add(1);
        r.eax = a(TEXT);
    }
    r.ebx = r.eax;
    r.eax = a(CHAT_FONT);
    r.ecx = 0xA;
    r.edx = r.ebp;
    r.ebp = r.ebp.wrapping_add(0x1E);
    part(DRAW_TEXT, &mut r)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, chat_input_line);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp esi, dword_52A4B8
    let copy = match crate::net::lobby::copy(
        0x418083,
        0x42,
        &[0x3B, 0x35, 0xB8, 0xA4, 0x52, 0x00],
        &[0xE, 0x18, 0x1D, 0x3C],
        &[STR_APPEND, APPEND_STRING, DRAW_TEXT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(EDITED), region(TEXT, 0x40)];
    let saved = Saved::new(&all);
    let text = b"hello\0";
    for edited in [3u32, u32::MAX] {
        run_both(
            &mut t,
            &format!("edited {edited:#x}"),
            copy.addr(),
            Regs {
                eax: text.as_ptr() as u32,
                esi: 3,
                ebp: 0x1A0,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                p(EDITED, edited);
                crate::net::lobby::fill(TEXT, 0x40, 8);
                answer(crate::net::draw_setting_value::string_answers());
            },
            &chat_input_line,
        );
    }
    drop(saved);
    t.finish()
}
