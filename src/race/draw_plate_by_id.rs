//! `race::draw_plate_by_id` (0x41B2D4): string id ebx into the text buffer and drawn on slot eax's plate (`race::draw_name_plate`) (see `race/name_plates.rs`).
//!
//! In:  eax = the slot, ebx = the string id
//! Out: every register preserved
//!
//! Callers: `race::update_name_plates` (two). Shadow mode runs the original alone (it
//! writes a surface). Self-test: a copy whose plate call goes to a stand-in (the strings real),
//! against the Rust version with the same: string ids.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::name_plates::*;
use crate::race::plate_test::{FAIL, SIZE, SURFACE, handler};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_plate_by_id", rva(0x41B2D4), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_plate_by_id(r, &mut by_id_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_plate_by_id(r, &mut by_id_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x41B2D4,
            len: 0x1E,
            entry: &[0x53, 0x50, 0x8B, 0xC3, 0x8D, 0x1D, 0x30, 0xE4],
            calls: &[0x0B, 0x10, 0x17],
            jumps_out: &[],
        },
        &|c| {
            [(0x45B695usize, 1u32)]
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
        let ebx = [1u32, 0x20, 0x1003][n];
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
                draw_plate_by_id(r, &mut |p, r| {
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
