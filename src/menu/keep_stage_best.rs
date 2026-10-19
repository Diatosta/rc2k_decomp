//! `menu::keep_stage_best` (0x48E02D): when the current player's +0x94 is above (signed)
//! the chosen stage record's +0x2C, the record takes it: +0x2C that, +0x9C the player's
//! nationality (+4), +0xA0 the car (+8), and the 0x20 bytes of the player's name (at
//! +0x18) at +0x7C.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events, after the results. What +0x94 is is not read (a
//! figure of the stage's; the time trial's stage page may show it). Refuses (before
//! anything) a current player past the four or missing, a stage the tables do not give
//! (`event_flow::stage_record`), or a name pointer of 0 when the record is taken.
//! Self-test: the game's records for each stage choice there, the figure below, equal to
//! and above the record's, put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::event_flow::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::keep_stage_best", rva(0x48E02D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn keep_stage_best() -> Result<(), Refused> {
    let record = stage_record()?;
    let p = current_player()?;
    let figure = dword(p + 0x94);
    if (dword(record + 0x2C) as i32) >= figure as i32 {
        return Ok(());
    }
    let name = dword(p + 0x18);
    if name == 0 {
        return Err(BadInput("a player's name pointer of 0").into());
    }
    put(record + 0x2C, figure);
    put(record + 0x9C, dword(p + 4));
    put(record + 0xA0, dword(p + 8));
    unsafe { std::ptr::copy(name as *const u8, (record + 0x7C) as *mut u8, 0x20) };
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    keep_stage_best()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = stage_regions().unwrap_or_default();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(p) = current_player() else {
        return t.skip("no current player");
    };
    if dword(p + 0x18) == 0 {
        return t.skip("the current player has no name pointer");
    }
    let mut vars = [STAGE_KIND, STAGE_SUB, STAGE_NUMBER]
        .map(|v| at(rva(v)))
        .to_vec();
    vars.push((p + 0x94, 4));
    let saved_vars = capture(&vars);
    let choices = stage_choices();
    if choices.is_empty() {
        return t.skip("no stage tables");
    }
    for c in choices {
        choose_stage(c);
        let Some(regions) = stage_regions() else {
            continue;
        };
        let saved = capture(&regions);
        let best = dword(regions[0].0 + 0x2C);
        for figure in [
            best.wrapping_sub(1),
            best,
            best.wrapping_add(1),
            0x7FFF_FFFF,
        ] {
            check_state(
                &mut t,
                &format!("stage {c:x?}, record {best:#x}, figure {figure:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    restore(&regions, &saved);
                    choose_stage(c);
                    put(p + 0x94, figure);
                },
                &regions,
                &rust,
            );
        }
        restore(&regions, &saved);
    }
    restore(&vars, &saved_vars);
    t.finish()
}
