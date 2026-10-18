//! `race::draw_car_shined` (0x461669): `g_shine` for the car by `dword_623D80` and
//! `dword_60AD0C`, the car drawn (`race::draw_car`), `g_shine` put back (see
//! `race/car_draw.rs`).
//!
//! In:  esi = the car's block
//! Out: eax, edx the shine's product, then as `draw_car` leaves them (callers return)
//!
//! Callers: `race::draw_car_entry`, 0x455B38. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose call goes to a stand-in noting `g_shine`, against the
//! Rust version with the same: distances in and out of the range.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::car_draw::*;
use crate::race::face_stage::{SEEN_RVA, compare_plain, seen_handler};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_car_shined", rva(0x461669), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_car_shined(r, &mut shined_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    SEEN_RVA.store(rva(0x5CCE74), Ordering::SeqCst);
    crate::install::stand_ins::set_handler(seen_handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x461669,
            len: 0x41,
            entry: &[0xFF, 0x35, 0x74, 0xCE, 0x5C, 0x00, 0xB8, 0xE0],
            calls: &[0x35],
            jumps_out: &[],
        },
        &|c| (c == rva(0x4616AA)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions: Vec<Region> = vec![at(rva(0x5CCE74))];
    let mut kept = regions.clone();
    kept.extend([at(rva(0x623D80)), at(rva(0x60AD0C))]);
    let saved = capture(&kept);
    for (distance, scale) in [
        (0u32, 0x100u32),
        (0x1_0000, 0x2345),
        (0x4_0000, 0xFFFF_FFFF),
        (0xFFFF_0000, 0x100),
        (0x8000_0000, 7),
    ] {
        let line = compare_plain(
            &format!("distance {distance:#x} scale {scale:#x}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x623D80), distance);
                game_write(rva(0x60AD0C), scale);
                game_write(rva(0x5CCE74), 0x1234u32);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                draw_car_shined(r, &mut |p, r| {
                    seen_handler(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    SEEN_RVA.store(0, Ordering::SeqCst);
    t.finish()
}
