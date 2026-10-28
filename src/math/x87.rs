//! Arithmetic the original did with the x87, where a result has to be the original's to
//! the last bit: plain Rust on `X` (`x87_float.rs`), an `f64` that rounds as the control
//! word says, in the original's order of operations; `rcpss` is an approximation that
//! differs between processors and stays an instruction (an SSE intrinsic). No x87
//! instruction runs outside the tests (2026-10-25).
//! All values are floats given and returned as their bits.

/// The original's instructions, for the reference versions the tests compare with.
#[cfg(all(test, target_arch = "x86"))]
macro_rules! x87 {
    ($($line:literal),+; $($name:ident = $value:expr),+) => {
        unsafe {
            core::arch::asm!(
                $($line),+,
                $($name = in(reg) $value),+,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            )
        }
    };
}
#[cfg(all(test, target_arch = "x86"))]
pub(crate) use x87;

use super::x87_float as bh;

/// A float read from game memory (possibly unaligned), as the x87 holds it once loaded.
/// `bytes` bytes at `p` checked to lie in one region (the program ends otherwise): what a
/// helper below reads and writes, checked before it starts.
fn span<T>(p: *const T, bytes: usize) {
    crate::pointers::check_or_end(
        crate::pointers::game_address(p),
        bytes,
        "a math helper's operand",
    );
}

unsafe fn load(p: *const u32, index: usize) -> bh::X {
    bh::ld(unsafe { p.add(index).read_unaligned() })
}

/// `a + b`.
pub fn add(a: u32, b: u32) -> u32 {
    bh::st(bh::ld(a) + bh::ld(b))
}

/// `a - b`.
pub fn sub(a: u32, b: u32) -> u32 {
    bh::st(bh::ld(a) - bh::ld(b))
}

/// A whole number as a float.
pub fn from_int(a: i32) -> u32 {
    bh::st(bh::int(a))
}

/// `a * b`.
pub fn mul(a: u32, b: u32) -> u32 {
    bh::st(bh::ld(a) * bh::ld(b))
}

/// `1 / a * b`.
pub fn reciprocal_times(a: u32, b: u32) -> u32 {
    bh::st(bh::ONE / bh::ld(a) * bh::ld(b))
}

/// `x` mirrored about a + b: `-(x - a - b) + b + a`.
pub fn mirrored(x: u32, a: u32, b: u32) -> u32 {
    bh::st(-(bh::ld(x) - bh::ld(a) - bh::ld(b)) + bh::ld(b) + bh::ld(a))
}

/// A float as it comes back from a register.
pub fn loaded(a: u32) -> u32 {
    bh::st(bh::ld(a))
}

/// The whole number `a` rounds to, as the control word says.
pub fn to_int(a: u32) -> i32 {
    bh::to_int(bh::ld(a))
}

/// The product as a whole number.
pub fn mul_to_int(a: u32, b: u32) -> i32 {
    bh::to_int(bh::ld(a) * bh::ld(b))
}

/// `a / b`, two whole numbers, as a float.
pub fn int_ratio(a: i32, b: i32) -> u32 {
    bh::st(bh::int(a) / bh::int(b))
}

/// `1 - a / b`.
pub fn one_less_int_ratio(a: i32, b: i32) -> u32 {
    bh::st(bh::ONE - bh::int(a) / bh::int(b))
}

/// `rcpss xmm0, a; mulss xmm0, b` (an SSE approximation, kept as the instruction: its
/// result differs between processors).
pub fn rcpss_times(a: u32, b: u32) -> u32 {
    use crate::arch::{_mm_cvtss_f32, _mm_mul_ss, _mm_rcp_ss, _mm_set_ss};
    let (a, b) = (f32::from_bits(a), f32::from_bits(b));
    // SSE is part of the i686 target's baseline.
    unsafe { _mm_cvtss_f32(_mm_mul_ss(_mm_rcp_ss(_mm_set_ss(a)), _mm_set_ss(b))).to_bits() }
}

/// A row of one matrix times a column of another: `a[0] b[0] + (a[1] b[4] + a[2] b[8])`,
/// the last two products added first, stored to `out`.
#[cfg(all(test, target_arch = "x86"))] // checked against the assembly; the helpers work on cells now
pub fn row_times_column(a: *const u32, b: *const u32, out: *mut u32) {
    span(a, 12);
    span(b, 36);
    span(out, 4);

    let p1 = unsafe { load(a, 0) * load(b, 0) };
    let p2 = unsafe { load(a, 1) * load(b, 4) };
    let p3 = unsafe { load(a, 2) * load(b, 8) };
    unsafe { out.write_unaligned(bh::st(p1 + (p2 + p3))) };
}

/// `out = a + out`.
#[cfg(all(test, target_arch = "x86"))] // checked against the assembly; the helpers work on cells now
pub fn add_into(a: *const u32, out: *mut u32) {
    span(a, 4);
    span(out, 4);

    let sum = unsafe { load(a, 0) + load(out, 0) };
    unsafe { out.write_unaligned(bh::st(sum)) };
}

/// A whole number over sixteen (one doubled four times).
pub fn int_over_16(a: i32) -> u32 {
    bh::st(bh::int(a) / bh::X(16.0))
}

/// The length of two whole numbers at right angles, as a whole number:
/// `sqrt(a * a + b * b)`.
pub fn int_hypot(a: i32, b: i32) -> i32 {
    let (a, b) = (bh::int(a), bh::int(b));
    bh::to_int((a * a + b * b).sqrt())
}

/// `a / b`.
pub fn div(a: u32, b: u32) -> u32 {
    bh::st(bh::ld(a) / bh::ld(b))
}

/// `a * b - c * d`.
pub fn product_difference(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(bh::ld(a) * bh::ld(b) - bh::ld(c) * bh::ld(d))
}

/// The same negated.
pub fn negated_product_difference(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(-(bh::ld(a) * bh::ld(b) - bh::ld(c) * bh::ld(d)))
}

/// The length of three floats: `sqrt(v0 v0 + (v1 v1 + v2 v2))`.
#[cfg(all(test, target_arch = "x86"))] // checked against the assembly; the helpers work on cells now
pub fn length(v: *const u32) -> u32 {
    span(v, 12);

    let (x, y, z) = unsafe { (load(v, 0), load(v, 1), load(v, 2)) };
    bh::st((x * x + (y * y + z * z)).sqrt())
}

/// Whether the two compare as equal, which they also do when one is no number.
pub fn equal_or_unordered(a: u32, b: u32) -> bool {
    let (a, b) = (bh::ld(a), bh::ld(b));
    matches!(a.partial_cmp(&b), None | Some(std::cmp::Ordering::Equal))
}

/// The sine and the cosine of a whole number divided three times: `angle / a / a / b`.
pub fn sine_cosine(angle: i32, a: u32, b: u32) -> (u32, u32) {
    let x = bh::int(angle) / bh::ld(a) / bh::ld(a) / bh::ld(b);
    let (sin, cos) = x.sin_cos();
    (bh::st(sin), bh::st(cos))
}

/// The root of a whole number over a float.
pub fn int_over_root(a: i32, d: u32) -> u32 {
    bh::st((bh::int(a) / bh::ld(d)).sqrt())
}

/// `sqrt(a)`.
pub fn sqrt(a: u32) -> u32 {
    bh::st(bh::ld(a).sqrt())
}

/// The sine of n times pi over d: `sin(n * pi / d)`.
pub fn sine_of_pi_fraction(n: i32, d: i32) -> u32 {
    bh::st((bh::int(n).times_pi() / bh::int(d)).sin())
}

/// `a / b * m * n`, as a whole number.
pub fn ratio_scaled_to_int(a: u32, b: u32, m: i32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) / bh::ld(b) * bh::int(m) * bh::int(n))
}

/// The angle of (x, y) over pi, times two floats, as a whole number:
/// `atan2(y, x) / pi * a * b`.
pub fn angle_scaled_to_int(y: i32, x: i32, a: u32, b: u32) -> i32 {
    let angle = bh::int(y).atan2_over_pi(bh::int(x));
    bh::to_int(angle * bh::ld(a) * bh::ld(b))
}

