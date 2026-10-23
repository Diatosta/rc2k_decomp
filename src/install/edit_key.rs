//! `install::edit_key` (0x4148F7): a key while an edit item (kind 4) is being edited.
//! - ah bit 0: al is the character itself;
//! - Escape (1): the edit ended (record bit 0 off) and the text put back from
//!   `byte_95EAF8` (its most bytes and two); ah gets bit 3;
//! - Enter (0x1C): the edit ended; a text of nothing but spaces and control bytes is cleared
//!   (its most bytes and two) and the record's default text (`[record+2]`, a string id)
//!   put in; ah gets bit 3;
//! - 0x6E: the text cleared; Backspace (0xE): its last byte cut;
//! - the four arrows: nothing;
//! - any other: the key's character (`input::key_to_char`, with `dword_4F07F8` as the
//!   layout while `byte_4F07D2` is 2) added if it is not below a space and there is room
//!   (the most bytes), and cut again if the text is then wider than `[record+6]`
//!   (`text::measure_text` with the font of the record's kind: `byte_51D52D`, `dword_51D4C4`).
//!
//! In:  al the key, ah its bits, esi the item (+8 the text, +0xC the record)
//! Out: esi, edi, ebp kept; zero set and carry clear, but for Escape and Enter (the flags of
//!      `or ah, 8`: zero clear); eax, ebx, ecx, edx as each way leaves them
//! Caller `install::page_key`. Shadow mode compares (the measuring only writes the font's
//! two results). Self-test: an edit of the test's and the game's fonts, against the
//! original, every key's way.

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CF, COLOURS, FONTS, SCRIPT_PATH, ZF, a, append_string_by_id, g, logic, rb, rd, region, rw, wb,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original, dispatch_regs, regs_entry};
use crate::service::harness::Region;
use rc2k_core::install::edits::{self, EDITING, EditKey};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::edit_key", rva(0x4148F7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LAYOUT_MODE: usize = 0x4F07D2; // byte_4F07D2
const LAYOUT: usize = 0x4F07D4; // g_keyboardLayout
const OTHER_LAYOUT: usize = 0x4F07F8; // dword_4F07F8

/// The font of an edit record's kind.
pub fn edit_font(record: u32) -> u32 {
    let kind = rb(record + 4) as u32;
    let font = rb(a(COLOURS) + 5 + 8 * kind) as u32;
    rd(a(FONTS) + 4 * font)
}

fn past_nul(mut at: u32) -> u32 {
    loop {
        let b = rb(at);
        at = at.wrapping_add(1);
        if b < 1 {
            return at;
        }
    }
}

fn clear(text: u32, most: u32, r: &mut Regs) {
    for n in 0..most + 2 {
        wb(text + n, 0);
    }
    r.ecx = 0;
}

