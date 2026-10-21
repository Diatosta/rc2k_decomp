//! Small float steps of `make_masks`' tree (IDA, 2026-10-17): the 2 x 2 turn a line gives,
//! three rows scaled to set lengths, and two cosine-eased steps.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::math::x87::x87;
use crate::service::harness::{Region, at};

/// The 2 x 2 turn: `flt_82E396`, `flt_82E39A`, `flt_82E3A2`, `flt_82E3A6`.
pub const TURN_RVA: [usize; 4] = [rva(0x82E396), rva(0x82E39A), rva(0x82E3A2), rva(0x82E3A6)];
/// The three rows' lengths for `scale_rows3`.
const LENGTHS_RVA: usize = rva(0x89CA24); // flt_89CA24 .. flt_89CA2C
const EASE_RVA: usize = rva(0x89C9C0); // flt_89C9C0
const STEPS_RVA: [usize; 2] = [rva(0x89C9C4), rva(0x89C9C8)]; // dword_89C9C4, dword_89C9C8
const SCRATCH_RVA: [usize; 2] = [rva(0x764F40), rva(0x71C880)]; // dword_764F40, dword_71C880

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// `marks::set_line_turn` (0x4BF432) and, `back`, `marks::set_line_turn_back` (0x4BF466):
/// from the line at esi (+0, +4 to +0xC, +0x10) dx into +0x396 and +0x3A6, minus dy (dy
/// when `back`) into +0x3A2, its negation into +0x39A. Every register kept.
pub unsafe fn set_line_turn(line: u32, back: bool) {
    unsafe {
        let dx = x87::sub(get(line + 0xC), get(line));
        let dy = if back {
            x87::sub(get(line + 0x10), get(line + 4))
        } else {
            x87::negated_sub(get(line + 0x10), get(line + 4))
        };
        game_write(TURN_RVA[0], dx);
        game_write(TURN_RVA[2], dy);
        game_write(TURN_RVA[1], x87::negated(dy));
        game_write(TURN_RVA[3], x87::loaded(dx));
    }
}

pub fn turn_regions() -> Vec<Region> {
    TURN_RVA.iter().map(|&r| at(r)).collect()
}

/// `marks::scale_rows3` (0x4BF498): the three rows of three floats at esi each made the
/// length `flt_89CA24` .. `flt_89CA2C` says (`x87::scale_row`). Every register kept.
pub unsafe fn scale_rows3(rows: u32) {
    unsafe {
        for n in 0..3 {
            x87::scale_row(
                (rows + 12 * n) as *mut u32,
                game_read(LENGTHS_RVA + 4 * n as usize),
            );
        }
    }
}

/// `marks::smooth_steps` (0x4BE928): from a (ebx) to b (ecx) as floats, step eax: t (b -
/// a) times `flt_89C9C0` (a float), then ecx a + t - t cos(pi (eax + 1) / `dword_89C9C4`)
/// and ebx the same with eax over `dword_89C9C8`. `dword_764F40` and `dword_71C880` left
/// holding a. eax kept.
pub unsafe fn smooth_steps(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let a = g.ebx;
        (g.ecx, g.ebx) = smooth_pair(g.eax, g.ebx, g.ecx);
        for r in SCRATCH_RVA {
            game_write(r, a);
        }
    }
    g
}

/// `smooth_steps`' two results (ecx, ebx) from step n, a and b, writing nothing.
pub fn smooth_pair(n: u32, a: u32, b: u32) -> (u32, u32) {
    unsafe {
        let t = x87::sub_mul(b, a, game_read(EASE_RVA));
        let steps: [i32; 2] = STEPS_RVA.map(|r| game_read(r));
        (
            x87::cosine_step(steps[0], n.wrapping_add(1) as i32, t, a),
            x87::cosine_step(steps[1], n as i32, t, a),
        )
    }
}

pub fn step_regions() -> Vec<Region> {
    SCRATCH_RVA.iter().map(|&r| at(r)).collect()
}

/// `marks::turn_pair` (0x4BF409): st(0) a, st(1) b turned by `g_marksLineTurn` (m0 .. m3):
/// st(0) a m3 + b m2, st(1) b m0 + a m1, unrounded (the original's `fld st(1)`, `fxch`s
/// and `faddp`s). Every register kept.
pub unsafe fn turn_pair(inputs: [[u8; 10]; 2]) -> [[u8; 10]; 2] {
    let mut out = [[0u8; 10]; 2];
    let m = game_addr(TURN_RVA[0]) as u32;
    x87!(
        "fld tbyte ptr [{i} + 10]", "fld tbyte ptr [{i}]",
        ".byte 0xD9, 0xC1", ".byte 0xD9, 0xC1",
        "fmul dword ptr [{m} + 0x10]", ".byte 0xD9, 0xC9", "fmul dword ptr [{m} + 0xC]",
        ".byte 0xDE, 0xC1", ".byte 0xD9, 0xCA",
        "fmul dword ptr [{m}]", ".byte 0xD9, 0xC9", "fmul dword ptr [{m} + 4]",
        ".byte 0xDE, 0xC1", ".byte 0xD9, 0xC9",
        "fstp tbyte ptr [{o}]", "fstp tbyte ptr [{o} + 10]";
        i = inputs.as_ptr(), m = m, o = out.as_mut_ptr()
    );
    out
}

