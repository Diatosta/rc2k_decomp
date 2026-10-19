//! The event pages' row steps (agent E, 2026-10-19): the first row shown
//! (`g_champFirstRow`, 0x7556B8) ten back or on, then `menu::stage_range_caption`. Not
//! `pusha`'d. `last` is `dword_75597C` - 9 (`menu::previous_stage_rows` 0x4A3736,
//! `next_stage_rows` 0x4A375F) or 0x5A - 9 (`previous_stage_rows_of_90` 0x4A378C,
//! `next_stage_rows_of_90` 0x4A37B5).
//! - back: at 10 or below (signed) the row is `last` (eax it), else ten less;
//! - on: eax `last`; at `last` or past (signed) the row is 1, else ten more.
//!
//! Left/right handlers of the event pages' items (menu tables 0x74FAFC ..). Compared in
//! shadow mode (the row and the caption). Self-test: the original and the Rust version,
//! rows around both ends.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};

const FIRST_RVA: usize = rva(0x7556B8); // g_champFirstRow
const COUNT_RVA: usize = rva(0x75597C); // dword_75597C
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText

/// The Rust version: `of_90` the fixed count, `on` the direction.
pub fn step_rows(entry: &Regs, of_90: bool, on: bool) -> Result<Left, Refused> {
    unsafe {
        let count = if of_90 {
            0x5A
        } else {
            game_read::<u32>(COUNT_RVA)
        };
        let last = count.wrapping_sub(9);
        let row: u32 = game_read(FIRST_RVA);
        let mut left = Left::nothing();
        let row = if on {
            left = left.with(EAX, last);
            if row as i32 >= last as i32 {
                1
            } else {
                row.wrapping_add(10)
            }
        } else if row as i32 <= 10 {
            left = left.with(EAX, last);
            last
        } else {
            row - 10
        };
        game_write(FIRST_RVA, row);
        crate::menu::stage_range_caption::stage_range_caption(entry)?;
        Ok(left)
    }
}

fn regions() -> [Region; 2] {
    [at(FIRST_RVA), (game_addr(CAPTION_RVA) as u32, 0x20)]
}

pub fn dispatch_rows(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    of_90: bool,
    on: bool,
) {
    dispatch_state(hook, r, call_original, &regions(), &|r| {
        step_rows(r, of_90, on)
    });
}

pub fn test_rows(
    hook: &RawHook,
    call_original: CallOriginal,
    of_90: bool,
    on: bool,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let inputs = [at(COUNT_RVA)];
    let saved = (capture(&regions()), capture(&inputs));
    for count in [0x5Au32, 0x1E, 9] {
        for row in [1u32, 10, 11, 0x50, 0x51, 0x52, 0xFFFF_FFF0] {
            check_state(
                &mut t,
                &format!("count {count:#x}, row {row:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    game_write(COUNT_RVA, count);
                    game_write(FIRST_RVA, row);
                },
                &regions(),
                &|r| step_rows(r, of_90, on),
            );
        }
    }
    restore(&regions(), &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