/// A whole number times a float.
pub fn int_times(a: i32, b: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b))
}

/// A whole number over a float.
pub fn int_over(a: i32, b: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b))
}

/// A whole number less a float.
pub fn int_less(a: i32, b: u32) -> u32 {
    bh::st(bh::int(a) - bh::ld(b))
}

/// `1 / a`.
pub fn reciprocal(a: u32) -> u32 {
    bh::st(bh::ONE / bh::ld(a))
}

/// Two whole numbers and two floats multiplied: `a * b * c * d`.
pub fn int_float_int_float(a: i32, b: u32, c: i32, d: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b) * bh::int(c) * bh::ld(d))
}

/// `a * b / c / d` as a whole number.
pub fn mul_div_div_to_int(a: u32, b: u32, c: u32, d: u32) -> i32 {
    bh::to_int(bh::ld(a) * bh::ld(b) / bh::ld(c) / bh::ld(d))
}

/// `sub_41FC4D`'s sequence: x, y, z squared and summed (y and z first), the root taken and
/// compared with `zero`; unless equal (or unordered), each of the three is divided by the
/// root (not rounded to a float first) and stored back; then the root is stored. Returns
/// the root's bits and whether it compared equal.
pub fn normalise_in_place(v: *mut u32, zero: u32) -> (u32, bool) {
    span(v, 12);

    let (x, y, z) = unsafe { (load(v, 0), load(v, 1), load(v, 2)) };
    let root = (x * x + (y * y + z * z)).sqrt();
    let zero = bh::ld(zero);
    let equal = matches!(
        root.partial_cmp(&zero),
        None | Some(std::cmp::Ordering::Equal)
    );
    if !equal {
        unsafe {
            v.write_unaligned(bh::st(x / root));
            v.add(1).write_unaligned(bh::st(y / root));
            v.add(2).write_unaligned(bh::st(z / root));
        }
    }
    (bh::st(root), equal)
}

/// `a * b - c * d`, whole numbers, rounded once to a float.
pub fn int_product_difference(a: i32, b: i32, c: i32, d: i32) -> u32 {
    bh::st(bh::int(a) * bh::int(b) - bh::int(c) * bh::int(d))
}

/// A whole number divided by a float, rounded once.
pub fn int_div(a: i32, b: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b))
}

/// A whole number over two floats, doubled, rounded once: `a / b / c` added to itself.
pub fn int_div_div_doubled(a: i32, b: u32, c: u32) -> u32 {
    let q = bh::int(a) / bh::ld(b) / bh::ld(c);
    bh::st(q + q)
}

/// `math::turn_scratch_xz`'s turn: x' = x cos + z sin, z' = z cos - x sin, each rounded
/// once, written back over x and z.
pub fn turn_in_place(sine: *const u32, cosine: *const u32, x: *mut u32, z: *mut u32) {
    span(sine, 4);
    span(cosine, 4);
    span(x, 4);
    span(z, 4);

    let (s, c) = unsafe { (load(sine, 0), load(cosine, 0)) };
    let (xv, zv) = unsafe { (load(x, 0), load(z, 0)) };
    let new_x = xv * c + zv * s;
    let new_z = c * zv - s * xv;
    unsafe {
        z.write_unaligned(bh::st(new_z));
        x.write_unaligned(bh::st(new_x));
    }
}

/// `race::add_surface_share`'s sum: `a / b * c / d * e + f`.
pub fn surface_share(a: i32, b: u32, c: u32, d: i32, e: u32, f: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b) * bh::ld(c) / bh::int(d) * bh::ld(e) + bh::ld(f))
}

/// `atan2(y, x) / pi * a * b * c`, as a whole number.
pub fn angle_scaled3_to_int(y: i32, x: i32, a: u32, b: u32, c: u32) -> i32 {
    let angle = bh::int(y).atan2_over_pi(bh::int(x));
    bh::to_int(angle * bh::ld(a) * bh::ld(b) * bh::ld(c))
}

/// `atan2(y, x) / pi * a * a * a * b * b`, as a whole number.
pub fn angle_scaled5_to_int(y: i32, x: i32, a: u32, b: u32) -> i32 {
    let angle = bh::int(y).atan2_over_pi(bh::int(x));
    bh::to_int(angle * bh::ld(a) * bh::ld(a) * bh::ld(a) * bh::ld(b) * bh::ld(b))
}

/// The length of (a, b) over that of (c, d), whole numbers, as a float.
pub fn hypot_ratio(a: i32, b: i32, c: i32, d: i32) -> u32 {
    let (a, b, c, d) = (bh::int(a), bh::int(b), bh::int(c), bh::int(d));
    let first = (a * a + b * b).sqrt();
    let second = (c * c + d * d).sqrt();
    bh::st(first / second)
}

/// A whole number times a float, as a whole number.
pub fn int_times_to_int(a: i32, b: u32) -> i32 {
    bh::to_int(bh::int(a) * bh::ld(b))
}

/// A whole number over a float, less a float, rounded once.
pub fn int_div_less(a: i32, b: u32, c: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b) - bh::ld(c))
}

/// `count` sines of an angle that starts at 0 and grows by `step` each time (the angle kept
/// as a double, never rounded to a float), stored one dword each from `out`.
pub fn sine_table(step: u32, out: *mut u32, count: u32) {
    span(out, 4 * count as usize);

    let step = bh::ld(step);
    let mut angle = bh::X(0.0);
    for i in 0..count as usize {
        unsafe { out.add(i).write_unaligned(bh::st(angle.sin())) };
        angle += step;
    }
}

/// `1 / a` stored, and that (unrounded) times a float stored.
pub fn reciprocal_int_and_times(a: i32, b: u32) -> (u32, u32) {
    let r = bh::ONE / bh::int(a);
    (bh::st(r), bh::st(r * bh::ld(b)))
}

/// `a * b` stored, and that (unrounded) over `c` stored.
pub fn mul_and_div(a: u32, b: u32, c: u32) -> (u32, u32) {
    let p = bh::ld(a) * bh::ld(b);
    (bh::st(p), bh::st(p / bh::ld(c)))
}

/// `(a - b) / c * d`.
pub fn sub_div_mul(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st((bh::ld(a) - bh::ld(b)) / bh::ld(c) * bh::ld(d))
}

/// `(a - b) / (c - d) * e`.
pub fn diff_ratio_mul(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    bh::st((bh::ld(a) - bh::ld(b)) / (bh::ld(c) - bh::ld(d)) * bh::ld(e))
}

/// `((a - b) m + b) n`, less `less` (none: nothing taken off), rounded once.
pub fn lerp_scaled_less(a: u32, b: u32, m: u32, n: u32, less: Option<u32>) -> u32 {
    let scaled = ((bh::ld(a) - bh::ld(b)) * bh::ld(m) + bh::ld(b)) * bh::ld(n);
    match less {
        Some(less) => bh::st(scaled - bh::ld(less)),
        None => bh::st(scaled),
    }
}

/// `-(a - b)`.
pub fn negated_sub(a: u32, b: u32) -> u32 {
    bh::st(-(bh::ld(a) - bh::ld(b)))
}

/// `a x + (b y + c z)`, whole numbers in, a whole number out.
pub fn weighted3_to_int(a: i32, b: i32, c: i32, x: u32, y: u32, z: u32) -> i32 {
    let first = bh::int(a) * bh::ld(x);
    let second = bh::int(b) * bh::ld(y);
    let third = bh::int(c) * bh::ld(z);
    bh::to_int(first + (second + third))
}

/// `a * b + c`.
pub fn int_times_plus(a: i32, b: u32, c: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b) + bh::ld(c))
}

/// The step `(last - first) / by`, then for each of `count` whole numbers at `values` the
/// number less the running sum, rounded to a whole number and stored back, and the sum
/// grown by the step (both kept as doubles).
pub fn ramp_down(values: *mut i32, count: u32, last: i32, first: i32, by: i32) {
    span(values, 4 * count as usize);

    let step = (bh::int(last) - bh::int(first)) / bh::int(by);
    let mut sum = bh::X(0.0);
    for i in 0..count as usize {
        let p = unsafe { values.add(i) };
        let value = unsafe { p.read_unaligned() };
        unsafe { p.write_unaligned(bh::to_int(bh::int(value) - sum)) };
        sum += step;
    }
}

