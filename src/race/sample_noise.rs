//! `race::sample_noise` (0x44FD38): a 64 x 64 byte map sampled at the work car's place
//! (`g_car` +0x6A4 and +0x6AC: the top byte the cell, the next two the fraction), blended
//! between its four corners (`dword_637620` .. `dword_637634`; `imul` and `shrd` by 16),
//! shifted left 8.
//!
//! In:  esi = the map (`g_noiseBytes` from `race::fog_table`)
//! Out: eax the value, ebx and ecx the fractions, edx the last product's high half
//!
//! See `race/frame_draw.rs`. Callers: `race::fog_table` (0x44FC3E) and `sub_43BC68`
//! (0x43BC6E, 0x43BCB1; `mov` follows). Shadow mode: by state. Self-test: a test map, the
//! work car's place set (cells at the edges, fractions); all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::sample_noise", rva(0x44FD38), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn noise_regions() -> Vec<Region> {
    vec![(game_addr(rva(0x637620)) as u32, 0x18)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { sample_noise(r.esi) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &noise_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let map: Vec<u8> = (0..0x1000u32)
        .map(|k| (k.wrapping_mul(0x9E37_79B9) >> 24) as u8)
        .collect();
    let car = game_addr(rva(0x71A67C)) as u32;
    let regions = noise_regions();
    let mut kept = regions.clone();
    kept.extend([(car, 4), (car + 8, 4)]);
    let saved = capture(&kept);
    for (x, z) in [
        (0u32, 0u32),
        (0x3F12_3456, 0x0180_0000),
        (0xFFFF_FFFF, 0x3FFF_FF00),
        (0x1234_5678, 0x9ABC_DEF0),
    ] {
        check_state(
            &mut t,
            &format!("place {x:#x} {z:#x}"),
            call_original,
            Regs {
                esi: map.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                (car as *mut u32).write_unaligned(x);
                ((car + 8) as *mut u32).write_unaligned(z);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(map);
    t.finish()
}
