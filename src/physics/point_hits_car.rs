//! `physics::point_hits_car` (0x45E800): whether a point of an object lies inside the
//! work car's box. `dword_614110` + 1; the point (three whole numbers at esi) plus the
//! object's place (ebp +0x30 ..) plus the offset `dword_6D3B40` .. less the car's place
//! (`unk_71A10C` ..) into `dword_6D3B4C` ..; that / `flt_5C5D98` into `dword_6D3B78` ..
//! and `flt_6D3B58` .., turned by the car's heading (`physics::turn_by_heading`, ebx the
//! y) and back to whole numbers (x `flt_5C5D98`, `fistp`) into `dword_6D3090` ... Inside
//! when x is below `dword_614BC8` and not below minus it, and z below `dword_614BC0` and
//! not below minus `dword_614BC4` (each by the sign of the `cmp`, as the original). Inside
//! with the contact kind `dword_6D3624` 10, 1, 5 or 6 also: the point's x, 0, z (/
//! `flt_5C5D98`) into `unk_71A1FC` .., the car's speed's length (`unk_71A170`,
//! `physics::vector_length`) into `flt_6D3108`, `unk_71A208` raised to its size,
//! `unk_71A20C` = 1, `unk_71A1CC` = 0x32. All in `pusha` / `popa`.
//!
//! In:  esi the point, ebp the object
//! Out: CF set when inside; every register kept
//!
//! Callers: `physics::object_hits_car`, `physics::scene_point_hits_car` (both `jb`).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): points about the box by lot, every contact kind, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::turn_scratch_xz::scratch;
use crate::math::x87::loaded;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{a, base, g, p, x87_run};
use crate::physics::turn_by_heading::turn_by_heading;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{rounded, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::physics::contacts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::point_hits_car", rva(0x45E800), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `dword_6D3624`: the kind of contact being looked at.
pub const KIND: usize = 0x6D3624;

/// The Rust version: whether the point at `point` of the object at `object` is inside.
// Converted: `rc2k_core::physics::contacts::{point_part, inside_box, kind_hits}` (the x87
// steps stay).
pub unsafe fn point_hits_car(point: u32, object: u32) -> bool {
    let rd = |x: u32| unsafe { (x as *const u32).read_unaligned() };
    p(0x614110, g(0x614110).wrapping_add(1));
    let mut y = 0;
    for k in 0..3u32 {
        let v = contacts::point_part(
            rd(point + 4 * k),
            rd(object + 0x30 + 4 * k),
            g(0x6D3B40 + 4 * k as usize),
            g(0x71A10C + 4 * k as usize),
        );
        p(0x6D3B4C + 4 * k as usize, v);
        if k == 1 {
            y = v;
        }
    }
    x87_run!(
        "fild dword ptr [{b} + 0x2D3B4C]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2D3B78]",
        "fild dword ptr [{b} + 0x2D3B50]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2D3B7C]",
        "fild dword ptr [{b} + 0x2D3B54]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2D3B80]";
        b = base()
    );
    for k in 0..3 {
        p(0x6D3B58 + 4 * k, g(0x6D3B78 + 4 * k));
    }
    unsafe { turn_by_heading(a(0x6D3B58), y) };
    x87_run!(
        "fld dword ptr [{b} + 0x2D3B58]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fistp dword ptr [{b} + 0x2D3090]",
        "fld dword ptr [{b} + 0x2D3B5C]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fistp dword ptr [{b} + 0x2D3094]",
        "fld dword ptr [{b} + 0x2D3B60]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fistp dword ptr [{b} + 0x2D3098]";
        b = base()
    );
    let (x, z) = (g(0x6D3090), g(0x6D3098));
    if !contacts::inside_box(x, z, g(0x614BC8), g(0x614BC0), g(0x614BC4)) {
        return false;
    }
    if contacts::kind_hits(g(KIND)) {
        x87_run!(
            "fild dword ptr [{b} + 0x2D3090]",
            "fdiv dword ptr [{b} + 0x1C5D98]",
            "fstp dword ptr [{b} + 0x31A1FC]",
            ".byte 0xD9, 0xEE", // fldz
            "fstp dword ptr [{b} + 0x31A200]",
            "fild dword ptr [{b} + 0x2D3098]",
            "fdiv dword ptr [{b} + 0x1C5D98]",
            "fstp dword ptr [{b} + 0x31A204]";
            b = base()
        );
        p(0x6D3108, rounded(unsafe { vector_length(a(0x71A170)) }));
        if let Some(hit) = rc2k_core::physics::walls::raised_hit(
            f32::from_bits(g(0x6D3108)),
            f32::from_bits(g(0x71A208)),
        ) {
            p(0x71A208, hit.to_bits());
        }
        p(0x71A20C, 1);
        p(0x71A1CC, 0x32);
    }
    true
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { point_hits_car(r.esi, r.ebp) }))
}