/// The difference of two whole numbers, rounded once to a float.
pub fn int_less_int(a: i32, b: i32) -> u32 {
    bh::st(bh::int(a) - bh::int(b))
}

/// `(a - b) / c`, a and b whole numbers.
pub fn int_less_int_over(a: i32, b: i32, c: u32) -> u32 {
    bh::st((bh::int(a) - bh::int(b)) / bh::ld(c))
}

/// `(a - b) / c`.
pub fn sub_div(a: u32, b: u32, c: u32) -> u32 {
    bh::st((bh::ld(a) - bh::ld(b)) / bh::ld(c))
}

/// `sin(a) b + c`, as a whole number.
pub fn sine_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    bh::to_int(bh::ld(a).sin() * bh::ld(b) + bh::ld(c))
}

/// `c - a a b`, as a whole number: `-(a a) b + c`.
pub fn negated_square_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    let a = bh::ld(a);
    bh::to_int(-(a * a) * bh::ld(b) + bh::ld(c))
}

/// The sum plus the length of (x, y, z): `sqrt(x x + (y y + z z)) + sum`.
pub fn length_plus_int(x: i32, y: i32, z: i32, sum: i32) -> i32 {
    let (x, y, z) = (bh::int(x), bh::int(y), bh::int(z));
    bh::to_int((x * x + (y * y + z * z)).sqrt() + bh::int(sum))
}

/// `c - a / b * c`, a whole number: c less a / b of c.
pub fn share_left_to_int(a: i32, b: i32, c: u32) -> i32 {
    let c = bh::ld(c);
    bh::to_int(c - bh::int(a) / bh::int(b) * c)
}

// The control word as this thread's float code sees it (phase 6, 2026-10-24): the plain
// Rust helpers read its rounding (bits 10 .. 11) where the x87 would round to a whole
// number. The precision bits are not modelled: plain `f64` steps are the x87's at
// double precision (0x027F, a 32-bit Windows thread's start), within the last bit at
// extended (0x037F, after the game's `finit`). In the tests the real control word is kept
// equal to it, for the reference assembly (`set_control_word`, `finit`).
thread_local! {
    static CONTROL: std::cell::Cell<u16> = const { std::cell::Cell::new(0x027F) };
}

/// The rounding the control word asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    Nearest,
    Down,
    Up,
    Chop,
}

/// This thread's rounding (the emulated control word's bits 10 .. 11).
pub fn rounding() -> Rounding {
    match (CONTROL.with(|c| c.get()) >> 10) & 3 {
        0 => Rounding::Nearest,
        1 => Rounding::Down,
        2 => Rounding::Up,
        _ => Rounding::Chop,
    }
}

/// `fistp dword` of `x` at this thread's rounding: out of range or NaN gives the integer
/// indefinite, 0x80000000, as the x87 does with the invalid operation masked.
pub fn round_to_int(x: f64) -> i32 {
    let r = match rounding() {
        Rounding::Nearest => x.round_ties_even(),
        Rounding::Down => x.floor(),
        Rounding::Up => x.ceil(),
        Rounding::Chop => x.trunc(),
    };
    if r.is_nan() || !(-2_147_483_648.0..2_147_483_648.0).contains(&r) {
        i32::MIN
    } else {
        r as i32
    }
}

