//! `race::draw_car` (0x4616AA): car esi drawn: marks, matrices, the model's parts' bits and
//! the shown mask, the light, the body and nearer its four parts (each
//! `scene::draw_scene_object`); see `race/car_draw.rs`.
//!
//! In:  esi = the car's block (`dword_60627C` its number)
//! Out: the registers as `race::clear_car_marks` and the rest leave them (its caller returns)
//!
//! It begins with its call: private copy (0x7F7, its fifteen calls relocated), manual patch.
//! Refuses (before anything) in a race the drawn car without its block. Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose drawing calls go to a stand-in noting
//! `g_objectNumber` (the rest real), against the Rust version with the same: a copy of car
//! 0's block, its state bits, switches, depths and the drawn view's own car varied; all put
//! back (skipped without a race).

use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::car_draw::*;
use crate::race::face_stage::{SEEN_RVA, compare_plain, seen_handler};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::draw_car", rva(0x4616AA), thunk, &ORIGINAL)
    .private_copy(0x7F7)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLS: [usize; 15] = [
    0x0, 0x6D9, 0x743, 0x748, 0x74D, 0x752, 0x77F, 0x784, 0x79B, 0x7A0, 0x7B7, 0x7BC, 0x7D3, 0x7D8,
    0x7F1,
];

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { draw_car(r, &mut car_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let first: u32 = unsafe { game_read(rva(0x71B3B8)) };
    if first == 0 || unsafe { game_read::<u32>(rva(0x621480)) } == 0 {
        return t.skip("no race loaded");
    }
    SEEN_RVA.store(rva(0x6214FC), Ordering::SeqCst);
    crate::install::stand_ins::set_handler(seen_handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4616AA,
            len: 0x7F7,
            entry: &[0xE8, 0x8E, 0x09, 0x00, 0x00, 0x51, 0x50, 0x57],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|c| (c == rva(0x451E3E)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut car = vec![0u8; 0x1000];
    unsafe { std::ptr::copy(first as *const u8, car.as_mut_ptr(), 0xFE0) };
    let c = car.as_mut_ptr() as u32;
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut regions: Vec<Region> = vec![
        g(0x6417E0, 0x100),
        g(0x5E9F8C, 0x538),
        g(0x641760, 0x40),
        g(0x5CCE1C, 0x64),
        g(0x608CA8, 0xC),
        g(0x606280, 8),
        g(0x71BA08, 8),
        g(0x635084, 0x18),
        g(0x63501C, 0x10),
        g(0x5CD688, 0x100),
        g(0x624148, 0x18),
        g(0x624098, 0x1C),
        g(0x6399B8, 0x1000),
        (c + 0xF04, 4),
    ];
    for a in [
        0x6D18D4usize,
        0x5E9EC8,
        0x62150C,
        0x6415A4,
        0x62400C,
        0x5CCE74,
        0x6365C8,
        0x62DAF4,
        0x635038,
        0x635054,
        0x62DAF8,
        0x6214FC,
        0x5CDA88,
        0x63D9B8,
    ] {
        regions.push(at(rva(a)));
    }
    let mut kept = regions.clone();
    for a in [0x60627Cusize, 0x606270, 0x60626C] {
        kept.push(at(rva(a)));
    }
    let saved = capture(&kept);
    for (state, switches, depth, stage, own) in [
        (0u32, 0x1E0u32, 0x1_0000u32, 0u32, 0u32),
        (2, 0x3_F3E0, 0x15_0000, 5, 0),
        (4 | 0x100, 0x0_2020, 0x29_0000, 0x40, 1),
        (0x10, 0x1E0, 0x3D_0000, 3, 0),
        (0x100, 0xFFFF_FFFF, 0x8_0000, 0x33, 1),
    ] {
        let line = compare_plain(
            &format!(
                "state {state:#x} switches {switches:#x} depth {depth:#x} stage {stage} own {own}"
            ),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x60627C), 0u32);
                game_write(rva(0x606270), stage);
                game_write(rva(0x60626C), own);
                game_write(rva(0x6415A4), depth);
                ((c + 0x6FC) as *mut u32).write_unaligned(state);
                ((c + 0x298) as *mut u32).write_unaligned(switches);
            },
            &regions,
            copy.addr(),
            Regs {
                esi: c,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                draw_car(r, &mut |p, r| {
                    seen_handler(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    SEEN_RVA.store(0, Ordering::SeqCst);
    drop(car);
    t.finish()
}
