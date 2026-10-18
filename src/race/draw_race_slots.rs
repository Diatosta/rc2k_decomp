//! `race::draw_race_slots` (0x4621E5): `race::draw_slot_object` for each of the eight race
//! slots (0x78 bytes at `g_raceSlots`) in use (+0 not 0), inside `pusha`.
//!
//! In:  nothing
//! Out: ecx 8, esi past the slots; the rest kept (a `mov` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x45583F), each view drawn. Refuses
//! (before anything) a slot's car without its block. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose slot call goes to a stand-in that notes esi and
//! ecx, against the Rust version with the same; the slots' first dwords set; all put back.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::race::frame_draw::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_race_slots", rva(0x4621E5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let done =
        unsafe { draw_race_slots(r, &mut |s| draw_slot_object(s, &mut slot_part).map(|_| ())) };
    match done {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

static CALLED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn stand_in(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((r.esi, r.ecx));
    r.eax ^= 0x1234;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(stand_in);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4621E5,
            len: 0x21,
            entry: &[0x8D, 0x35, 0xC8, 0xBA, 0x71, 0x00, 0xB9, 0x00],
            calls: &[0x11],
            jumps_out: &[],
        },
        &|c| (c == rva(0x4626D1)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let slots = game_addr(rva(0x71BAC8)) as u32;
    let regions: Vec<Region> = (0..8).map(|n| (slots + 0x78 * n, 4)).collect();
    let saved = capture(&regions);
    for pattern in [0u32, 0xFF, 0x5A, 0x81] {
        let prepare = || {
            for n in 0..8u32 {
                // Objects below 0x50: no car's block is looked at.
                let v = if pattern >> n & 1 != 0 { 1 + n } else { 0 };
                unsafe { ((slots + 0x78 * n) as *mut u32).write_unaligned(v) };
            }
            CALLED.lock().unwrap().clear();
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (regs(&ro), std::mem::take(&mut *CALLED.lock().unwrap()));
        prepare();
        let mine = unsafe {
            draw_race_slots(&Regs::SENTINEL, &mut |s| {
                CALLED.lock().unwrap().push((s.esi, s.ecx));
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *CALLED.lock().unwrap()),
        );
        t.check(theirs == mine, || {
            format!("pattern {pattern:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