/// `finit` (9B DB E3): the x87 reset, its control word 0x37F (every exception masked,
/// 64-bit precision, rounding to nearest), as the original does it.
pub fn finit() {
    CONTROL.with(|c| c.set(0x037F));
    // The real x87 only matters to the tests' reference assembly.
    #[cfg(all(test, target_arch = "x86"))]
    unsafe {
        core::arch::asm!(
            ".byte 0x9B, 0xDB, 0xE3",
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The control word (`fnstcw`): this thread's emulated one.
pub fn control_word() -> u16 {
    CONTROL.with(|c| c.get())
}

/// `fldcw`: the control word set (the emulated one; in the tests the real one too).
pub fn set_control_word(w: u16) {
    CONTROL.with(|c| c.set(w));
    // The real control word only matters to the tests' reference assembly.
    #[cfg(all(test, target_arch = "x86"))]
    unsafe {
        core::arch::asm!("fldcw word ptr [{w}]", w = in(reg) &raw const w, options(nostack))
    };
}

/// `row_times_column` for 3 x 3 matrices: `b`'s column read 12 bytes apart.
/// `row_times_column` for 3 x 3 matrices: `b`'s column read 12 bytes apart.
pub fn row_times_column_3(a: *const u32, b: *const u32, out: *mut u32) {
    span(a, 12);
    span(b, 28);
    span(out, 4);

    let p1 = unsafe { load(a, 0) * load(b, 0) };
    let p2 = unsafe { load(a, 1) * load(b, 3) };
    let p3 = unsafe { load(a, 2) * load(b, 6) };
    unsafe { out.write_unaligned(bh::st(p1 + (p2 + p3))) };
}

/// A point of three whole numbers turned by a 3 x 3 matrix: x m0 + (y m1 + z m2) and
/// x m3 + (y m4 + z m5) rounded to whole numbers at `out` and `out + 4`, and
/// y m7 + (x m6 + z m8) at `out + 8` (the original's order). `out` may be `p`.
pub fn turn_int_point(p: *const i32, m: *const u32, out: *mut i32) {
    span(p, 12);
    span(m, 36);
    span(out, 12);

    unsafe {
        let (x, y, z) = (
            bh::int(p.read_unaligned()),
            bh::int(p.add(1).read_unaligned()),
            bh::int(p.add(2).read_unaligned()),
        );
        let first = x * load(m, 0) + (y * load(m, 1) + z * load(m, 2));
        out.write_unaligned(bh::to_int(first));
        let second = x * load(m, 3) + (y * load(m, 4) + z * load(m, 5));
        out.add(1).write_unaligned(bh::to_int(second));
        let third = y * load(m, 7) + (x * load(m, 6) + z * load(m, 8));
        out.add(2).write_unaligned(bh::to_int(third));
    }
}

/// `a * b * c`.
pub fn mul2(a: u32, b: u32, c: u32) -> u32 {
    bh::st(bh::ld(a) * bh::ld(b) * bh::ld(c))
}

/// 1 - sqrt(1 - ((i - a) b)^2), a quarter circle's curve, negated when `negate`.
pub fn circle_curve(i: i32, a: u32, b: u32, negate: bool) -> u32 {
    let t = (bh::int(i) - bh::ld(a)) * bh::ld(b);
    let curve = bh::ONE - (bh::ONE - t * t).sqrt();
    bh::st(if negate { -curve } else { curve })
}

/// `count` points round a circle of radius `r`, three dwords apart: sin(angle) r then
/// cos(angle) r, the dword between left alone. The angle starts at 0 and grows by
/// `pi / c * d` each time, kept as a double. `count` at least 1 (the original would run 2^32
/// times on 0; here nothing is done).
pub fn circle_points(r: *const u32, out: *mut u32, count: u32, c: *const i32, d: *const u32) {
    if count == 0 {
        return;
    }
    span(r, 4);
    span(c, 4);
    span(d, 4);
    span(out, 4 * (3 * (count as usize - 1) + 2));
    let mut angle = bh::X(0.0);
    for i in 0..count as usize {
        let (sin, cos) = angle.sin_cos();
        let cos_r = cos * unsafe { load(r, 0) };
        let sin_r = sin * unsafe { load(r, 0) };
        unsafe {
            out.add(i * 3 + 1).write_unaligned(bh::st(cos_r));
            out.add(i * 3).write_unaligned(bh::st(sin_r));
        }
        let divisor = bh::int(unsafe { c.read_unaligned() });
        angle += divisor.pi_over() * unsafe { load(d, 0) };
    }
}

/// `-a`.
pub fn negated(a: u32) -> u32 {
    bh::st(-bh::ld(a))
}

/// `(b - a) k`.
pub fn sub_mul(b: u32, a: u32, k: u32) -> u32 {
    bh::st((bh::ld(b) - bh::ld(a)) * bh::ld(k))
}

/// `a + t - t cos(pi m / c)`: `(-cos(pi / c * m)) t + a + t`.
pub fn cosine_step(c: i32, m: i32, t: u32, a: u32) -> u32 {
    let angle = bh::int(c).pi_over() * bh::int(m);
    bh::st(-angle.cos() * bh::ld(t) + bh::ld(a) + bh::ld(t))
}

/// Three floats at `row` each times `c` over their length: the root of
/// `x x + (y y + z z)`, `c / root`, then each times that, rounded once and stored in turn.
pub fn scale_row(row: *mut u32, c: u32) {
    span(row, 12);

    let (x, y, z) = unsafe { (load(row, 0), load(row, 1), load(row, 2)) };
    let k = bh::ld(c) / (x * x + (y * y + z * z)).sqrt();
    unsafe {
        row.write_unaligned(bh::st(k * x));
        row.add(1).write_unaligned(bh::st(k * y));
        row.add(2).write_unaligned(bh::st(k * z));
    }
}

/// `x + (k - (y - n)) s`, n the whole number y rounds to (as the control word says),
/// which is given back too.
pub fn frac_step(y: u32, k: u32, s: u32, x: u32) -> (i32, u32) {
    let y = bh::ld(y);
    let n = bh::to_int(y);
    (
        n,
        bh::st((bh::ld(k) - (y - bh::int(n))) * bh::ld(s) + bh::ld(x)),
    )
}

/// `a` rounded to a whole number, kept a float (as the control word says).
pub fn round_float(a: u32) -> u32 {
    bh::st(bh::ld(a).rint())
}

/// `a` compared with `b`: the status word's C0 .. C2 (0x700), so 0x100 when `a` is less,
/// 0x500 when either is no number, 0 otherwise (equal sets only C3, which is masked off).
pub fn compare_status(a: u32, b: u32) -> u16 {
    bh::compare(bh::ld(a), bh::ld(b)) & 0x700
}

/// `a - n`, as a whole number.
pub fn float_less_int_to_int(a: u32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) - bh::int(n))
}

/// x0 + ((1 / (c - a)) (b - a)) (x2 - x0), where the long edge is at b.
pub fn split_x(a: u32, b: u32, c: u32, x0: u32, x2: u32) -> u32 {
    let ratio = bh::ONE / (bh::ld(c) - bh::ld(a));
    let part = ratio * (bh::ld(b) - bh::ld(a));
    bh::st(part * (bh::ld(x2) - bh::ld(x0)) + bh::ld(x0))
}

/// The half (b - a) k as a float, and a plus it unrounded, then as a float.
pub fn half_and_middle(b: u32, a: u32, k: u32) -> (u32, u32) {
    let half = (bh::ld(b) - bh::ld(a)) * bh::ld(k);
    (bh::st(half), bh::st(half + bh::ld(a)))
}

/// `pi / n`.
pub fn pi_over(n: i32) -> u32 {
    bh::st(bh::int(n).pi_over())
}

/// m - h cos(a [+ s]): `-cos(a [+ s]) h + m`.
pub fn less_cosine(a: u32, s: Option<u32>, h: u32, m: u32) -> u32 {
    let angle = match s {
        Some(s) => bh::ld(a) + bh::ld(s),
        None => bh::ld(a),
    };
    bh::st(-angle.cos() * bh::ld(h) + bh::ld(m))
}

/// `sin(a)`.
pub fn sine(a: u32) -> u32 {
    bh::st(bh::ld(a).sin())
}

/// `cos(a)`, negated with `negate`.
pub fn cosine(a: u32, negate: bool) -> u32 {
    let cos = bh::ld(a).cos();
    bh::st(if negate { -cos } else { cos })
}

/// `a * b + c`.
pub fn mul_add(a: u32, b: u32, c: u32) -> u32 {
    bh::st(bh::ld(a) * bh::ld(b) + bh::ld(c))
}

/// The length of (a, b, c), the last two squares added first, rounded.
pub fn int_length3(a: i32, b: i32, c: i32) -> i32 {
    let (a, b, c) = (bh::int(a), bh::int(b), bh::int(c));
    bh::to_int((a * a + (b * b + c * c)).sqrt())
}

/// The sum, or 0x80000000 when it does not fit.
pub fn int_sum(a: i32, b: i32) -> i32 {
    bh::to_int(bh::int(a) + bh::int(b))
}

/// x1² + y1² against x0² + y0², the status word's C0 .. C2: 0x100 when the first is the
/// smaller, 0 otherwise.
pub fn compare_squares(x0: i32, y0: i32, x1: i32, y1: i32) -> u16 {
    let (x0, y0, x1, y1) = (bh::int(x0), bh::int(y0), bh::int(x1), bh::int(y1));
    let old = x0 * x0 + y0 * y0;
    let new = x1 * x1 + y1 * y1;
    if new < old { 0x100 } else { 0 }
}

/// `a / n`, a whole number over a whole number.
pub fn int_div_int(a: i32, n: i32) -> u32 {
    bh::st(bh::int(a) / bh::int(n))
}

/// `a - b`, kept as an 80-bit value.
pub fn sub_80(a: u32, b: u32) -> [u8; 10] {
    bh::f80(bh::ld(a) - bh::ld(b))
}

/// `a - b` (a whole number), kept as an 80-bit value.
pub fn int_sub_80(a: i32, b: u32) -> [u8; 10] {
    bh::f80(bh::int(a) - bh::ld(b))
}

/// `a + b`, as a whole number.
pub fn add_to_int(a: u32, b: u32) -> i32 {
    bh::to_int(bh::ld(a) + bh::ld(b))
}

/// `frndint`: `x` rounded to a whole number as this thread's control word says, kept a
/// double.
pub fn round_f64(x: f64) -> f64 {
    match rounding() {
        Rounding::Nearest => x.round_ties_even(),
        Rounding::Down => x.floor(),
        Rounding::Up => x.ceil(),
        Rounding::Chop => x.trunc(),
    }
}

// ---- part B (the helpers from here to the end) ----
//
// Plain Rust (phase 6, 2026-10-24), as part A: a value on the x87 stack is an `X`
// (`x87_float.rs`: an `f64` rounding as the control word says), a dword float stored is
// `bh::st`, a whole number loaded is `bh::int`, stored `bh::to_int`, an 80-bit value kept
// is converted at the boundary (`bh::f80`, `bh::from_f80`). The order of the operations is
// the original's. The assembly versions are in `x87_reference_b.rs`, which the tests
// compare these against.

/// `fld a; fsub b; [fchs;] fcomp c; fstsw`: the status word's C0 .. C2.
pub fn sub_compare(a: u32, b: u32, negate: bool, c: u32) -> u16 {
    let mut d = bh::ld(a) - bh::ld(b);
    if negate {
        d = -d;
    }
    bh::compare(d, bh::ld(c)) & 0x700
}

/// (r m + c) ((a - b) s + e) k with `v` [m, c, a, b, s, e, k], the factors unrounded.
pub fn random_step(r: i32, v: [u32; 7]) -> u32 {
    let first = bh::int(r) * bh::ld(v[0]) + bh::ld(v[1]);
    let second = ((bh::ld(v[2]) - bh::ld(v[3])) * bh::ld(v[4]) + bh::ld(v[5])) * bh::ld(v[6]);
    bh::st(first * second)
}

/// `((y + a) - b)` squared.
pub fn int_plus_less_squared(y: i32, a: u32, b: u32) -> u32 {
    let d = bh::int(y) + bh::ld(a) - bh::ld(b);
    bh::st(d * d)
}

/// One of the marks' kind records (0x30 bytes, 12 floats): +0xC and +0x24 negated in
/// place, then each of the twelve times a / b (kept unrounded) rounded to a whole number in
/// place.
pub fn scale_record(r: *mut u32, a: u32, b: u32) {
    span(r, 48);

    unsafe {
        for i in [3, 9] {
            r.add(i).write(bh::st(-bh::ld(r.add(i).read())));
        }
        let ratio = bh::ld(a) / bh::ld(b);
        for i in 0..12 {
            let x = bh::ld(r.add(i).read()) * ratio;
            r.add(i).write(bh::to_int(x) as u32);
        }
    }
}

/// `a m + c` stored (the first), then `(a m + c) + d - e` stored (the second).
pub fn int_times_plus_on(a: i32, m: u32, c: u32, d: u32, e: u32) -> (u32, u32) {
    let t = bh::int(a) * bh::ld(m) + bh::ld(c);
    (bh::st(t), bh::st(t + bh::ld(d) - bh::ld(e)))
}

/// `a - b - b - b`.
pub fn sub_thrice(a: u32, b: u32) -> u32 {
    bh::st(bh::ld(a) - bh::ld(b) - bh::ld(b) - bh::ld(b))
}

/// `a + b`, a whole number and a float.
pub fn int_plus(a: i32, b: u32) -> u32 {
    bh::st(bh::int(a) + bh::ld(b))
}

/// `(t - c) k + c`: c eased toward t by k.
pub fn ease(t: u32, c: u32, k: u32) -> u32 {
    bh::st((bh::ld(t) - bh::ld(c)) * bh::ld(k) + bh::ld(c))
}

/// `a + b` as a whole number.
pub fn int_plus_to_int(a: i32, b: u32) -> i32 {
    bh::to_int(bh::int(a) + bh::ld(b))
}

/// `0 - a + b` as a whole number.
pub fn zero_less_int_plus_int(a: i32, b: i32) -> i32 {
    bh::to_int(bh::X(0.0) - bh::int(a) + bh::int(b))
}

/// `f - a + b` as a whole number.
pub fn less_int_plus_int(f: u32, a: i32, b: i32) -> i32 {
    bh::to_int(bh::ld(f) - bh::int(a) + bh::int(b))
}

/// 2 pi less a. The x87's `pi` constant has 64 bits (0xC90FDAA22168C235, one less when
/// the rounding is down or chop), so 2 pi is `2 PI` plus a small low part, and the
/// difference is rounded once, as the x87 rounds it.
pub fn two_pi_less(a: u32) -> u32 {
    let low_units = match rounding() {
        Rounding::Down | Rounding::Chop => 564.0,
        _ => 565.0,
    };
    let two_pi = 2.0 * std::f64::consts::PI;
    let two_pi_low = 2.0 * low_units * 2f64.powi(-62);
    let x = f64::from(f32::from_bits(a));
    let s = two_pi - x;
    // the error of that subtraction, exactly (two-sum)
    let bb = s - two_pi;
    let err = (two_pi - (s - bb)) + (-x - bb);
    bh::st(bh::X(s) + bh::X(err + two_pi_low))
}

/// `((a m) + c) k`.
pub fn int_times_plus_times(a: i32, m: u32, c: u32, k: u32) -> u32 {
    bh::st((bh::int(a) * bh::ld(m) + bh::ld(c)) * bh::ld(k))
}

/// `a / n`.
pub fn over_int(a: u32, n: i32) -> u32 {
    bh::st(bh::ld(a) / bh::int(n))
}

/// `(±0 b + a c) k`, the zero's product kept.
pub fn zero_times_plus_times(a: u32, b: u32, c: u32, k: u32, negative: bool) -> u32 {
    let zero = bh::X(if negative { -0.0 } else { 0.0 });
    bh::st((zero * bh::ld(b) + bh::ld(a) * bh::ld(c)) * bh::ld(k))
}

/// `v` plus the remainder of `r` by `m` (`fprem`: the quotient truncated, the sign of
/// `r`). The x87 reduces by 63 bits at most in one step and the original ignores that it
/// stopped short, so for a `m` below 2^-33 (exponents 64 or more apart) its result is a
/// partial remainder that this does not reproduce: this is the complete remainder.
pub fn prem_plus(r: i32, m: u32, v: u32) -> u32 {
    bh::st(bh::int(r).fprem(bh::ld(m)) + bh::ld(v))
}

/// `v` grown by `v a r k`: `v + (r k) (v a)`.
pub fn grown(r: i32, k: u64, v: u32, a: u32) -> u32 {
    let rk = bh::int(r) * bh::X(f64::from_bits(k));
    bh::st(rk * (bh::ld(v) * bh::ld(a)) + bh::ld(v))
}

/// A rate by the difference of a and b: `k4 (one - one / (|a - b| k1 k2 + k3))`, whole.
pub fn rate_to_int(a: u32, b: u32, k1: u32, k2: u32, k3: u32, one: u32, k4: u32) -> i32 {
    let d = (bh::ld(a) - bh::ld(b)).abs() * bh::ld(k1) * bh::ld(k2) + bh::ld(k3);
    bh::to_int((bh::ld(one) - bh::ld(one) / d) * bh::ld(k4))
}

/// `a m1 + b m2 + d`, kept as 80 bits.
pub fn pair_sum_80(a: i32, m1: u32, b: i32, m2: u32, d: u32) -> [u8; 10] {
    let s = bh::int(a) * bh::ld(m1) + bh::int(b) * bh::ld(m2);
    bh::f80(s + bh::ld(d))
}

/// `e - f` (e in 80 bits) as a float.
pub fn ext_less(e: [u8; 10], f: u32) -> u32 {
    bh::st(bh::from_f80(e) - bh::ld(f))
}

/// Three products of three, each multiplied in order, summed in order: (p0 + p1) + p2.
fn three_sum(p: [[u32; 3]; 3]) -> bh::X {
    let prod = |q: [u32; 3]| bh::ld(q[0]) * bh::ld(q[1]) * bh::ld(q[2]);
    prod(p[0]) + prod(p[1]) + prod(p[2])
}

/// Three products of three, summed in order: (p0 + p1) + p2.
pub fn three_products(p: [[u32; 3]; 3]) -> u32 {
    bh::st(three_sum(p))
}

/// `a b` stored (the first), `|a b| c` as a whole number (the second).
pub fn times_kept_abs_to_int(a: u32, b: u32, c: u32) -> (u32, i32) {
    let t = bh::ld(a) * bh::ld(b);
    (bh::st(t), bh::to_int(t.abs() * bh::ld(c)))
}

/// `(a - b) k1 + b k2` stored (the first), `|that| c` as a whole number (the second).
pub fn blend_kept_abs_to_int(a: u32, b: u32, k1: u32, k2: u32, c: u32) -> (u32, i32) {
    let t = (bh::ld(a) - bh::ld(b)) * bh::ld(k1) + bh::ld(b) * bh::ld(k2);
    (bh::st(t), bh::to_int(t.abs() * bh::ld(c)))
}

/// `(f s - |s| [m]) k`.
pub fn times_less_abs_times(f: u32, s: u32, m: Option<u32>, k: u32) -> u32 {
    let mut abs = bh::ld(s).abs();
    if let Some(m) = m {
        abs *= bh::ld(m);
    }
    bh::st((bh::ld(f) * bh::ld(s) - abs) * bh::ld(k))
}

/// `d - m + f i`.
pub fn less_plus_int_times(d: u32, m: u32, f: u32, i: i32) -> u32 {
    bh::st(bh::ld(d) - bh::ld(m) + bh::ld(f) * bh::int(i))
}

/// `((a b) i [+ c]) s`, kept as 80 bits.
pub fn times_int_plus_times_80(a: u32, b: u32, i: i32, c: Option<u32>, s: u32) -> [u8; 10] {
    let mut x = bh::ld(a) * bh::ld(b) * bh::int(i);
    if let Some(c) = c {
        x += bh::ld(c);
    }
    bh::f80(x * bh::ld(s))
}

/// `a m1 + b m2 + d - e`.
pub fn pair_sum_less(a: i32, m1: u32, b: i32, m2: u32, d: u32, e: u32) -> u32 {
    let s = bh::int(a) * bh::ld(m1) + bh::int(b) * bh::ld(m2);
    bh::st(s + bh::ld(d) - bh::ld(e))
}

/// `d + a b c`.
pub fn product_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(bh::ld(d) + bh::ld(a) * bh::ld(b) * bh::ld(c))
}