/// `clamp_pair`'s four scales: `flt_89C9FC`, `flt_89CA00` (in), `flt_89CA04`, `flt_89CA08`
/// (out).
const PAIR_SCALES_RVA: usize = rva(0x89C9FC);

/// `fld tbyte; fmul; fistp` for st(0) then st(1): their whole-number scalings.
fn scaled_ints(inputs: &[[u8; 10]; 2], k: *const u32) -> (i32, i32) {
    let mut v = [0i32; 2];
    x87!(
        "fld tbyte ptr [{i} + 10]", "fld tbyte ptr [{i}]",
        "fmul dword ptr [{k}]", "fistp dword ptr [{v}]",
        "fmul dword ptr [{k} + 4]", "fistp dword ptr [{v} + 4]";
        i = inputs.as_ptr(), k = k, v = v.as_mut_ptr()
    );
    (v[0], v[1])
}

/// `fild v; fdiv k; fstp tbyte`.
fn int_over_80(v: i32, k: *const u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!(
        "fild dword ptr [{v}]", "fdiv dword ptr [{k}]", "fstp tbyte ptr [{o}]";
        v = &v, k = k, o = out.as_mut_ptr()
    );
    out
}

/// `marks::clamp_pair` (0x4BEF22): st(0) times `flt_89C9FC` rounded into ebx, st(1) times
/// `flt_89CA00` into eax; eax held to 0 .. 0xFFFF and ebx to 0 .. 0x7FFF; then st(1) eax
/// over `flt_89CA04` and st(0) ebx over `flt_89CA08`, unrounded.
pub unsafe fn clamp_pair(entry: &Regs, inputs: [[u8; 10]; 2]) -> (Regs, [[u8; 10]; 2]) {
    let mut g = *entry;
    let k = game_addr(PAIR_SCALES_RVA) as *const u32;
    let (b, a) = scaled_ints(&inputs, k);
    g.ebx = b.clamp(0, 0x7FFF) as u32;
    g.eax = a.clamp(0, 0xFFFF) as u32;
    let outputs = unsafe {
        [
            int_over_80(g.ebx as i32, k.add(3)),
            int_over_80(g.eax as i32, k.add(2)),
        ]
    };
    (g, outputs)
}

// ---- the converted versions (`rc2k_core::marks::turns`) ----

/// The line turn's four floats.
fn line_turn_now() -> rc2k_core::marks::turns::LineTurn {
    TURN_RVA.map(|r| f32::from_bits(unsafe { game_read(r) }))
}

/// The converted `set_line_turn` / `set_line_turn_back` (plain floats): the hooks'.
// Converted: `rc2k_core::marks::turns::line_turn`
pub unsafe fn set_line_turn_plain(line: u32, back: bool) {
    use rc2k_core::marks::turns::{LINE_TURN_ORDER, line_turn};
    let f = |o: u32| f32::from_bits(unsafe { get(line + o) });
    let m = line_turn([f(0), f(4)], [f(0xC), f(0x10)], back);
    for n in LINE_TURN_ORDER {
        unsafe { game_write(TURN_RVA[n], m[n].to_bits()) };
    }
}

/// The converted `scale_rows3` (plain floats): the hook's.
// Converted: `rc2k_core::marks::turns::scale_rows`
pub unsafe fn scale_rows3_plain(rows: u32) {
    use crate::scene::vector3::{read_floats, write_floats};
    unsafe {
        let r: [[f32; 3]; 3] = std::array::from_fn(|n| read_floats(rows + 12 * n as u32));
        let lengths: [u32; 3] = game_read(LENGTHS_RVA);
        let made = rc2k_core::marks::turns::scale_rows(r, lengths.map(f32::from_bits));
        write_floats(rows, made.as_flattened());
    }
}

/// The converted `smooth_steps` (plain floats): the hook's.
// Converted: `rc2k_core::marks::turns::smooth_pair`
pub unsafe fn smooth_steps_plain(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let a = g.ebx;
        let ease: u32 = game_read(EASE_RVA);
        let steps: [i32; 2] = STEPS_RVA.map(|r| game_read(r));
        let (c, b) = rc2k_core::marks::turns::smooth_pair(
            g.eax,
            f32::from_bits(g.ebx),
            f32::from_bits(g.ecx),
            f32::from_bits(ease),
            steps,
        );
        (g.ecx, g.ebx) = (c.to_bits(), b.to_bits());
        for r in SCRATCH_RVA {
            game_write(r, a);
        }
    }
    g
}

/// The converted `turn_pair` (plain floats): the hook's.
// Converted: `rc2k_core::marks::turns::turn_pair`
pub unsafe fn turn_pair_plain(inputs: [[u8; 10]; 2]) -> [[u8; 10]; 2] {
    use rc2k_core::math::extended::{from_f64, to_f64};
    let (a, b) =
        rc2k_core::marks::turns::turn_pair(line_turn_now(), to_f64(inputs[0]), to_f64(inputs[1]));
    [from_f64(a), from_f64(b)]
}

/// Two 80-bit values from floats, for the tests.
pub fn test_pair(seed: u32) -> [[u8; 10]; 2] {
    let mut v = [0u32; 2];
    crate::scene::matrix3::test_floats(seed, &mut v);
    v.map(crate::scene::matrix3::extended)
}
