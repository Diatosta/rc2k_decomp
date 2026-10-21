//! `install::page_key` (0x4146E4): a key on the page's highlighted item. Nothing for no key
//! or one of 0x80 and above (`test al, al; jle`). An edit being edited takes every key
//! (`install::edit_key`); else 0x66, 0x6B, 0x68, 0x69 (`install::key_up`, `key_down`,
//! `key_left`, `key_right`), 2 .. 0xA (`install::key_number`), Enter (`install::item_enter`);
//! Escape leaves the page with answer 0 unless the list's flags have 0x10. After an edit's
//! key or any other key: 0x52 a screenshot (`gfx::take_screenshot`) with screenshots on.
//! Then a key that did something (zero set) sets `byte_51D414` and gives zero set.
//!
//! In:  eax the key (al; ah its bits), esi the item, edi the list
//! Out: zero set when the key did something; ebx (and what the callees leave) changed
//! Caller `install::run_page`, reading zero. Its calls through `part`. Ungated (Enter can
//! run an action). Shadow mode runs the original alone. Self-test: a copy whose eight calls
//! go to stand-ins (answering zero set or clear), against the Rust version, every key's way.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CF, KEY_TAKEN, LIST_ANSWER, LIST_LEAVE, Part, ZF, answer, at, both, cmp, copy, gb, logic, pb,
    quiet, rb, rd, region,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts};
use rc2k_core::install::items::{self, ItemKey, PageKey};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::page_key", rva(0x4146E4), thunk, &ORIGINAL)
    .private_copy(0xA0)
    .relocate_calls(&[0x1A, 0x25, 0x30, 0x3B, 0x46, 0x55, 0x60, 0x8E])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SCREENSHOTS_ON: usize = 0x4F52B4; // g_screenshotsOn
const CALLEES: [u32; 8] = [
    0x4148F7, // install::edit_key
    0x414784, // install::key_up
    0x414794, // install::key_down
    0x4147A4, // install::key_left
    0x4147C0, // install::key_right
    0x4147DC, // install::key_number
    0x4147FE, // install::item_enter
    0x407897, // gfx::take_screenshot
];

fn zero(r: &Regs) -> bool {
    r.eflags & ZF != 0
}

pub fn page_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let key = r.eax as u8;
    logic(&mut r, key as u32);
    // Converted: `rc2k_core::install::items` (`page_key`, `takes_every_key`, `STAYS`,
    // `SCREENSHOT_KEY`).
    let Some(way) = items::page_key(key) else {
        return Ok(r);
    };
    let kind = rb(r.esi) & 0xF;
    r.ebx = r.ebx & !0xFF | kind as u32;
    cmp(&mut r, kind as u32, 4);
    let edited = items::takes_every_key(kind, || {
        r.ebx = rd(r.esi + 0xC);
        let editing = rb(r.ebx) & 1 != 0;
        logic(&mut r, editing as u32);
        editing
    });
    if edited {
        part(at(CALLEES[0] as usize), &mut r)?;
    } else {
        let callee = match way {
            PageKey::Item(ItemKey::Up) => Some(1),
            PageKey::Item(ItemKey::Down) => Some(2),
            PageKey::Item(ItemKey::Left) => Some(3),
            PageKey::Item(ItemKey::Right) => Some(4),
            PageKey::Item(ItemKey::Number) => Some(5),
            PageKey::Enter => Some(6),
            _ => None,
        };
        if let Some(n) = callee {
            // the flags of the last `cmp al, ..` before the call
            let last = if n == 5 { 0xA } else { key as u32 };
            cmp(&mut r, key as u32, last);
            part(at(CALLEES[n] as usize), &mut r)?;
            return Ok(took(r));
        }
        if way == PageKey::Escape {
            let flags = rb(r.edi) & items::STAYS;
            logic(&mut r, flags as u32);
            if flags == 0 {
                pb(LIST_ANSWER, 0);
                pb(LIST_LEAVE, 1);
            }
            return Ok(r);
        }
        cmp(&mut r, key as u32, 1);
    }
    let flags = r.eflags;
    if gb(SCREENSHOTS_ON) != 0 && (r.eax as u8) == items::SCREENSHOT_KEY {
        part(at(CALLEES[7] as usize), &mut r)?;
    }
    r.eflags = flags;
    Ok(took(r))
}

/// `jnz` out, else the key marked as taken and zero set.
fn took(mut r: Regs) -> Regs {
    if zero(&r) {
        pb(KEY_TAKEN, 1);
        r.set_zero(true);
        r.set_carry(false);
    }
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        page_key(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<(usize, u32)> = [0x1A, 0x25, 0x30, 0x3B, 0x46, 0x55, 0x60, 0x8E]
        .iter()
        .zip(CALLEES)
        .map(|(&o, c)| (o, c))
        .collect();
    // test al, al; jle near
    let copy = match copy(
        0x4146E4,
        0xA0,
        &[0x84, 0xC0, 0x0F, 0x8E, 0x97, 0x00, 0x00, 0x00],
        &calls,
        &CALLEES,
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let list = TestList::new();
    let regions = [
        list.region(),
        region(KEY_TAKEN, 3),
        region(SCREENSHOTS_ON, 1),
    ];
    let saved = Saved::new(&regions);
    for kind in [1u8, 2, 4] {
        for editing in [false, true] {
            for flags in [0u8, 0x10] {
                for shots in [0u8, 1] {
                    for did in [false, true] {
                        for key in [
                            0u32, 0x80, 0x66, 0x6B, 0x68, 0x69, 2, 0xA, 0xB, 0x1C, 1, 0x52, 0x30,
                        ] {
                            if editing && kind != 4 {
                                continue;
                            }
                            let prepare = || {
                                list.set(flags, &[kind, kind], 1, 0, 0, editing as u8, 100, b"x");
                                pb(KEY_TAKEN, 0);
                                pb(LIST_ANSWER, 9);
                                pb(LIST_LEAVE, 0);
                                pb(SCREENSHOTS_ON, shots);
                                answer(Box::new(move |_, r| r.set_zero(did)));
                            };
                            let before = Regs {
                                eax: 0x5500 | key,
                                esi: list.item(1),
                                edi: list.base(),
                                ..Regs::SENTINEL
                            };
                            both(
                                &mut t,
                                &format!(
                                    "kind {kind}, editing {editing}, flags {flags:#x}, \
                                     screenshots {shots}, callee's zero {did}, key {key:#x}"
                                ),
                                &copy,
                                before,
                                &regions,
                                ZF,
                                &[],
                                &prepare,
                                &|r, part, _| page_key(r, part),
                            );
                        }
                    }
                }
            }
        }
    }
    quiet();
    let _ = CF;
    drop(saved);
    t.finish()
}