/// `f i` as a whole number.
pub fn float_times_int_to_int(f: u32, i: i32) -> i32 {
    bh::to_int(bh::ld(f) * bh::int(i))
}

/// A rate by a: `k4 (one - one / (a k1 k2 + k3))`, whole.
pub fn speed_rate_to_int(a: u32, k1: u32, k2: u32, k3: u32, one: u32, k4: u32) -> i32 {
    let d = bh::ld(a) * bh::ld(k1) * bh::ld(k2) + bh::ld(k3);
    bh::to_int((bh::ld(one) - bh::ld(one) / d) * bh::ld(k4))
}

/// `|a| + |b| + |a - c| + d` as a whole number (the first) and as a float (the second).
pub fn abs_sum_to_int_kept(a: u32, b: u32, c: u32, d: u32) -> (i32, u32) {
    let s = bh::ld(a).abs() + bh::ld(b).abs() + (bh::ld(a) - bh::ld(c)).abs() + bh::ld(d);
    (bh::to_int(s), bh::st(s))
}

/// `|a| k1 + |b - c|`, times k2, stored (the first); its status word against k3 (the
/// second, C3, C2 and C0 only).
pub fn abs_blend_compare(a: u32, k1: u32, b: u32, c: u32, k2: u32, k3: u32) -> (u32, u16) {
    let t = (bh::ld(a).abs() * bh::ld(k1) + (bh::ld(b) - bh::ld(c)).abs()) * bh::ld(k2);
    (bh::st(t), bh::compare(t, bh::ld(k3)))
}

