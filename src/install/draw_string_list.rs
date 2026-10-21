//! `install::draw_string_list` (0x402370): a list of `eax` records (16 bytes each: dx, dy,
//! font, string id) drawn, each at (dx + x, dy + y) with its font (`text::draw_string_by_id`).
//! The x and y come in ecx and edx (the caller's, read back from the saved slots).
//!
//! In:  eax the count, ebx the table, ecx x, edx y
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files` and the copy-progress and page frames. The draws
//! through `part`. Refuses a count of 0 (the original's loop is a do-while: it would draw
//! 2^32 records). Shadow mode runs the original alone (drawing twice would double it).
//! Self-test: a copy whose draw goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{DRAW_STRING, Part, at, both, quiet, rd};
use crate::race::viewer::dispatch_parts;
use rc2k_core::install::layout;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::draw_string_list", rva(0x402370), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn draw_string_list(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (x, y) = (entry.ecx, entry.edx);
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (offset, left) in layout::list_records(entry.eax)? {
        let record = entry.ebx.wrapping_add(offset);
        let (x, y) = layout::list_place(rd(record), rd(record + 4), x, y);
        let mut r = Regs {
            ecx: x,
            edx: y,
            ebx: rd(record + 8),
            eax: rd(record + 0xC),
            edi: record.wrapping_add(layout::LIST_RECORD),
            ebp: left,
            ..*entry
        };
        part(at(DRAW_STRING), &mut r)?;
    }
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        draw_string_list(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov edi, ebx; mov ebp, eax
    let copy = match crate::install::installer::copy(
        0x402370,
        0x25,
        &[0x60, 0x8B, 0xFB, 0x8B, 0xE8],
        &[(0x1B, 0x40234A)],
        &[0x40234A],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let mut table = [0u8; 0x40];
    for (n, v) in [10u32, 20, 0x100, 0x94, 30, 40, 0x101, 0x1000]
        .iter()
        .enumerate()
    {
        table[4 * n..4 * n + 4].copy_from_slice(&v.to_le_bytes());
    }
    for count in [1u32, 2] {
        both(
            &mut t,
            &format!("count {count}"),
            &copy,
            Regs {
                eax: count,
                ebx: table.as_ptr() as u32,
                ecx: 100,
                edx: 200,
                ..Regs::SENTINEL
            },
            &[],
            0,
            &[],
            &|| {},
            &|r, part, _| draw_string_list(r, part),
        );
    }
    t.finish()
}
