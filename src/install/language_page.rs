//! `install::language_page` (0x414199): the language page. At the start (ebx 0) with only
//! one language (`g_languageCount` <= 1) or this version already installed (`dword_51B9C8`
//! 1), the list's `[edi]` gets 0x40 (the page is skipped). At the end (ebx negative) with a
//! chosen answer (`byte_51D415` != 0) the language is chosen (`install::choose_language`)
//! and loaded (`strings::load_language`). Then the items past the languages
//! (`g_languageCount` of them, up to 9) get 0x40 (hidden).
//!
//! In:  ebx the state, edi the list
//! Out: eax and what the callees leave; the flags of the hide loop (no caller reads them)
//! A page action. Caller `install::run_page`. Its two calls through `part`. Ungated
//! (loading the language reads files and compiles). Shadow mode runs the original alone.
//! Self-test: a copy whose two calls go to stand-ins, against the Rust version, every
//! state and language count.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CHOOSE_LANGUAGE, LANGUAGE_COUNT, LIST_ANSWER, LOAD_LANGUAGE, Part, SAME_VERSION, ZF, at, both,
    copy, g, gb, logic, quiet, rb, wb,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts};
use rc2k_core::install::pages;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::language_page", rva(0x414199), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub fn language_page(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if r.ebx == 0 {
        // The same version, or at most one language (unsigned): the item itself hidden
        // (`or byte ptr [edi], 40h`, its flags). Else on to the hide loop (`loc_4141B4`).
        // Converted: `rc2k_core::install::pages` (`language_page_skipped`,
        // `hidden_languages`).
        if pages::language_page_skipped(g(SAME_VERSION) == 1, g(LANGUAGE_COUNT)) {
            let v = rb(r.edi) | pages::HIDDEN;
            wb(r.edi, v);
            logic(&mut r, v as u32);
            return Ok(r);
        }
    } else if (r.ebx as i32) < 0 {
        r.eax = gb(LIST_ANSWER) as u32;
        if r.eax != 0 {
            part(at(CHOOSE_LANGUAGE), &mut r)?;
            part(at(LOAD_LANGUAGE), &mut r)?;
        }
    }
    // hide the items past the languages
    let (first, hidden) = pages::hidden_languages(g(LANGUAGE_COUNT));
    r.edx = first.wrapping_add(r.edi).wrapping_add(0x18);
    r.ecx = hidden;
    let c = r.ecx;
    logic(&mut r, c);
    if r.ecx != 0 {
        loop {
            wb(r.edx, rb(r.edx) | pages::HIDDEN);
            r.edx = r.edx.wrapping_add(0x10);
            r.ecx = r.ecx.wrapping_sub(1);
            if r.ecx == 0 {
                break;
            }
        }
        logic(&mut r, 0);
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        language_page(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // test ebx, ebx; jnz loc_4141B4
    let copy = match copy(
        0x414199,
        0x56,
        &[
            0x85, 0xDB, 0x75, 0x17, 0x83, 0x3D, 0xC8, 0xB9, 0x51, 0x00, 0x01,
        ],
        &[(0x2A, 0x41397A), (0x2F, 0x40331F)],
        &[0x41397A, 0x40331F],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let list = TestList::new();
    let regions = [list.region()];
    let saved = Saved::new(&regions);
    // what `prepare` writes besides the compared regions: put back at the end
    let _kept = crate::race::viewer::Saved::new(&[
        crate::install::installer::region(0x51B9C8, 4),
        crate::install::installer::region(0x51BEEA, 4),
        crate::install::installer::region(0x51D415, 1),
    ]);
    for same in [0u32, 1] {
        for count in [1u32, 3, 9] {
            for (ebx, answer) in [(0u32, 0u8), (1, 0), (0xFFFF_FFFF, 0), (0xFFFF_FFFF, 2)] {
                let prepare = || {
                    list.set(0, &[1, 1, 1, 1, 1, 1, 1, 1, 1], 0, 0, 0, 0, 0, b"");
                    crate::install::installer::p(SAME_VERSION, same);
                    crate::install::installer::p(LANGUAGE_COUNT, count);
                    crate::install::installer::pb(LIST_ANSWER, answer);
                };
                let before = Regs {
                    ebx,
                    edi: list.base(),
                    ..Regs::SENTINEL
                };
                both(
                    &mut t,
                    &format!("same {same}, count {count}, ebx {ebx:#x}, answer {answer}"),
                    &copy,
                    before,
                    &regions,
                    ZF,
                    &[],
                    &prepare,
                    &|r, part, _| language_page(r, part),
                );
            }
        }
    }
    drop(saved);
    t.finish()
}