/// `v a + b` as a whole number.
pub fn times_plus_to_int(v: u32, a: u32, b: u32) -> i32 {
    bh::to_int(bh::ld(v) * bh::ld(a) + bh::ld(b))
}

/// `d + f i`, kept as 80 bits.
pub fn plus_int_times_80(d: u32, f: u32, i: i32) -> [u8; 10] {
    bh::f80(bh::ld(d) + bh::ld(f) * bh::int(i))
}

/// `three_products` kept as 80 bits.
pub fn three_products_80(p: [[u32; 3]; 3]) -> [u8; 10] {
    bh::f80(three_sum(p))
}

/// `a k` against c: the status word's C3, C2 and C0.
pub fn times_compare(a: u32, k: u32, c: u32) -> u16 {
    bh::compare(bh::ld(a) * bh::ld(k), bh::ld(c))
}

/// `a k n` as a whole number.
pub fn times_int_to_int(a: u32, k: u32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) * bh::ld(k) * bh::int(n))
}

/// `a b + c` stored (the first) and negated (the second).
pub fn int_times_plus_and_negated(a: i32, b: u32, c: u32) -> (u32, u32) {
    let t = bh::int(a) * bh::ld(b) + bh::ld(c);
    (bh::st(t), bh::st(-t))
}

/// `a b - c - d`.
pub fn int_times_less_less(a: i32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b) - bh::ld(c) - bh::ld(d))
}

/// `(|a| + |b|) k` as a whole number.
pub fn abs_sum_times_to_int(a: u32, b: u32, k: u32) -> i32 {
    bh::to_int((bh::ld(a).abs() + bh::ld(b).abs()) * bh::ld(k))
}

/// The point (a, b, c) (a on top of the x87 stack) by the matrix at `m` (its rows 0x10
/// apart, as floats): x = (m0 a + (m4 b + m8 c)) [+ m12], y from m1, m5, m9 [m13], z from
/// m2, m6, m10 [m14], unrounded.
fn matrix_turn(m: *const u32, a: bh::X, b: bh::X, c: bh::X, translate: bool) -> [bh::X; 3] {
    span(m, 0x3C);
    let mut out = [bh::X(0.0); 3];
    for (col, o) in out.iter_mut().enumerate() {
        let at = |row: usize| bh::ld(unsafe { m.add(row * 4 + col).read() });
        let mut x = at(0) * a + (at(1) * b + at(2) * c);
        if translate {
            x += at(3);
        }
        *o = x;
    }
    out
}

/// `sub_4CE05E`'s point: (-t0 k, (t1 - d) k, t2 k) turned by the matrix at `m` (its rows
/// 0x10 apart), placed by +0x30 .. and the place taken off again; then x, y, y less
/// `lift`, z.
pub fn turned_place_less(t: [u32; 3], k: u32, d: u32, lift: u32, m: u32) -> [u32; 4] {
    let kf = bh::ld(k);
    let a = -bh::ld(t[0]) * kf;
    let b = (bh::ld(t[1]) - bh::ld(d)) * kf;
    let c = bh::ld(t[2]) * kf;
    let m = crate::pointers::host(m) as *const u32;
    let [x, y, z] = matrix_turn(m, c, b, a, true);
    let at = |i: usize| bh::ld(unsafe { m.add(i).read() });
    [
        bh::st(x - at(12)),
        bh::st(y - at(13)),
        bh::st(y - at(13) - bh::ld(lift)),
        bh::st(z - at(14)),
    ]
}

/// `r k` kept; then for each of three: `((p r k) ticks) + at`.
pub fn drifted(r: i32, k: u32, p: [u32; 3], ticks: i32, at: [u32; 3]) -> [u32; 3] {
    let rk = bh::int(r) * bh::ld(k);
    let one = |i: usize| bh::st(bh::ld(p[i]) * rk * bh::int(ticks) + bh::ld(at[i]));
    [one(0), one(1), one(2)]
}

/// `1 - 1 / (a k + 1)`.
pub fn one_less_inverse(a: u32, k: u32, one: u32) -> u32 {
    let d = bh::ld(a) * bh::ld(k) + bh::ld(one);
    bh::st(bh::ld(one) - bh::ld(one) / d)
}

/// `a k / n`.
pub fn times_over_int(a: u32, k: u32, n: i32) -> u32 {
    bh::st(bh::ld(a) * bh::ld(k) / bh::int(n))
}

/// `a b`, kept as 80 bits.
pub fn mul_80(a: u32, b: u32) -> [u8; 10] {
    bh::f80(bh::ld(a) * bh::ld(b))
}

/// The point (p0, p1, p2) turned and placed by the matrix at `m`; then x, y, y less
/// `lift`, z.
pub fn turned_place(p: [u32; 3], lift: u32, m: u32) -> [u32; 4] {
    let m = crate::pointers::host(m) as *const u32;
    let [x, y, z] = matrix_turn(m, bh::ld(p[0]), bh::ld(p[1]), bh::ld(p[2]), true);
    [bh::st(x), bh::st(y), bh::st(y - bh::ld(lift)), bh::st(z)]
}

/// `a b` as a whole number.
pub fn int_times_int_to_int(a: i32, b: i32) -> i32 {
    bh::to_int(bh::int(a) * bh::int(b))
}

/// `(a + b + c + d) k`.
pub fn sum4_times(a: u32, b: u32, c: u32, d: u32, k: u32) -> u32 {
    bh::st((bh::ld(a) + bh::ld(b) + bh::ld(c) + bh::ld(d)) * bh::ld(k))
}

/// The length of (p0, p1, p2): sqrt(p0 p0 + (p1 p1 + p2 p2)), unrounded.
fn length3(p: [u32; 3]) -> bh::X {
    let (x, y, z) = (bh::ld(p[0]), bh::ld(p[1]), bh::ld(p[2]));
    (x * x + (y * y + z * z)).sqrt()
}

/// The length of (a, b, c) as a whole number (the first); then `k3 (one - one / (length
/// k1 + k2))` as a whole number (the second).
pub fn length_and_rate(p: [u32; 3], k1: u32, k2: u32, one: u32, k3: u32) -> (i32, i32) {
    let length = length3(p);
    let d = length * bh::ld(k1) + bh::ld(k2);
    (
        bh::to_int(length),
        bh::to_int((bh::ld(one) - bh::ld(one) / d) * bh::ld(k3)),
    )
}

