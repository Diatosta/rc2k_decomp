//! `net::begin_edit` (0x416E1A): an item's text starts being edited: the item (`dword_52A4B8`),
//! the text (`dword_52A4B4`), the widest it may be (`dword_52A4BC`) and the font it is
//! measured with (`dword_52A4C0`) noted, and the text's 64 bytes kept (`byte_95EAF8`) for
//! Escape.
//!
//! In:  eax = the item, ebx = the text, ecx = the width, edx = the font
//! Out: every register kept (`pusha` .. `popa`)
//! Callers: `net::begin_setting_edit`, `net::begin_chat`. Shadow mode: both run, the five
//! variables compared. Self-test: a copy against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    EDIT_FONT, EDIT_SAVED, EDIT_TEXT, EDIT_WIDTH, EDITED, Part, Saved, a, fill, movsb, p, region,
    run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::begin_edit", rva(0x416E1A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves (the entry's).
pub fn begin_edit(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    p(EDITED, entry.eax);
    p(EDIT_TEXT, entry.ebx);
    p(EDIT_WIDTH, entry.ecx);
    p(EDIT_FONT, entry.edx);
    movsb(a(EDIT_SAVED), entry.ebx, 0x40);
    Ok(*entry)
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    vec![region(EDIT_TEXT, 0x10), region(EDIT_SAVED, 0x40)]
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &regions(), 0, &|r| {
        begin_edit(r, &mut crate::net::lobby::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov dword_52A4B8, eax
    let copy = match crate::net::lobby::copy(
        0x416E1A,
        0x29,
        &[0x60, 0xA3, 0xB8, 0xA4, 0x52, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = regions();
    let saved = Saved::new(&all);
    let text: Vec<u8> = (0..0x40u8).map(|n| n ^ 0x5A).collect();
    for item in [0u32, 3] {
        run_both(
            &mut t,
            &format!("item {item}"),
            copy.addr(),
            Regs {
                eax: item,
                ebx: text.as_ptr() as u32,
                ecx: 0x30C,
                edx: 0x0051_E624,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                fill(EDIT_TEXT, 0x10, 4);
                fill(EDIT_SAVED, 0x40, 5);
            },
            &begin_edit,
        );
    }
    drop(saved);
    t.finish()
}