pub fn edit_key(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let text = rd(r.esi + 8);
    let record = rd(r.esi + 0xC);
    let most = rb(record + 1) as u32;
    let key = r.eax as u8;
    let done = |mut r: Regs| {
        r.set_zero(true);
        r.set_carry(false);
        Ok(r)
    };
    let with_bit_3 = |mut r: Regs| {
        r.eax |= 0x800;
        let ah = (r.eax >> 8) & 0xFF;
        logic(&mut r, ah);
        Ok(r)
    };
    let backspace = |mut r: Regs| {
        r.ebx = text;
        if rb(text) != 0 {
            r.ebx = past_nul(text);
            wb(r.ebx.wrapping_sub(2), 0);
        }
        done(r)
    };
    // Converted: `rc2k_core::install::edits` (`EditKey`, `blank`, `takes`, `too_wide`).
    let ch = if r.eax & 0x100 != 0 {
        key
    } else {
        match EditKey::of(key) {
            EditKey::Escape => {
                wb(record, rb(record) & !EDITING);
                let backup = a(SCRIPT_PATH);
                for n in 0..most + 2 {
                    wb(text + n, rb(backup + n));
                }
                r.ecx = 0;
                return with_bit_3(r);
            }
            EditKey::Enter => {
                wb(record, rb(record) & !EDITING);
                let words = unsafe { Raw::new(text as *const u8) };
                let (blank, stop) = read_through(edits::blank(&words));
                r.ebx = text.wrapping_add(stop as u32);
                if !blank {
                    return with_bit_3(r);
                }
                clear(text, most, &mut r);
                let mut s = Regs {
                    eax: rw(record + 2) as u32,
                    ebx: text,
                    ..r
                };
                append_string_by_id(&mut s);
                r.ebx = s.ebx;
                return with_bit_3(r);
            }
            EditKey::Clear => {
                clear(text, most, &mut r);
                return done(r);
            }
            EditKey::Backspace => return backspace(r),
            EditKey::Arrow => return done(r),
            EditKey::Other => {
                let swap = rb(a(LAYOUT_MODE)) == 2;
                let kept = g(LAYOUT);
                if swap {
                    crate::install::installer::p(LAYOUT, a(OTHER_LAYOUT));
                }
                let out = unsafe { crate::input::key_to_char::key_to_char(r.eax) };
                if swap {
                    crate::install::installer::p(LAYOUT, kept);
                }
                out? as u8
            }
        }
    };
    r.edx = r.edx & !0xFF | ch as u32;
    if ch < 0x20 {
        return done(r);
    }
    r.ebx = past_nul(text);
    r.ecx = most.wrapping_add(text).wrapping_add(1);
    let length = r.ebx.wrapping_sub(text).wrapping_sub(1) as usize;
    if !edits::takes(ch, length, most as u8) {
        return done(r);
    }
    wb(r.ebx.wrapping_sub(1), ch);
    wb(r.ebx, 0);
    let mut m = Regs {
        eax: edit_font(record),
        ebx: text,
        ..r
    };
    crate::install::installer::game_part(a(crate::install::installer::MEASURE_TEXT), &mut m)?;
    r.ebx = m.ebx;
    r.ecx = m.ecx;
    r.edx = m.edx;
    r.eax = rw(record + 6) as u32;
    if edits::too_wide(r.ecx, r.eax as u16) {
        return backspace(r);
    }
    done(r)
}

fn regions_of(item: u32) -> Vec<Region> {
    let record = rd(item + 0xC);
    let mut all = vec![
        (record, 8),
        (rd(item + 8), rb(record + 1) as usize + 2),
        region(LAYOUT, 4),
        (edit_font(record), 0x20),
    ];
    all.extend(crate::input::key_to_char::regions());
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions_of(r.esi);
    dispatch_regs(&HOOK, r, call_original, &regions, CF | ZF, &edit_key);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &edit_key)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::input::key_to_char::has_layout() {
        return t.skip("no keyboard layout yet");
    }
    let list = TestList::new();
    list.set(0, &[4, 4], 0, 0, 0, 1, 100, b"");
    if edit_font(rd(list.item(1) + 0xC)) == 0 {
        return t.skip("the installer's fonts are not there");
    }
    let mut regions = regions_of(list.item(1));
    regions.push(list.region());
    regions.push(region(SCRIPT_PATH, 0x102));
    let saved = Saved::new(&regions);
    for text in [
        &b"c:\\games"[..],
        b"",
        b"  \x01 ",
        b"0123456789abcdef0123456789abcde",
    ] {
        for width in [0x200u16, 30] {
            for eax in [
                1u32, 0x1C, 0x6E, 0xE, 0x66, 0x69, 0x10, 0x1E, 0x39, 0x0B, 0x9E, 0x141, 0x105,
            ] {
                let prepare = || {
                    list.set(0, &[4, 4], 0, 0, 0, 1, width, text);
                    let backup = a(SCRIPT_PATH);
                    for n in 0..0x22 {
                        wb(backup + n, b"backup text\0"[n as usize % 12]);
                    }
                };
                prepare();
                let before = Regs {
                    eax: 0x7700_0000 | eax,
                    esi: list.item(1),
                    edi: list.base(),
                    ..Regs::SENTINEL
                };
                check_original(
                    &mut t,
                    &format!(
                        "key {eax:#x}, width {width}, text {}",
                        crate::strings::utils::bytes_str(text)
                    ),
                    call_original,
                    before,
                    &regions,
                    CF | ZF,
                    &prepare,
                    &edit_key,
                );
            }
        }
    }
    drop(saved);
    t.finish()
}