/// (a, b, c) by one over its length, the factor kept unrounded.
pub fn normalised3(p: [u32; 3], one: u32) -> [u32; 3] {
    let inverse = bh::ld(one) / length3(p);
    [0, 1, 2].map(|i| bh::st(bh::ld(p[i]) * inverse))
}

/// `sub_4C9AF6`'s third side: the cross product of (p0, p1, p2) and the vector at `w`
/// (unrounded) turned by the transposed matrix at `m` (rows 0x10 apart).
pub fn crossed_turned(p: [u32; 3], w: u32, m: u32) -> [u32; 3] {
    let wp = crate::pointers::host(w) as *const u32;
    let m = crate::pointers::host(m) as *const u32;
    span(wp, 12);
    let (p0, p1, p2) = (bh::ld(p[0]), bh::ld(p[1]), bh::ld(p[2]));
    let (w0, w1, w2) = unsafe {
        (
            bh::ld(wp.read()),
            bh::ld(wp.add(1).read()),
            bh::ld(wp.add(2).read()),
        )
    };
    let cz = p0 * w1 - p1 * w0;
    let cy = p2 * w0 - p0 * w2;
    let cx = p1 * w2 - p2 * w1;
    let [x, y, z] = matrix_turn(m, cx, cy, cz, false);
    [bh::st(x), bh::st(y), bh::st(z)]
}

/// The vector at `p` made of length 1: the length stored (the second), each component by
/// `1 / length`, rounded.
pub fn unit_and_length(p: [u32; 3]) -> ([u32; 3], u32) {
    let length = length3(p);
    let inverse = bh::X(1.0) / length;
    let out = [0, 1, 2].map(|i| bh::st(bh::ld(p[i]) * inverse));
    (out, bh::st(length))
}

/// `-a - b + c + d`.
pub fn negated_less_plus_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(-bh::ld(a) - bh::ld(b) + bh::ld(c) + bh::ld(d))
}

/// The sine (the first) and cosine (the second) of `t`.
pub fn sine_and_cosine(t: u32) -> (u32, u32) {
    let t = bh::ld(t);
    (bh::st(t.sin()), bh::st(t.cos()))
}

/// `sub_4C1194`'s turn of (y, z) by the stored sine and cosine: z s + y c and z c - y s.
pub fn turn_yz(y: u32, z: u32, s: u32, c: u32) -> (u32, u32) {
    let (y, z, s, c) = (bh::ld(y), bh::ld(z), bh::ld(s), bh::ld(c));
    (bh::st(z * s + y * c), bh::st(z * c - y * s))
}

/// `a k + b` as a whole number.
pub fn int_times_plus_int_to_int(a: i32, k: u32, b: i32) -> i32 {
    bh::to_int(bh::int(a) * bh::ld(k) + bh::int(b))
}

/// `a ka + (b kb + c kc)`, kept as 80 bits.
pub fn blend3_80(a: i32, b: i32, c: i32, k: [u32; 3]) -> [u8; 10] {
    let first = bh::int(a) * bh::ld(k[0]);
    let second = bh::int(b) * bh::ld(k[1]);
    let third = bh::int(c) * bh::ld(k[2]);
    bh::f80(first + (second + third))
}

/// `a + (v / kd) km` stored (the first), its sine times ks stored (the second).
pub fn angle_and_wave(a: u32, v: i32, kd: u32, km: u32, ks: u32) -> (u32, u32) {
    let angle = bh::ld(a) + bh::int(v) / bh::ld(kd) * bh::ld(km);
    (bh::st(angle), bh::st(angle.sin() * bh::ld(ks)))
}

/// `b + (a - b) f`, as a whole number.
pub fn int_lerp_to_int(a: i32, b: i32, f: u32) -> i32 {
    bh::to_int((bh::int(a) - bh::int(b)) * bh::ld(f) + bh::int(b))
}

/// `a / b k`.
pub fn int_over_int_times(a: i32, b: i32, k: u32) -> u32 {
    bh::st(bh::int(a) / bh::int(b) * bh::ld(k))
}

/// `sub_4B8A0B`'s crossing: t = (m - fc) / (fs - fc), then for each (s, c) pair
/// (s - c) t + c, t kept unrounded.
pub fn clip_lerp3(m: u32, fc: u32, fs: u32, pairs: [[u32; 2]; 3]) -> [u32; 3] {
    let t = (bh::ld(m) - bh::ld(fc)) / (bh::ld(fs) - bh::ld(fc));
    pairs.map(|[s, c]| bh::st((bh::ld(s) - bh::ld(c)) * t + bh::ld(c)))
}

/// `sub_4C969A`'s projection: q = focal / z, kept; x' = q x + cx, y' = -(q y) + cy,
/// s = q k.
pub fn slot_project(focal: u32, z: u32, x: u32, cx: u32, y: u32, cy: u32, k: u32) -> [u32; 3] {
    let q = bh::ld(focal) / bh::ld(z);
    [
        bh::st(q * bh::ld(x) + bh::ld(cx)),
        bh::st(-(q * bh::ld(y)) + bh::ld(cy)),
        bh::st(q * bh::ld(k)),
    ]
}

/// `v - k` (v in 80 bits) as a float.
pub fn ext_sub(v: [u8; 10], k: u32) -> u32 {
    bh::st(bh::from_f80(v) - bh::ld(k))
}

/// `i a b`.
pub fn int_times_times(i: i32, a: u32, b: u32) -> u32 {
    bh::st(bh::int(i) * bh::ld(a) * bh::ld(b))
}

/// `a + (b - a) r`.
pub fn toward(a: u32, b: u32, r: u32) -> u32 {
    bh::st(bh::ld(a) + (bh::ld(b) - bh::ld(a)) * bh::ld(r))
}

/// `(b - a) k + c m`.
pub fn diff_times_plus_times(b: u32, a: u32, k: u32, c: u32, m: u32) -> u32 {
    bh::st((bh::ld(b) - bh::ld(a)) * bh::ld(k) + bh::ld(c) * bh::ld(m))
}

/// `sub_4CFDD5`'s turn: the point p (x, y, z, 80 bits) by the matrix at m read by
/// columns, each (m[c] x + (m[c+4] y + m[c+8] z)) + m[c+12]; the three rounded.
pub fn column_turn(p: &[[u8; 10]; 3], m: u32) -> [u32; 3] {
    let m = crate::pointers::host(m) as *const u32;
    let [x, y, z] = {
        matrix_turn(
            m,
            bh::from_f80(p[0]),
            bh::from_f80(p[1]),
            bh::from_f80(p[2]),
            true,
        )
    };
    [bh::st(x), bh::st(y), bh::st(z)]
}

/// `sub_4CFDD5`'s square: q = focal / c, kept; z' = c zk; s = size q; Y = -(q b) + cy and
/// X = q a + cx kept unrounded, the corners Y - s, Y - s, Y + s, Y + s and X - s, X + s,
/// X + s, X - s (s as stored). Out: [s, z', y0 .. y3, x0 .. x3].
pub fn particle_square(a: u32, b: u32, c: u32, k: [u32; 4], size: i32) -> [u32; 10] {
    // k: focal, cx, cy, zk
    let q = bh::ld(k[0]) / bh::ld(c);
    let zp = bh::st(bh::ld(c) * bh::ld(k[3]));
    let s_bits = bh::st(bh::int(size) * q);
    let s = bh::ld(s_bits);
    let y = -(q * bh::ld(b)) + bh::ld(k[2]);
    let x = q * bh::ld(a) + bh::ld(k[1]);
    [
        s_bits,
        zp,
        bh::st(y - s),
        bh::st(y - s),
        bh::st(y + s),
        bh::st(y + s),
        bh::st(x - s),
        bh::st(x + s),
        bh::st(x + s),
        bh::st(x - s),
    ]
}

/// `a + v t`.
pub fn plus_times_int(a: u32, v: u32, t: i32) -> u32 {
    bh::st(bh::ld(v) * bh::int(t) + bh::ld(a))
}

