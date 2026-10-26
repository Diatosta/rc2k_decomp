//! `race::draw_list_entry` (0x454FF0): one sort or depth entry drawn by its second word
//! (see `race/frame_draw.rs`): a course face (kind 0xFF by `scene::draw_described_object`,
//! below 0x10 by `race::draw_tinted_face`), a
//! flare (`race::draw_flare`), a car (`race::draw_car_entry`) or a lamp
//! (`race::draw_lamp_flare`); `dword_6365C8` 0 after.
//!
//! In:  ebx = the word
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `race::draw_sorted_entries`, `race::draw_depth_entries`. Refuses (before
//! anything) a word of 0x4000000 or more. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose five drawing calls go to stand-ins that note eax .. ecx,
//! against the Rust version on the same; test faces (`g_courseFaces` pointed at them),
//! each kind of word; all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::frame_draw::*;
use crate::race::level_init::{note, take_called};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_list_entry", rva(0x454FF0), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_list_entry(r, &mut entry_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_list_entry(r, &mut entry_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x454FF0,
            len: 0x115,
            entry: &[0x60, 0x89, 0x1D, 0xD4, 0xCC, 0x5C, 0x00, 0x81],
            calls: &[0x7C, 0xBF, 0xF4, 0x101, 0x10E],
            jumps_out: &[],
        },
        &|c| {
            ENTRY_CALLEES
                .iter()
                .find(|&&(_, k)| rva(k) == c)
                .map(|&(p, _)| crate::install::stand_ins::stand_in(p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut faces = vec![0u8; 0x40 * 4];
    let base = faces.as_mut_ptr() as u32;
    for (f, kind) in [0xFFu8, 0x05, 0x12, 0x0F].iter().enumerate() {
        faces[0x40 * f] = *kind;
    }
    let regions: Vec<Region> = vec![
        at(rva(0x5CCCD4)),
        at(CURRENT_FACE_RVA),
        (game_addr(rva(0x63E164)) as u32, 0xC),
        at(rva(0x6365C8)),
    ];
    let mut kept = regions.clone();
    kept.push(at(rva(0x5CCCBC)));
    let saved = capture(&kept);
    for word in [
        0u32,
        1,
        2,
        3,
        0x8000_0001,
        0x100_0005,
        0x200_0003,
        0x300_000F,
        0x8300_0002,
    ] {
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x5CCCBC), base);
            game_write(rva(0x6365C8), 2u32);
            take_called();
        };
        let entry = Regs {
            ebx: word,
            ..Regs::SENTINEL
        };
        prepare();
        let mut ro = entry;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (regs(&ro), take_called(), capture(&regions));
        prepare();
        let done = unsafe {
            draw_list_entry(&entry, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (regs(&entry), take_called(), capture(&regions));
        t.check(done.is_ok() && theirs == mine, || {
            format!("word {word:#x}: original {theirs:x?}; rust {mine:x?} {done:?}")
        });
    }
    restore(&kept, &saved);
    drop(faces);
    t.finish()
}
