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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    KEY_TAKEN, LIST_ANSWER, LIST_LEAVE, Part, ZF, at, cmp, gb, logic, pb, rb, rd,
};
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::{self, ItemKey, PageKey};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        page_key(r, &mut crate::install::installer::game_part)
    })
}
