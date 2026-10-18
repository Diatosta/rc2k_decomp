//! `race::car_number_textures` (0x46203D): car esi's marks' textures (lamps' sides, the
//! model's letter, its number's digits, the stage's plate, the 17 decal bytes); see
//! `race/car_draw.rs`.
//!
//! In:  esi = the car's block
//! Out: eax, ebx, ecx, edx, ebp as it leaves them (its caller saves eax, ecx, edi, esi)
//!
//! Caller: `sub_4616AA`. Shadow mode: by state. Self-test: a test block (a copy of car 0's
//! when there is one), +0x1B8 / +0x1BC, +0x718, +0xAA8, +0x310 and the stage varied.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::car_draw::car_number_textures;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::car_number_textures", rva(0x46203D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_number_textures(r) }))
}

fn regions() -> Vec<Region> {
    vec![
        (game_addr(rva(0x5CD688)) as u32, 0x100),
        at(rva(0x5CDA88)),
        at(rva(0x62400C)),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(rva(0x623850)));
    let saved = capture(&kept);
    let mut car = vec![0u8; 0x1000];
    let first: u32 = unsafe { game_read(rva(0x71B3B8)) };
    if first != 0 {
        unsafe { std::ptr::copy(first as *const u8, car.as_mut_ptr(), 0xFE0) };
    }
    for (k, b) in car[0xAC0..0xAD1].iter_mut().enumerate() {
        *b = 0x40 + k as u8;
    }
    let c = car.as_mut_ptr() as u32;
    let model = unsafe { ((c + 0x24) as *const u32).read_unaligned() } % 16;
    for (sides, letter, number, below, stage) in [
        ((1.0f32, 9.0f32), 100.0f32, 7u32, 1.0f32, 3u32),
        ((-6.0, 2.0), 3000.4, 42, 500.0, 15),
        ((5.0, -5.0), -20.0, 0, 50.0, 0x60),
        ((f32::NAN, 0.0), 99999.0, 123, f32::NAN, 0x2F),
    ] {
        check_state(
            &mut t,
            &format!(
                "sides {sides:?} letter {letter} number {number} below {below} stage {stage:#x}"
            ),
            call_original,
            Regs {
                esi: c,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x623850), stage);
                let w = |o: u32, v: u32| ((c + o) as *mut u32).write_unaligned(v);
                w(0x24, model);
                w(0x1B8, sides.0.to_bits());
                w(0x1BC, sides.1.to_bits());
                w(0x718, letter.to_bits());
                w(0xAA8, number);
                w(0x310, below.to_bits());
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(car);
    t.finish()
}
