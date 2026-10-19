//! `net::edit_key` (0x416E43): a key for the text being edited (`net::begin_edit`), if one
//! is (`dword_52A4B8` not -1): Escape (1) puts the kept text back and ends the editing,
//! Enter (0x1C) ends it, 0x6E clears the text's 64 bytes, backspace (0x0E) drops its last
//! character; any other key is made a character (`input::key_to_char`), which, if it is one
//! (0x20 or more), goes at the text's end (if it has fewer than 0x3F) with a NUL after it,
//! and is taken off again if the text is then wider (`text::measure_text` with the editing's
//! font, signed) than allowed.
//!
//! In:  al = the key
//! Out: every register kept (`pusha` .. `popa`); the flags of `cmp dword_52A4B8, -1`: zero
//!      set when no text is edited (any more)
//! Callers: `net::setting_key`, `net::chat_key`. Its calls through `part` (`net::lobby_calls`:
//! the Rust versions). Shadow mode runs the original alone (the dead key's state). Self-test:
//! a copy whose calls go to stand-ins (the character and the width given), against the Rust
//! version: not editing, each key, a character below 0x20, too wide, a full text.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva};
use crate::net::lobby::{
    EDIT_FONT, EDIT_SAVED, EDIT_TEXT, EDIT_WIDTH, EDITED, FLAGS, KEY_TO_CHAR, MEASURE_TEXT, Part,
    Saved, a, answer, dispatch_lobby, fill, g, movsb, p, region, run_both, set_sign,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::edit_key", rva(0x416E43), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The flags of `cmp dword_52A4B8, -1`.
pub fn editing_flags(r: &mut Regs) {
    let v = g(EDITED);
    r.set_zero(v == u32::MAX);
    r.set_carry(v != u32::MAX);
    set_sign(r, v.wrapping_add(1) as i32);
}

fn byte(at: u32) -> u8 {
    unsafe { (at as *const u8).read() }
}

fn set_byte(at: u32, v: u8) {
    unsafe { (at as *mut u8).write(v) }
}

/// The Rust version: the registers it leaves (the entry's, the flags of the last compare).
pub fn edit_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(EDITED) != u32::MAX {
        let text = g(EDIT_TEXT);
        r.ebx = text;
        match r.eax as u8 {
            1 => {
                movsb(text, a(EDIT_SAVED), 0x40);
                p(EDITED, u32::MAX);
            }
            0x1C => p(EDITED, u32::MAX),
            0x6E => unsafe { std::ptr::write_bytes(text as *mut u8, 0, 0x40) },
            0x0E => {
                if byte(text) != 0 {
                    let mut end = text;
                    while byte(end) != 0 {
                        end += 1;
                    }
                    set_byte(end - 1, 0);
                }
            }
            _ => {
                part(KEY_TO_CHAR, &mut r)?;
                if r.eax as u8 >= 0x20 {
                    r.eax &= !0xFF00;
                    if let Some(at) = (0..0x3F).find(|&n| byte(text + n) == 0) {
                        r.ecx = at;
                        unsafe { ((text + at) as *mut u16).write_unaligned(r.eax as u16) };
                        let mut m = Regs {
                            eax: g(EDIT_FONT),
                            ..r
                        };
                        part(MEASURE_TEXT, &mut m)?;
                        if m.ecx as i32 > g(EDIT_WIDTH) as i32 {
                            set_byte(text + at, 0);
                        }
                    }
                }
            }
        }
    }
    let mut left = Regs {
        eflags: r.eflags,
        ..*entry
    };
    editing_flags(&mut left);
    Ok(left)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, edit_key);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp dword_52A4B8, 0FFFFFFFFh
    let copy = match crate::net::lobby::copy(
        0x416E43,
        0xA9,
        &[0x60, 0x83, 0x3D, 0xB8, 0xA4, 0x52, 0x00, 0xFF],
        &[0x6B, 0x8F],
        &[KEY_TO_CHAR, MEASURE_TEXT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut text = vec![0u8; 0x50];
    let at = text.as_mut_ptr() as u32;
    let mut all = vec![region(EDIT_TEXT, 0x10), region(EDIT_SAVED, 0x40)];
    let saved = Saved::new(&all);
    all.push((at, 0x50));
    let long = [b'x'; 0x3F];
    // (editing, the key, the text, what key_to_char gives, the width measured)
    let cases: [(bool, u32, &[u8], u32, u32); 13] = [
        (false, 0x1E, b"abc", 0x61, 0),
        (true, 1, b"abc", 0, 0),
        (true, 0x1C, b"abc", 0, 0),
        (true, 0x6E, b"abc", 0, 0),
        (true, 0x0E, b"abc", 0, 0),
        (true, 0x0E, b"", 0, 0),
        (true, 0x0E, b"q", 0, 0),
        (true, 0x1E, b"abc", 0x0000_1261, 0x100),
        (true, 0x1E, b"abc", 0x0000_0261, 0x30C),
        (true, 0x1E, b"abc", 0x61, 0x30D),
        (true, 0x1E, b"abc", 0x1F, 0),
        (true, 0x9E, b"abc", 0xAB00_0000, 0),
        (true, 0x1E, &long, 0x61, 0),
    ];
    for (editing, key, sample, given, width) in cases {
        run_both(
            &mut t,
            &format!(
                "editing {editing}, key {key:#x}, text of {}, char {given:#x}, width {width:#x}",
                sample.len()
            ),
            copy.addr(),
            Regs {
                eax: 0x1234_0000 | key,
                ..Regs::SENTINEL
            },
            &all,
            FLAGS,
            &[],
            &|| unsafe {
                fill(EDIT_SAVED, 0x40, 11);
                p(EDITED, if editing { 2 } else { u32::MAX });
                p(EDIT_TEXT, at);
                p(EDIT_WIDTH, 0x30C);
                p(EDIT_FONT, 0x0051_E424);
                std::ptr::write_bytes(at as *mut u8, 0xC3, 0x50);
                std::ptr::copy_nonoverlapping(sample.as_ptr(), at as *mut u8, sample.len());
                (at as *mut u8).add(sample.len()).write(0);
                answer(Box::new(move |place, r| match place {
                    KEY_TO_CHAR => {
                        r.eax = given;
                        r.set_carry(given as u8 == 0);
                        r.set_zero(given as u8 == 1);
                    }
                    _ => {
                        r.eax = 0;
                        r.ebx = 1;
                        r.ecx = width;
                        r.edx = 0x40;
                    }
                }));
            },
            &edit_key,
        );
    }
    let _ = (&mut text, ZERO_FLAG);
    drop(saved);
    t.finish()
}
