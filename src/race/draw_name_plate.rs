//! `race::draw_name_plate` (0x45B695): slot eax's plate: three lines cleared, text ebx drawn three times across them (see `race/name_plates.rs`).
//!
//! In:  eax = the slot, ebx = the text
//! Out: every register preserved
//!
//! It begins with its lock's call: private copy (0x75, its calls relocated), manual
//! patch. The surface's lock and unlock are the platform's (section 15); the text is
//! `text::draw_text_keep_regs`. Callers: `race::update_name_plates` (two), `race::draw_plate_by_id`. Shadow mode runs the original alone (it
//! writes a surface). Self-test: a copy whose lock, unlock and text calls go to stand-ins
//! (`race/plate_test.rs`: a test surface), against the Rust version with the same: line
//! heights, a failed lock.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::name_plates::*;
use crate::race::plate_test::{FAIL, SIZE, SURFACE, handler};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::draw_name_plate", rva(0x45B695), thunk, &ORIGINAL)
    .private_copy(0x75)
    .relocate_calls(&[0x01, 0x54, 0x5E, 0x68, 0x6E])
    .manual_patch()
    .self_test(self_test);

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
    if let Err(e) = unsafe { draw_name_plate(r, &mut plate_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45B695,
            len: 0x75,
            entry: &[0x60, 0xE8, 0xB0, 0xF3, 0xFB, 0xFF, 0x72, 0x6B],
            calls: &[0x01, 0x54, 0x5E, 0x68, 0x6E],
            jumps_out: &[],
        },
        &|c| {
            [(0x41AA4Busize, 1u32), (0x41AB02, 2), (0x402303, 3)]
                .iter()
                .find(|(a, _)| rva(*a) == c)
                .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut surface = vec![0xAAu8; SIZE];
    let s = surface.as_mut_ptr() as u32;
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions: Vec<Region> = vec![
        (s, SIZE),
        g(0x51E304, 0x40),
        g(0x93E430, 0x100),
        g(0x4F49B4, 0x14),
    ];
    let mut kept = regions.clone();
    kept.extend([at(rva(0x51E2CC))]);
    let saved = capture(&kept);
    for (n, (line, fail)) in [(5u32, false), (12, false), (3, true)]
        .into_iter()
        .enumerate()
    {
        let ebx = {
            let _ = n;
            game_addr(rva(0x93E430)) as u32
        };
        let what = compare_plain(
            &format!("line {line} fail {fail} ebx {ebx:#x}"),
            &|| unsafe {
                restore(&kept, &saved);
                (s as *mut u8).write_bytes(0xAA, SIZE);
                game_write(rva(0x51E2CC), line);
                SURFACE.store(s, Ordering::SeqCst);
                FAIL.store(fail, Ordering::SeqCst);
            },
            &regions,
            copy.addr(),
            Regs {
                eax: 7,
                ebx,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                draw_name_plate(r, &mut |p, r| {
                    handler(p, r);
                    Ok(())
                })
            },
        );
        t.check(what.is_none(), || what.unwrap_or_default());
    }
    restore(&kept, &saved);
    drop(surface);
    t.finish()
}