/// `a + (v + w) t`.
pub fn plus_sum_times_int(a: u32, v: u32, w: u32, t: i32) -> u32 {
    bh::st((bh::ld(v) + bh::ld(w)) * bh::int(t) + bh::ld(a))
}

/// `a - a k`.
pub fn less_own_times(a: u32, k: u32) -> u32 {
    bh::st(bh::ld(a) - bh::ld(a) * bh::ld(k))
}

/// `k t + c` as a whole number.
pub fn int_times_int_plus_int(k: i32, t: i32, c: i32) -> i32 {
    bh::to_int(bh::int(k) * bh::int(t) + bh::int(c))
}

/// `sin` (or `cos`) of `r k1 k2 k3`, times m, plus a.
pub fn int_times3_wave_plus(r: i32, k: [u32; 3], m: u32, a: u32, cosine: bool) -> u32 {
    let x = bh::int(r) * bh::ld(k[0]) * bh::ld(k[1]) * bh::ld(k[2]);
    let wave = if cosine { x.cos() } else { x.sin() };
    bh::st(wave * bh::ld(m) + bh::ld(a))
}

/// `(r k1 + a) k2 + b`.
pub fn int_times_plus_times_plus(r: i32, k1: u32, a: u32, k2: u32, b: u32) -> u32 {
    bh::st((bh::int(r) * bh::ld(k1) + bh::ld(a)) * bh::ld(k2) + bh::ld(b))
}

/// `sub_4CBD12`'s square: q = focal / c, kept; z' = c zk; s = size q; the half sides
/// (s, s) and (s, -s) turned by sin / cos (u' = v cos + u sin, v' = -v sin + u cos, the
/// sides as stored); Y = -(q b) + cy and X = q a + cx kept unrounded, the corners Y - v1,
/// Y - v2, Y + v1, Y + v2 and X - u1, X - u2, X + u1, X + u2. Out: [s, z', u1, v1, u2, v2,
/// y0 .. y3, x0 .. x3].
pub fn turned_sprite(
    a: u32,
    b: u32,
    c: u32,
    k: [u32; 4],
    size: i32,
    sin: u32,
    cos: u32,
) -> [u32; 14] {
    // k: focal, cx, cy, zk
    let q = bh::ld(k[0]) / bh::ld(c);
    let zp = bh::st(bh::ld(c) * bh::ld(k[3]));
    let s = bh::int(size) * q;
    let s_bits = bh::st(s);
    let neg_s_bits = bh::st(-s);
    let (sn, cs) = (bh::ld(sin), bh::ld(cos));
    // (u, v) turned: u' = v cos + u sin, v' = -v sin + u cos
    let turn = |u: bh::X, v: bh::X| (bh::st(v * cs + u * sn), bh::st(-v * sn + u * cs));
    let (u1, v1) = turn(bh::ld(s_bits), bh::ld(s_bits));
    let (u2, v2) = turn(bh::ld(s_bits), bh::ld(neg_s_bits));
    let (u1f, v1f, u2f, v2f) = (bh::ld(u1), bh::ld(v1), bh::ld(u2), bh::ld(v2));
    let y = -(q * bh::ld(b)) + bh::ld(k[2]);
    let x = q * bh::ld(a) + bh::ld(k[1]);
    [
        s_bits,
        zp,
        u1,
        v1,
        u2,
        v2,
        bh::st(y - v1f),
        bh::st(y - v2f),
        bh::st(y + v1f),
        bh::st(y + v2f),
        bh::st(x - u1f),
        bh::st(x - u2f),
        bh::st(x + u1f),
        bh::st(x + u2f),
    ]
}

/// `a + b`, left in 80 bits.
pub fn add_80(a: u32, b: u32) -> [u8; 10] {
    bh::f80(bh::ld(a) + bh::ld(b))
}

/// `a / b`, two whole numbers, as a float.
pub fn int_over_int(a: i32, b: i32) -> u32 {
    bh::st(bh::int(a) / bh::int(b))
}

/// A whole number as a float.
pub fn int_to_float(a: i32) -> u32 {
    bh::st(bh::int(a))
}

/// `sub_4C4FFB`'s turn of a corner (x, y) by sin / cos about (cx, cy): x' = (x cos + y
/// sin) + cx, y' = (-x sin + y cos) + cy.
pub fn turn_about(x: u32, y: u32, sin: u32, cos: u32, cx: u32, cy: u32) -> [u32; 2] {
    let (x, y, sn, cs) = (bh::ld(x), bh::ld(y), bh::ld(sin), bh::ld(cos));
    [
        bh::st(x * cs + y * sn + bh::ld(cx)),
        bh::st(-x * sn + y * cs + bh::ld(cy)),
    ]
}

/// `c - v k` as a whole number.
pub fn times_subr_to_int(v: u32, k: u32, c: u32) -> i32 {
    bh::to_int(bh::ld(c) - bh::ld(v) * bh::ld(k))
}

/// `[-]a b + c`, left in 80 bits.
pub fn times_plus_80(a: u32, b: u32, c: u32, negated: bool) -> [u8; 10] {
    let mut x = bh::ld(a);
    if negated {
        x = -x;
    }
    bh::f80(x * bh::ld(b) + bh::ld(c))
}

/// `k t + c` as a whole number.
pub fn times_int_plus_int_to_int(k: u32, t: i32, c: i32) -> i32 {
    bh::to_int(bh::ld(k) * bh::int(t) + bh::int(c))
}

/// `v - v k t`.
pub fn damp(v: u32, k: u32, t: i32) -> u32 {
    bh::st(bh::ld(v) - bh::ld(v) * bh::ld(k) * bh::int(t))
}

/// `v + (i - v) k t`, as a whole number and as a float.
pub fn approach(i: i32, v: u32, k: u32, t: i32) -> (i32, u32) {
    let x = (bh::int(i) - bh::ld(v)) * bh::ld(k) * bh::int(t) + bh::ld(v);
    (bh::to_int(x), bh::st(x))
}

/// `a - v t`.
pub fn less_times_int(a: u32, v: u32, t: i32) -> u32 {
    bh::st(bh::ld(a) - bh::ld(v) * bh::int(t))
}

/// `a + b k` as a whole number.
pub fn plus_product_to_int(a: u32, b: u32, k: u32) -> i32 {
    bh::to_int(bh::ld(a) + bh::ld(b) * bh::ld(k))
}

/// `i a b` as a whole number.
pub fn int_times_times_to_int(i: i32, a: u32, b: u32) -> i32 {
    bh::to_int(bh::int(i) * bh::ld(a) * bh::ld(b))
}

/// `-(a k + c)`, left in 80 bits.
pub fn times_plus_negated_80(a: u32, k: u32, c: u32) -> [u8; 10] {
    bh::f80(-(bh::ld(a) * bh::ld(k) + bh::ld(c)))
}

/// `-((a - b) k + c)`, left in 80 bits.
pub fn less_times_plus_negated_80(a: u32, b: u32, k: u32, c: u32) -> [u8; 10] {
    bh::f80(-((bh::ld(a) - bh::ld(b)) * bh::ld(k) + bh::ld(c)))
}

/// The x87 control word set to chop (`and 0F3FFh; or 0C00h`), the old one returned.
pub fn set_chop() -> u16 {
    let old = control_word();
    set_control((old & 0xF3FF) | 0x0C00);
    old
}

/// The x87 control word put back.
pub fn set_control(cw: u16) {
    set_control_word(cw);
}

/// `v` rounded to a whole number (as the rounding is set), its fraction times k, as a
/// whole number.
pub fn fraction_times_to_int(v: u32, k: u32) -> i32 {
    let i = bh::to_int(bh::ld(v));
    bh::to_int((bh::ld(v) - bh::int(i)) * bh::ld(k))
}

/// `a / b` as a whole number.
pub fn int_over_int_to_int(a: i32, b: i32) -> i32 {
    bh::to_int(bh::int(a) / bh::int(b))
}

/// `(a + b + c) k`.
pub fn sum3_times(a: u32, b: u32, c: u32, k: u32) -> u32 {
    bh::st((bh::ld(a) + bh::ld(b) + bh::ld(c)) * bh::ld(k))
}
