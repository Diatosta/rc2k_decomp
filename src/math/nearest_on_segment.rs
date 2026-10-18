//! `math::nearest_on_segment` (0x4633DE): the point of a segment nearest a point, seen
//! from above.
//!
//! In:  the segment's ends A (`dword_71C384` .. `dword_71C38C`) and B (`dword_71C390` ..
//!      `dword_71C398`), the point O (`dword_71C39C` .. `dword_71C3A4`): floats
//! Out: `dword_71C3A8` .. `dword_71C3B0` the point; every register preserved
//! With x and z only (A - O, B - O and A - B into `flt_71C3B8` .. `flt_71C3D4`): t =
//! (A - O).(A - B) over |A - B|^2, 0 when that is equal to `flt_71C3C8` (0.0); held to
//! `flt_5C5D90` (0) .. `flt_5C5D38` (1) into `flt_71C3B4`; the point = A + (B - A) t in
//! all three. The x87 code is the original's; its two stack values at the compare are
//! kept as 80-bit numbers between the runs.
//! Callers: `sound::merge_spot_voices` (and unreached code at 0x463349).
//! Shadow mode: both, the variables compared.
//! Self-test: the original against the Rust version on points of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::math::x87_run::compare_80;
use crate::physics::flow::{base, hold_high, hold_low, p, x87_run};
use crate::service::harness::{Region, capture, restore};
use crate::sound::turn_parts::{Part, both, dispatch_turn, f};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::nearest_on_segment", rva(0x4633DE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// dword_71C384 .. flt_71C3D4: the ends, the point, the result and the work.
pub const VARS: usize = 0x71C384;
pub const VARS_LEN: usize = 0x54;
const T: usize = 0x71C3B4;

fn regions(_: &Regs) -> Vec<Region> {
    vec![(game_addr(rva(VARS)) as u32, VARS_LEN)]
}

// ---- Rust version ----

pub fn nearest_on_segment_in(_: &mut Regs, _: Part) -> Result<(), Refused> {
    let b = base();
    let (mut top, mut below) = ([0u8; 10], [0u8; 10]);
    x87_run!(
        "fld dword ptr [{b} + 0x31C384]",
        "fsub dword ptr [{b} + 0x31C39C]",
        "fstp dword ptr [{b} + 0x31C3B8]",
        "fld dword ptr [{b} + 0x31C38C]",
        "fsub dword ptr [{b} + 0x31C3A4]",
        "fstp dword ptr [{b} + 0x31C3BC]",
        "fld dword ptr [{b} + 0x31C390]",
        "fsub dword ptr [{b} + 0x31C39C]",
        "fstp dword ptr [{b} + 0x31C3C0]",
        "fld dword ptr [{b} + 0x31C398]",
        "fsub dword ptr [{b} + 0x31C3A4]",
        "fstp dword ptr [{b} + 0x31C3C4]",
        "fld dword ptr [{b} + 0x31C3B8]",
        "fsub dword ptr [{b} + 0x31C3C0]",
        "fstp dword ptr [{b} + 0x31C3D0]",
        "fld dword ptr [{b} + 0x31C3BC]",
        "fsub dword ptr [{b} + 0x31C3C4]",
        "fstp dword ptr [{b} + 0x31C3D4]",
        "fld dword ptr [{b} + 0x31C3B8]",
        "fmul dword ptr [{b} + 0x31C3D0]",
        "fld dword ptr [{b} + 0x31C3BC]",
        "fmul dword ptr [{b} + 0x31C3D4]",
        ".byte 0xDE, 0xC1",
        "fld dword ptr [{b} + 0x31C3C0]",
        "fmul dword ptr [{b} + 0x31C3D0]",
        "fld dword ptr [{b} + 0x31C3C4]",
        "fmul dword ptr [{b} + 0x31C3D4]",
        ".byte 0xDE, 0xC1",
        ".byte 0xD8, 0xE9",
        "fstp tbyte ptr [{t}]",
        "fstp tbyte ptr [{u}]";
        b = b, t = top.as_mut_ptr(), u = below.as_mut_ptr()
    );
    // fcom flt_71C3C8; C3: equal (or unordered).
    if compare_80(top, crate::physics::flow::g(0x71C3C8)) & 0x4000 != 0 {
        p(T, 0);
    } else {
        x87_run!(
            "fld tbyte ptr [{u}]",
            "fld tbyte ptr [{t}]",
            ".byte 0xDE, 0xF9",
            "fstp dword ptr [{b} + 0x31C3B4]";
            b = b, t = top.as_ptr(), u = below.as_ptr()
        );
        hold_low(T, 0x5C5D90);
        hold_high(T, 0x5C5D38);
    }
    x87_run!(
        "fld dword ptr [{b} + 0x31C390]",
        "fsub dword ptr [{b} + 0x31C384]",
        "fmul dword ptr [{b} + 0x31C3B4]",
        "fadd dword ptr [{b} + 0x31C384]",
        "fstp dword ptr [{b} + 0x31C3A8]",
        "fld dword ptr [{b} + 0x31C394]",
        "fsub dword ptr [{b} + 0x31C388]",
        "fmul dword ptr [{b} + 0x31C3B4]",
        "fadd dword ptr [{b} + 0x31C388]",
        "fstp dword ptr [{b} + 0x31C3AC]",
        "fld dword ptr [{b} + 0x31C398]",
        "fsub dword ptr [{b} + 0x31C38C]",
        "fmul dword ptr [{b} + 0x31C3B4]",
        "fadd dword ptr [{b} + 0x31C38C]",
        "fstp dword ptr [{b} + 0x31C3B0]";
        b = b
    );
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(
        &HOOK,
        r,
        call_original,
        nearest_on_segment_in,
        Some(&regions),
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = regions(&Regs::SENTINEL);
    let saved = capture(&all);
    let points: [[f32; 9]; 6] = [
        [0.0, 1.0, 0.0, 10.0, 2.0, 0.0, 5.0, 0.0, 3.0],
        [0.0, 0.0, 0.0, 10.0, 0.0, 10.0, -5.0, 0.0, -5.0],
        [0.0, 0.0, 0.0, 10.0, 0.0, 10.0, 20.0, 0.0, 20.0],
        [3.0, 0.0, 4.0, 3.0, 9.0, 4.0, 0.0, 0.0, 0.0],
        [1.5, -2.0, 7.25, -8.0, 0.5, 3.0, 0.1, 0.2, 0.3],
        [1e20, 0.0, -1e20, -1e20, 0.0, 1e20, 0.0, 0.0, 0.0],
    ];
    for (k, pts) in points.iter().enumerate() {
        let fill = || unsafe {
            for n in 0..VARS_LEN / 4 {
                game_write(rva(VARS + 4 * n), 0x5555_5555u32);
            }
            for (n, &v) in pts.iter().enumerate() {
                game_write(rva(VARS + 4 * n), f(v));
            }
            game_write(rva(0x71C3C8), 0u32);
        };
        both(
            &mut t,
            &format!("points {k}"),
            &|r| unsafe { call_original(r) },
            Regs::SENTINEL,
            &fill,
            &all,
            nearest_on_segment_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