/// What it writes (and `turn_by_heading`'s scratch point).
pub fn regions() -> Vec<Region> {
    vec![
        (a(0x614110), 4),
        (a(0x6D3B4C), 0x38),
        (a(0x6D3090), 12),
        (a(0x71A1FC), 0x14),
        (a(0x6D3108), 4),
        (a(0x71A1CC), 4),
        scratch(),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The car's figures it reads (set by lot in the tests): its place, heading, speed and
/// box, the offset.
pub fn inputs() -> Vec<Region> {
    vec![
        (a(0x6D3B40), 12),
        (a(0x71A10C), 12),
        (a(0x71A998), 4),
        (a(0x71A170), 12),
        (a(0x614BC0), 12),
        (a(0x71A208), 4),
        (a(KIND), 4),
    ]
}

/// Sets `inputs()` from `seed`: places within some 0x8000 of each other, a box of some
/// 0x10000 .. 0x30000, the contact kind `kind`.
pub fn set_inputs(seed: u32, kind: u32) {
    // The calls' counter (`dword_614110`, one up a call): the same start for both sides.
    p(0x614110, seed);
    let mut f = [0u32; 8];
    test_floats(seed.wrapping_mul(13).wrapping_add(5), &mut f);
    let small = |k: usize, m: i32| (f[k] as i32).wrapping_rem(m) as u32;
    for k in 0..3 {
        p(0x6D3B40 + 4 * k, small(k, 0x8000));
        p(0x71A10C + 4 * k, small(k + 3, 0x8000));
    }
    p(0x71A998, f[6] & 0xFFFF);
    for k in 0..3 {
        p(
            0x71A170 + 4 * k,
            loaded((f32::from_bits(f[k + 2]) % 4.0).to_bits()),
        );
        p(0x614BC0 + 4 * k, 0x10000 + (f[k + 4] & 0x1FFFF));
    }
    p(0x71A208, (f32::from_bits(f[7]) % 3.0).to_bits());
    p(KIND, kind);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend(inputs());
    let saved = capture(&all);
    let mut point = [0u32; 3];
    let mut object = [0u32; 0x10];
    let (pt, ob) = (point.as_mut_ptr() as u32, object.as_mut_ptr() as u32);
    all.push((pt, 12));
    all.push((ob, 0x40));
    let kinds = [10u32, 1, 5, 6, 0, 2, 9, 0xFF];
    let (mut inside, mut outside) = (0, 0);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..24u32 {
                let set = || {
                    set_inputs(seed, kinds[(seed % 8) as usize]);
                    let mut f = [0u32; 6];
                    test_floats(seed * 7 + 1, &mut f);
                    for k in 0..3 {
                        let v = (f[k] as i32).wrapping_rem(0x18000) as u32;
                        unsafe { ((pt + 4 * k as u32) as *mut u32).write(v) };
                        let o = (f[k + 3] as i32).wrapping_rem(0x8000) as u32;
                        unsafe { ((ob + 0x30 + 4 * k as u32) as *mut u32).write(o) };
                    }
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs {
                        esi: pt,
                        ebp: ob,
                        ..Regs::SENTINEL
                    },
                    &set,
                    &all,
                    &rust,
                );
                set();
                if unsafe { point_hits_car(pt, ob) } {
                    inside += 1;
                } else {
                    outside += 1;
                }
            }
        });
    }
    t.check(inside > 0 && outside > 0, || {
        format!("both ways not taken: {inside} inside, {outside} outside")
    });
    all.truncate(all.len() - 2);
    restore(&all, &saved);
    t.finish()
}
