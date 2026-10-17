//! `marks::make_masks` (0x4BDE9A): in `sub_4C0F2E`, the shape sets, corner tables, rings,
//! masks and pixel tables of the two shapes, in a work block taken and freed.
//!
//! In:  nothing
//! Out: eax the spans made, ebx the work block's list, the rest as the last step
//!
//! See `marks/masks.rs`. One caller, `sub_4C0F2E`. Shadow mode runs the original alone
//! (memory cannot be taken twice). Refuses as its steps do, mid-way. Self-test: a
//! redirected copy whose ten calls go to stand-ins noting every register, against the Rust
//! version on the same stand-ins: the calls, the registers and the variables it writes.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::marks::masks::*;
use crate::race::level_init::GIVES;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_masks", rva(0x4BDE9A), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { make_masks(r, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry: &[u8] = &[0xB8, 0x00, 0x00, 0x08, 0x00];
    let copy = match crate::race::mud::copy(
        0x4BDE9A,
        0xD2,
        entry,
        &[
            0xB, 0x23, 0x28, 0x2D, 0x48, 0x63, 0x74, 0x85, 0x9C, 0xB3, 0xCC,
        ],
        &[
            (0x4028A7, 0),
            (0x4BD019, 1),
            (0x4BEB25, 2),
            (0x4BE0A3, 3),
            (0x4BE469, 4),
            (0x4BE6C2, 5),
            (0x4BE752, 6),
            (0x4028F5, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note_step);
    let regions: Vec<Region> = vec![
        at(WORK_RVA),
        at(crate::marks::rings::CORNERS_RVA),
        at(CURSOR_RVA),
    ];
    let saved = capture(&regions);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (n, block) in [0x1234_0000u32, 0x0040_0000].into_iter().enumerate() {
        let prepare = || unsafe {
            restore(&regions, &saved);
            crate::framework::game_write(CURSOR_RVA, block + 0x88 * (n as u32 + 1));
            GIVES.store(block, Ordering::SeqCst);
            NOTED.lock().unwrap().clear();
        };
        prepare();
        let mut theirs = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut theirs, copy.addr()) };
        let theirs = (
            regs(&theirs),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        prepare();
        let mine = unsafe { make_masks(&Regs::SENTINEL, &mut test_step) };
        let mine = (
            mine.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!("block {block:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
