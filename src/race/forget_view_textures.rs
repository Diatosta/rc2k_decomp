//! `race::forget_view_textures` (0x45B200): the views' cars' changing textures (number
//! plates, dirt, the two extra slots) forgotten so they load again, their bits in
//! `dword_6415F4`; with the mirror on its picture copied or uploaded (see
//! `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) a null view block or a slot past 0x3FF. The upload
//! (`sub_44E828`) is the platform's (`call_game`). Caller: `sub_45429B` (0x45443D). Shadow
//! mode runs the original alone (the mirror's copy writes a surface). Self-test: a copy whose
//! two calls out go to stand-ins (the forgetting real), against the Rust version with the
//! same: test view blocks, each switch on and off, the mirror each way.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{forget_view_textures, view_textures_part};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::forget_view_textures",
    rva(0x45B200),
    thunk,
    &ORIGINAL,
)
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
    if let Err(e) = unsafe { forget_view_textures(r, &mut view_textures_part) } {
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
            address: 0x45B200,
            len: 0x157,
            entry: &[0xC7, 0x05, 0xF4, 0x15, 0x64, 0x00, 0x00, 0x00],
            calls: &[0x1E, 0x48, 0x72, 0xA2, 0xCC, 0xF0, 0x12B, 0x135, 0x147],
            jumps_out: &[],
        },
        &|c| {
            [(0x455DADusize, 1u32), (0x44E828, 2)]
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
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions: Vec<Region> = vec![
        at(rva(0x6415F4)),
        at(rva(0x6D1508)),
        g(0x5DB968, 0x1000),
        (b0 + 0x720, 4),
        (b1 + 0x720, 4),
    ];
    let mut kept = regions.clone();
    for a in [
        0x63DBB0usize,
        0x63DBB4,
        0x6D1830,
        0x6D1834,
        0x6415E0,
        0x6415E8,
        0x5D1E00,
        0x5D1E04,
        0x5D1DF8,
        0x5D1DFC,
        0x5D1E58,
        0x5D1E5C,
        0x5D0E08,
    ] {
        kept.push(at(rva(a)));
    }
    kept.push((game_addr(rva(0x4F6B74)) as u32, 1));
    let saved = capture(&kept);
    for (switches, mirror, uploaded) in [
        (0u32, (0u32, 0u32), 0u8),
        (0x3F, (1, 0), 0),
        (0x15, (1, 0), 1),
        (0x2A, (1, 1), 0),
        (0x01, (0, 0), 1),
    ] {
        let line = compare_plain(
            &format!("switches {switches:#x} mirror {mirror:?} uploaded {uploaded}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x63DBB0), b0);
                game_write(rva(0x63DBB4), b1);
                let on = |bit: u32| u32::from(switches & bit != 0);
                ((b0 + 0xADC) as *mut u32).write_unaligned(on(1));
                ((b1 + 0xADC) as *mut u32).write_unaligned(on(2));
                ((b0 + 0x720) as *mut u32).write_unaligned(on(4) * 3);
                ((b1 + 0x720) as *mut u32).write_unaligned(on(8) * 7);
                game_write(rva(0x6D1830), on(0x10));
                game_write(rva(0x6D1834), on(0x20));
                for (k, a) in [
                    0x5D1E00usize,
                    0x5D1E04,
                    0x5D1DF8,
                    0x5D1DFC,
                    0x5D1E58,
                    0x5D1E5C,
                    0x5D0E08,
                ]
                .into_iter()
                .enumerate()
                {
                    game_write(rva(a), 0x40 + 3 * k as u32);
                }
                game_write(rva(0x6415E0), mirror.0);
                game_write(rva(0x6415E8), mirror.1);
                (game_addr(rva(0x4F6B74)) as *mut u8).write(uploaded);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                forget_view_textures(r, &mut |p, r| {
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
