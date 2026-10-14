//! `input::append_key_name` (0x407F39): a key's name, as the keyboard pages show it.
//!
//! In:  eax = the key (its low seven bits), ebx = the cursor
//! Out: ebx moved on; eax as `append_string_by_id` leaves it, or the key's low seven bits;
//!      everything else preserved (ecx is pushed and popped)
//!
//! The current keyboard layout (`g_keyboardLayout`, 0x4F07D4) has at +0x10 a table of one
//! string id a key (words); a key with an id gets that string, one without "0x" and two
//! hexadecimal digits. No NUL is written. Eleven callers.
//! Refuses no layout, or a layout without its table of names.
//! Compared in shadow mode. Self-test: the 256 values of al into a buffer of 0xAA.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, game_read, hook_asm, rva, shadow};
use crate::input::key_to_char::LAYOUT_RVA;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::hex_append::hex_append;
use crate::strings::utils::bytes_str;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::append_key_name", rva(0x407F39), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The name's bytes and the eax the original leaves.
pub unsafe fn append_key_name(key: u32) -> Result<(Vec<u8>, u32), Refused> {
    let key = key & 0x7F;
    unsafe {
        let layout: u32 = game_read(LAYOUT_RVA);
        if layout == 0 {
            return Err(BadInput("no keyboard layout").into());
        }
        let names = ((layout + 0x10) as *const u32).read_unaligned();
        if names == 0 {
            return Err(BadInput("a keyboard layout without its key names").into());
        }
        let id = ((names + key * 2) as *const u16).read_unaligned() as u32;
        let mut out = Vec::new();
        if id != 0 {
            let eax =
                append_string_by_id(id, &mut |p, n| out.extend_from_slice(from_raw_parts(p, n)));
            Ok((out, eax))
        } else {
            out.extend_from_slice(b"0x");
            out.extend(hex_append(key, 2)?);
            Ok((out, key))
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (text, eax) = match unsafe { append_key_name(r.eax) } {
        Ok(it) => it,
        Err(e) => return unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    };
    if shadow() {
        let mut o = *r;
        unsafe { call_original(&mut o) };
        let theirs = unsafe {
            from_raw_parts(
                r.ebx as *const u8,
                (o.ebx as usize).wrapping_sub(r.ebx as usize).min(0x100),
            )
        };
        if theirs != &text[..] || o.eax != eax || !o.preserved_except(r, EAX | EBX) {
            HOOK.mismatch(|| {
                format!(
                    "key {:#x}: original {} eax={:#x}; rust {} eax={eax:#x}",
                    r.eax,
                    bytes_str(theirs),
                    o.eax,
                    bytes_str(&text)
                )
            });
        }
        *r = Regs { esp: r.esp, ..o };
        return;
    }
    unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), r.ebx as *mut u8, text.len()) };
    r.ebx += text.len() as u32;
    r.eax = eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if crate::strings::string_table::blob().is_null() {
        return t.skip("string table not loaded");
    }
    if !crate::input::key_to_char::has_layout() {
        return t.skip("no keyboard layout");
    }
    for value in 0..0x100u32 {
        let key = 0x1234_5600 | value;
        let mut theirs = [0xAAu8; 0x80];
        let before = Regs {
            eax: key,
            ebx: theirs.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut o = before;
        unsafe { call_original(&mut o) };
        let mut mine = [0xAAu8; 0x80];
        match unsafe { append_key_name(key) } {
            Ok((text, eax)) => {
                mine[..text.len()].copy_from_slice(&text);
                let ok = theirs == mine
                    && o.eax == eax
                    && o.ebx == before.ebx + text.len() as u32
                    && o.preserved_except(&before, EAX | EBX);
                t.check(ok, || {
                    format!(
                        "key {key:#x}: original {} eax={:#x} ebx+{}; rust {} eax={eax:#x}",
                        bytes_str(&theirs),
                        o.eax,
                        o.ebx.wrapping_sub(before.ebx),
                        bytes_str(&mine)
                    )
                });
            }
            Err(e) => t.check(false, || format!("key {key:#x}: rust refused: {e:?}")),
        }
    }
    t.finish()
}
