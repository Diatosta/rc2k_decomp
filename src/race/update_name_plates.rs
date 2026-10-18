//! `race::update_name_plates` (0x45B3B9): the time plate drawn again when its texts
//! changed, and each view's asked-for name drawn on its plate (by string id or text); see
//! `race/name_plates.rs`.
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) a null view block. Caller: `sub_4547AE` (0x45479A). Shadow
//! mode runs the original alone (it writes surfaces). Self-test: a copy whose three calls go
//! to stand-ins, against the Rust version with the same: test view blocks, names by id, by
//! text and none, the texts changed or not.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::race::name_plates::{update_name_plates, update_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::update_name_plates", rva(0x45B3B9), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { update_name_plates(r, &mut update_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45B3B9,
            len: 0x12D,
            entry: &[0xA1, 0x38, 0x18, 0x6D, 0x00, 0x3B, 0x05, 0x50],
            calls: &[0x39, 0xA3, 0xAA, 0xF6, 0xFD],
            jumps_out: &[],
        },
        &|c| {
            [(0x45B70Ausize, 1u32), (0x41B2D4, 2), (0x45B695, 3)]
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
    let mut blocks = vec![0u8; 0x2000];
    let b0 = blocks.as_mut_ptr() as u32;
    let b1 = b0 + 0x1000;
    let regions: Vec<Region> = vec![
        (game_addr_of(0x6D1850), 0x14),
        (b0 + 0x71C, 0x10),
        (b1 + 0x71C, 0x10),
    ];
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x63DBB0)),
        at(rva(0x63DBB4)),
        (game_addr_of(0x6D1838), 0x10),
        at(rva(0x5D0DF8)),
        at(rva(0x5D0DFC)),
    ]);
    let saved = capture(&kept);
    for (names, changed) in [
        ((0u32, 0u32), false),
        ((5, 0x7_0000), true),
        ((0x1_0000, 0xFFFF), false),
        ((0, 3), true),
    ] {
        let line = compare_plain(
            &format!("names {names:x?} changed {changed}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x63DBB0), b0);
                game_write(rva(0x63DBB4), b1);
                game_write(rva(0x5D0DF8), 0x31u32);
                game_write(rva(0x5D0DFC), 0x32u32);
                game_write(rva(0x6D1838), *b"1:23.45 \0\0\0\x0012.3");
                let copy: [u8; 16] = if changed {
                    *b"1:23.46 \0\0\0\x0012.3"
                } else {
                    *b"1:23.45 \0\0\0\x0012.3"
                };
                game_write(rva(0x6D1850), copy);
                ((b0 + 0x71C) as *mut u32).write_unaligned(names.0);
                ((b1 + 0x71C) as *mut u32).write_unaligned(names.1);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                update_name_plates(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    drop(blocks);
    t.finish()
}

fn game_addr_of(a: usize) -> u32 {
    crate::framework::game_addr(rva(a)) as u32
}
