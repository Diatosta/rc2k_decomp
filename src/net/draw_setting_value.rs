//! `net::draw_setting_value` (0x416BC4): a setting's value drawn (`text::draw_text` with
//! `font` at 0x190, y ebp; ebp 0x1E lower): for the item being edited (`dword_52A4B8`) the
//! text with string 4 (the cursor) after it, built in the game's text buffer; an empty text
//! as string 0x16E; any other as it is.
//!
//! In:  eax = the text, esi = the item, ebp = y
//! Out: as `text::draw_text` leaves them (eax 0, ebx past the text, ecx, edx); ebp + 0x1E;
//!      no `pusha`
//! Callers: `net::settings_frame`, `net::draw_number_value` (which goes on into it). Its
//! calls through `part` (`strings::append_nul` inline). Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose calls go to stand-ins (the strings appended), against
//! the Rust version: edited, empty, a text.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    APPEND_STRING, DRAW_TEXT, EDITED, FONT, Part, STR_APPEND, Saved, TEXT, a, answer,
    dispatch_lobby, dword, g, p, region, run_both,
};
use rc2k_core::net::screens::{self, Value};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::draw_setting_value", rva(0x416BC4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn nul(r: &mut Regs) {
    unsafe { (r.ebx as *mut u8).write(0) };
    r.ebx = r.ebx.wrapping_add(1);
}

/// The Rust version: the registers it leaves.
pub fn draw_setting_value(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::screens` (`text_value`, `Value`, `CARET`, `NOTHING`).
    let text = r.eax;
    match screens::text_value(r.esi == g(EDITED), || unsafe { (text as *const u8).read() }) {
        Value::Edited => {
            r.ebx = a(TEXT);
            part(STR_APPEND, &mut r)?;
            r.eax = screens::CARET;
            part(APPEND_STRING, &mut r)?;
            nul(&mut r);
            r.eax = a(TEXT);
        }
        Value::Nothing => {
            r.eax = screens::NOTHING;
            r.ebx = a(TEXT);
            part(APPEND_STRING, &mut r)?;
            nul(&mut r);
            r.eax = a(TEXT);
        }
        Value::AsIs => {}
    }
    r.ebx = r.eax;
    r.eax = a(FONT);
    r.ecx = 0x190;
    r.edx = r.ebp;
    r.ebp = r.ebp.wrapping_add(0x1E);
    part(DRAW_TEXT, &mut r)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, draw_setting_value);
}

/// The test's answers for the strings: appended as the game's.
pub fn string_answers() -> crate::net::lobby::Answer {
    Box::new(|place, r| match place {
        STR_APPEND => {
            r.ebx = unsafe {
                crate::strings::str_append::str_append(r.eax as *const u8, r.ebx as *mut u8)
            } as u32
        }
        APPEND_STRING => {
            unsafe { (r.ebx as *mut [u8; 2]).write_unaligned(*b"#!") };
            r.ebx += 2;
            r.eax |= 0x0055_0000;
        }
        crate::net::lobby::FORMAT_U32 => {
            let text = crate::strings::format_u32::format_u32(r.eax, r.ecx as u8);
            unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), r.ebx as *mut u8, text.len()) };
            r.ebx += text.len() as u32;
        }
        DRAW_TEXT => {
            r.eax = 0;
            r.ebx = r.ebx.wrapping_add(3);
        }
        _ => {}
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp esi, dword_52A4B8
    let copy = match crate::net::lobby::copy(
        0x416BC4,
        0x62,
        &[0x3B, 0x35, 0xB8, 0xA4, 0x52, 0x00],
        &[0xE, 0x18, 0x1D, 0x3A, 0x3F, 0x5C],
        &[STR_APPEND, APPEND_STRING, DRAW_TEXT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(EDITED), region(TEXT, 0x40)];
    let saved = Saved::new(&all);
    for (what, edited, text) in [
        ("edited", 1u32, &b"name\0"[..]),
        ("empty", u32::MAX, b"\0"),
        ("a text", 3, b"192.168.0.1\0"),
    ] {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs {
                eax: text.as_ptr() as u32,
                esi: 1,
                ebp: 0xB2,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                p(EDITED, edited);
                crate::net::lobby::fill(TEXT, 0x40, 3);
                answer(string_answers());
            },
            &draw_setting_value,
        );
    }
    drop(saved);
    t.finish()
}
