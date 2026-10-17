//! `marks::blend_corner` (0x4D0B4E): one point blended: its factor the record's float at
//! +0x90 + 4 x (the source point's three sign bits, x the highest; kept in
//! `flt_93C6C8`), each of x, y, z base + (source - base) by it, rounded (`fistp`).
//!
//! In:  edi the source point, edx the base point, esi where the result goes, ebp the
//!      record (a car's +0x29C)
//! Out: ebx the sign bits, eax the source's z doubled; the rest kept
//!
//! Caller: `marks::morph_object_points`. Self-test: points of the test's (every sign
//! pattern, large and small), factors by lot; compared on the result, `flt_93C6C8` and
//! the registers.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::blend_corner", rva(0x4D0B4E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const FACTOR_RVA: usize = rva(0x93C6C8); // flt_93C6C8

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn blend_corner(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let src = [get(g.edi), get(g.edi + 4), get(g.edi + 8)];
        g.ebx = src.iter().fold(0, |b, &v| (b << 1) | (v >> 31));
        g.eax = src[2].wrapping_add(src[2]);
        let f = x87::loaded(get(g.ebp + 0x90 + 4 * g.ebx));
        game_write(FACTOR_RVA, f);
        for (c, &s) in src.iter().enumerate() {
            let c = 4 * c as u32;
            let v = x87::int_lerp_to_int(s as i32, get(g.edx + c) as i32, f);
            ((g.esi + c) as *mut i32).write_unaligned(v);
        }
    }
    g
}

/// What it writes.
pub fn regions(dst: u32) -> Vec<Region> {
    vec![at(FACTOR_RVA), (dst, 12)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { blend_corner(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.esi), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = capture(&[at(FACTOR_RVA)]);
    let mut buffer = vec![0u32; 0x40];
    let b = buffer.as_mut_ptr() as u32;
    // The record's factors at +0x90 (8), the source at +0x100, the base +0x110, the out +0x120.
    for case in 0..12u32 {
        let src = [
            if case & 4 != 0 {
                -0x1_2345 - case as i32
            } else {
                0x4_0000 + case as i32
            },
            if case & 2 != 0 { -7 } else { 0x7FFF_0000 },
            if case & 1 != 0 {
                i32::MIN + case as i32
            } else {
                3
            },
        ];
        check_state(
            &mut t,
            &format!("source {src:x?}"),
            call_original,
            Regs {
                edi: b + 0x100,
                edx: b + 0x110,
                esi: b + 0x120,
                ebp: b,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                let p = b as *mut u32;
                for k in 0..8u32 {
                    *p.add((0x90 / 4 + k) as usize) =
                        ((k as f32 - 3.0) * 0.37 + case as f32 * 0.01).to_bits();
                }
                for (c, v) in src.iter().enumerate() {
                    *p.add(0x100 / 4 + c) = *v as u32;
                    *p.add(0x110 / 4 + c) = (c as i32 * 0x1_1111 - case as i32 * 77) as u32;
                    *p.add(0x120 / 4 + c) = 0xAAAA_AAAA;
                }
                game_write(FACTOR_RVA, 0xAAAA_AAAAu32);
            },
            &regions(b + 0x120),
            &rust,
        );
    }
    restore(&[at(FACTOR_RVA)], &saved);
    drop(buffer);
    t.finish()
}
