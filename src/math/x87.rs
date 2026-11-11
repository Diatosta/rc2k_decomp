//! Arithmetic the original did with the x87, where a result has to be the original's to
//! the last bit: plain Rust on `X` (`x87_float.rs`), an `f64` that rounds as the control
//! word says, in the original's order of operations; `rcpss` is an approximation that
//! differs between processors, taken from one processor's tables (`arch::rcpss`). No x87
//! instruction runs outside the tests (2026-10-25).
//! Floats are given as `impl Flt` (bits, an `f32` or the cell holding it); a result is the
//! `f32` the original stored, or whole number, status word or bits where the original
//! kept those.

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

/// `fstp dword`: a result stored as a float, as the control word says.
fn out(x: bh::X) -> f32 {
    f32::from_bits(bh::st(x))
}

mod load;
mod run;
pub use load::*;
pub use run::*;

/// `bytes` bytes at `p` checked to lie in one region (the program ends otherwise): what a
/// helper below reads and writes, checked before it starts.
#[cfg(all(test, target_arch = "x86"))]
fn span<T>(p: *const T, bytes: usize) {
    crate::pointers::check_or_end(
        crate::pointers::game_address(p),
        bytes,
        "a math helper's operand",
    );
}

/// A float read from game memory (possibly unaligned), as the x87 holds it once loaded.
#[cfg(all(test, target_arch = "x86"))] // the pointer forms' tests only
unsafe fn load(p: *const u32, index: usize) -> bh::X {
    bh::ld(unsafe { p.add(index).read_unaligned() })
}

/// A float given as it is held: its bits, a float, or the cell holding it (read when the
/// helper starts, as the helper's `ld`).
pub trait Flt {
    fn bits(&self) -> u32;
}

impl Flt for u32 {
    fn bits(&self) -> u32 {
        *self
    }
}

impl Flt for f32 {
    fn bits(&self) -> u32 {
        self.to_bits()
    }
}

impl<C: crate::session::Cell32 + ?Sized> Flt for C {
    fn bits(&self) -> u32 {
        crate::session::Cell32::bits(self)
    }
}

/// `a + b`.
pub fn add(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    out(bh::ld(a) + bh::ld(b))
}

/// `a - b`.
pub fn sub(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    out(bh::ld(a) - bh::ld(b))
}

/// A whole number as a float.
pub fn from_int(a: i32) -> f32 {
    out(bh::int(a))
}

/// `a * b`.
pub fn mul(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    out(bh::ld(a) * bh::ld(b))
}

/// `1 / a * b`.
pub fn reciprocal_times(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    out(bh::ONE / bh::ld(a) * bh::ld(b))
}

/// `x` mirrored about a + b: `-(x - a - b) + b + a`.
pub fn mirrored(x: impl Flt, a: impl Flt, b: impl Flt) -> f32 {
    let x = x.bits();
    let a = a.bits();
    let b = b.bits();
    out(-(bh::ld(x) - bh::ld(a) - bh::ld(b)) + bh::ld(b) + bh::ld(a))
}

/// A float as it comes back from a register.
pub fn loaded(a: impl Flt) -> f32 {
    let a = a.bits();
    out(bh::ld(a))
}

/// The whole number `a` rounds to, as the control word says.
pub fn to_int(a: impl Flt) -> i32 {
    let a = a.bits();
    bh::to_int(bh::ld(a))
}

/// The product as a whole number.
pub fn mul_to_int(a: impl Flt, b: impl Flt) -> i32 {
    let a = a.bits();
    let b = b.bits();
    bh::to_int(bh::ld(a) * bh::ld(b))
}

/// `a / b`, two whole numbers, as a float.
pub fn int_ratio(a: i32, b: i32) -> f32 {
    out(bh::int(a) / bh::int(b))
}

/// `1 - a / b`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn one_less_int_ratio(a: i32, b: i32) -> u32 {
    bh::st(bh::ONE - bh::int(a) / bh::int(b))
}

/// `rcpss xmm0, a; mulss xmm0, b` (`rcpss` an SSE approximation, from the tables of
/// `arch::rcpss`: the processors' own results differ).
pub fn rcpss_times(a: u32, b: u32) -> u32 {
    (crate::arch::rcpss(f32::from_bits(a)) * f32::from_bits(b)).to_bits()
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
pub fn int_over_16(a: i32) -> f32 {
    out(bh::int(a) / bh::X(16.0))
}

/// The length of two whole numbers at right angles, as a whole number:
/// `sqrt(a * a + b * b)`.
pub fn int_hypot(a: i32, b: i32) -> i32 {
    let (a, b) = (bh::int(a), bh::int(b));
    bh::to_int((a * a + b * b).sqrt())
}

/// `a / b`.
pub fn div(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    out(bh::ld(a) / bh::ld(b))
}

/// `a * b - c * d`.
pub fn product_difference(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let d = d.bits();
    out(bh::ld(a) * bh::ld(b) - bh::ld(c) * bh::ld(d))
}

/// The same negated.
#[cfg(all(test, target_arch = "x86"))] // replaced by `negated` of the plain step; the assembly comparison keeps it
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
pub fn equal_or_unordered(a: impl Flt, b: impl Flt) -> bool {
    let a = a.bits();
    let b = b.bits();
    let (a, b) = (bh::ld(a), bh::ld(b));
    matches!(a.partial_cmp(&b), None | Some(std::cmp::Ordering::Equal))
}

/// The sine and the cosine of a whole number divided three times: `angle / a / a / b`.
pub fn sine_cosine(angle: i32, a: impl Flt, b: impl Flt) -> (u32, u32) {
    let a = a.bits();
    let b = b.bits();
    let x = bh::int(angle) / bh::ld(a) / bh::ld(a) / bh::ld(b);
    let (sin, cos) = x.sin_cos();
    (bh::st(sin), bh::st(cos))
}

/// The root of a whole number over a float.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_over_root(a: i32, d: u32) -> u32 {
    bh::st((bh::int(a) / bh::ld(d)).sqrt())
}

/// `sqrt(a)`.
pub fn sqrt(a: impl Flt) -> f32 {
    let a = a.bits();
    out(bh::ld(a).sqrt())
}

/// The sine of n times pi over d: `sin(n * pi / d)`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn sine_of_pi_fraction(n: i32, d: i32) -> u32 {
    bh::st((bh::int(n).times_pi() / bh::int(d)).sin())
}

/// `a / b * m * n`, as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn ratio_scaled_to_int(a: u32, b: u32, m: i32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) / bh::ld(b) * bh::int(m) * bh::int(n))
}

/// The angle of (x, y) over pi, times two floats, as a whole number:
/// `atan2(y, x) / pi * a * b`.
pub fn angle_scaled_to_int(y: i32, x: i32, a: impl Flt, b: impl Flt) -> i32 {
    let a = a.bits();
    let b = b.bits();
    let angle = bh::int(y).atan2_over_pi(bh::int(x));
    bh::to_int(angle * bh::ld(a) * bh::ld(b))
}

/// A whole number times a float.
pub fn int_times(a: i32, b: impl Flt) -> f32 {
    let b = b.bits();
    out(bh::int(a) * bh::ld(b))
}

/// A whole number over a float.
pub fn int_over(a: i32, b: impl Flt) -> f32 {
    let b = b.bits();
    out(bh::int(a) / bh::ld(b))
}

/// A whole number less a float.
pub fn int_less(a: i32, b: impl Flt) -> f32 {
    let b = b.bits();
    out(bh::int(a) - bh::ld(b))
}

/// `1 / a`.
pub fn reciprocal(a: impl Flt) -> f32 {
    let a = a.bits();
    out(bh::ONE / bh::ld(a))
}

/// Two whole numbers and two floats multiplied: `a * b * c * d`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_float_int_float(a: i32, b: u32, c: i32, d: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b) * bh::int(c) * bh::ld(d))
}

/// `a * b / c / d` as a whole number.
pub fn mul_div_div_to_int(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> i32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let d = d.bits();
    bh::to_int(bh::ld(a) * bh::ld(b) / bh::ld(c) / bh::ld(d))
}

/// `race::normalise_work_vector`'s sequence: x, y, z squared and summed (y and z first), the root
/// taken and compared with `zero`; unless equal (or unordered), each of the three is divided by the
/// root (not rounded to a float first) and stored back; then the root is stored. Returns
/// the root's bits and whether it compared equal.
pub fn normalise_in_place(v: &mut [u32; 3], zero: impl Flt) -> (f32, bool) {
    let zero = zero.bits();
    let [x, y, z] = v.map(bh::ld);
    let root = (x * x + (y * y + z * z)).sqrt();
    let zero = bh::ld(zero);
    let equal = matches!(
        root.partial_cmp(&zero),
        None | Some(std::cmp::Ordering::Equal)
    );
    if !equal {
        *v = [bh::st(x / root), bh::st(y / root), bh::st(z / root)];
    }
    (out(root), equal)
}

/// `a * b - c * d`, whole numbers, rounded once to a float.
pub fn int_product_difference(a: i32, b: i32, c: i32, d: i32) -> f32 {
    out(bh::int(a) * bh::int(b) - bh::int(c) * bh::int(d))
}

/// A whole number over two floats, doubled, rounded once: `a / b / c` added to itself.
pub fn int_div_div_doubled(a: i32, b: impl Flt, c: impl Flt) -> f32 {
    let b = b.bits();
    let c = c.bits();
    let q = bh::int(a) / bh::ld(b) / bh::ld(c);
    out(q + q)
}

/// `math::rotate_scratch_about_y`'s turn: x' = x cos + z sin, z' = z cos - x sin, each rounded
/// once, written back over x and z.
pub fn turn_in_place(sine: impl Flt, cosine: impl Flt, x: &mut u32, z: &mut u32) {
    let sine = sine.bits();
    let cosine = cosine.bits();
    let (s, c) = (bh::ld(sine), bh::ld(cosine));
    let (xv, zv) = (bh::ld(*x), bh::ld(*z));
    let new_x = xv * c + zv * s;
    let new_z = c * zv - s * xv;
    *z = bh::st(new_z);
    *x = bh::st(new_x);
}

/// `race::add_surface_share`'s sum: `a / b * c / d * e + f`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn surface_share(a: i32, b: u32, c: u32, d: i32, e: u32, f: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b) * bh::ld(c) / bh::int(d) * bh::ld(e) + bh::ld(f))
}

/// `atan2(y, x) / pi * a * b * c`, as a whole number.
pub fn angle_scaled3_to_int(y: i32, x: i32, a: impl Flt, b: impl Flt, c: impl Flt) -> i32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let angle = bh::int(y).atan2_over_pi(bh::int(x));
    bh::to_int(angle * bh::ld(a) * bh::ld(b) * bh::ld(c))
}

/// `atan2(y, x) / pi * a * a * a * b * b`, as a whole number.
pub fn angle_scaled5_to_int(y: i32, x: i32, a: impl Flt, b: impl Flt) -> i32 {
    let a = a.bits();
    let b = b.bits();
    let angle = bh::int(y).atan2_over_pi(bh::int(x));
    bh::to_int(angle * bh::ld(a) * bh::ld(a) * bh::ld(a) * bh::ld(b) * bh::ld(b))
}

/// The length of (a, b) over that of (c, d), whole numbers, as a float.
pub fn hypot_ratio(a: i32, b: i32, c: i32, d: i32) -> f32 {
    let (a, b, c, d) = (bh::int(a), bh::int(b), bh::int(c), bh::int(d));
    let first = (a * a + b * b).sqrt();
    let second = (c * c + d * d).sqrt();
    out(first / second)
}

/// A whole number times a float, as a whole number.
pub fn int_times_to_int(a: i32, b: impl Flt) -> i32 {
    let b = b.bits();
    bh::to_int(bh::int(a) * bh::ld(b))
}

/// A whole number over a float, less a float, rounded once.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_div_less(a: i32, b: u32, c: u32) -> u32 {
    bh::st(bh::int(a) / bh::ld(b) - bh::ld(c))
}

/// `out.len()` sines of an angle that starts at 0 and grows by `step` each time (the angle
/// kept as a double, never rounded to a float), one dword each.
pub fn sine_table(step: impl Flt, out: &mut [u32]) {
    let step = step.bits();
    let step = bh::ld(step);
    let mut angle = bh::X(0.0);
    for o in out {
        *o = bh::st(angle.sin());
        angle += step;
    }
}

/// `1 / a` stored, and that (unrounded) times a float stored.
pub fn reciprocal_int_and_times(a: i32, b: impl Flt) -> (f32, f32) {
    let b = b.bits();
    let r = bh::ONE / bh::int(a);
    (out(r), out(r * bh::ld(b)))
}

/// `a * b` stored, and that (unrounded) over `c` stored.
pub fn mul_and_div(a: impl Flt, b: impl Flt, c: impl Flt) -> (f32, f32) {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let p = bh::ld(a) * bh::ld(b);
    (out(p), out(p / bh::ld(c)))
}

/// `(a - b) / c * d`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn sub_div_mul(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st((bh::ld(a) - bh::ld(b)) / bh::ld(c) * bh::ld(d))
}

/// `(a - b) / (c - d) * e`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn diff_ratio_mul(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    bh::st((bh::ld(a) - bh::ld(b)) / (bh::ld(c) - bh::ld(d)) * bh::ld(e))
}

/// `((a - b) m + b) n`, less `less` (none: nothing taken off), rounded once.
pub fn lerp_scaled_less(
    a: impl Flt,
    b: impl Flt,
    m: impl Flt,
    n: impl Flt,
    less: Option<u32>,
) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let m = m.bits();
    let n = n.bits();
    let scaled = ((bh::ld(a) - bh::ld(b)) * bh::ld(m) + bh::ld(b)) * bh::ld(n);
    match less {
        Some(less) => out(scaled - bh::ld(less)),
        None => out(scaled),
    }
}

/// `-(a - b)`.
#[cfg(all(test, target_arch = "x86"))] // replaced by `negated` of the plain step; the assembly comparison keeps it
pub fn negated_sub(a: u32, b: u32) -> u32 {
    bh::st(-(bh::ld(a) - bh::ld(b)))
}

/// `a x + (b y + c z)`, whole numbers in, a whole number out.
pub fn weighted3_to_int(a: i32, b: i32, c: i32, x: impl Flt, y: impl Flt, z: impl Flt) -> i32 {
    let x = x.bits();
    let y = y.bits();
    let z = z.bits();
    let first = bh::int(a) * bh::ld(x);
    let second = bh::int(b) * bh::ld(y);
    let third = bh::int(c) * bh::ld(z);
    bh::to_int(first + (second + third))
}

/// `a * b + c`.
pub fn int_times_plus(a: i32, b: impl Flt, c: impl Flt) -> f32 {
    let b = b.bits();
    let c = c.bits();
    out(bh::int(a) * bh::ld(b) + bh::ld(c))
}

/// The step `(last - first) / by`, then for each of the whole numbers in `values` the
/// number less the running sum, rounded to a whole number and stored back, and the sum
/// grown by the step (both kept as doubles).
pub fn ramp_down(values: &mut [i32], last: i32, first: i32, by: i32) {
    let step = (bh::int(last) - bh::int(first)) / bh::int(by);
    let mut sum = bh::X(0.0);
    for v in values {
        *v = bh::to_int(bh::int(*v) - sum);
        sum += step;
    }
}

/// The difference of two whole numbers, rounded once to a float.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_less_int(a: i32, b: i32) -> u32 {
    bh::st(bh::int(a) - bh::int(b))
}

/// `(a - b) / c`, a and b whole numbers.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_less_int_over(a: i32, b: i32, c: u32) -> u32 {
    bh::st((bh::int(a) - bh::int(b)) / bh::ld(c))
}

/// `(a - b) / c`.
pub fn sub_div(a: impl Flt, b: impl Flt, c: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    out((bh::ld(a) - bh::ld(b)) / bh::ld(c))
}

/// `sin(a) b + c`, as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn sine_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    bh::to_int(bh::ld(a).sin() * bh::ld(b) + bh::ld(c))
}

/// `c - a a b`, as a whole number: `-(a a) b + c`.
pub fn negated_square_times_plus_to_int(a: impl Flt, b: impl Flt, c: impl Flt) -> i32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let a = bh::ld(a);
    bh::to_int(-(a * a) * bh::ld(b) + bh::ld(c))
}

/// The sum plus the length of (x, y, z): `sqrt(x x + (y y + z z)) + sum`.
pub fn length_plus_int(x: i32, y: i32, z: i32, sum: i32) -> i32 {
    let (x, y, z) = (bh::int(x), bh::int(y), bh::int(z));
    bh::to_int((x * x + (y * y + z * z)).sqrt() + bh::int(sum))
}

/// `c - a / b * c`, a whole number: c less a / b of c.
pub fn share_left_to_int(a: i32, b: i32, c: impl Flt) -> i32 {
    let c = c.bits();
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

/// Whether this thread rounds to nearest, as it nearly always does (asked by every
/// operation of `x87_float::X`).
#[inline]
pub fn rounds_to_nearest() -> bool {
    CONTROL.with(std::cell::Cell::get) & 0xC00 == 0
}

/// This thread's rounding (the emulated control word's bits 10 .. 11).
pub fn rounding() -> Rounding {
    match (CONTROL.with(std::cell::Cell::get) >> 10) & 3 {
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
        );
    }
}

/// The control word (`fnstcw`): this thread's emulated one.
pub fn control_word() -> u16 {
    CONTROL.with(std::cell::Cell::get)
}

/// `fldcw`: the control word set (the emulated one; in the tests the real one too).
pub fn set_control_word(w: u16) {
    CONTROL.with(|c| c.set(w));
    // The real control word only matters to the tests' reference assembly.
    #[cfg(all(test, target_arch = "x86"))]
    unsafe {
        core::arch::asm!("fldcw word ptr [{w}]", w = in(reg) &raw const w, options(nostack));
    };
}

/// `a * b * c`.
pub fn mul2(a: impl Flt, b: impl Flt, c: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    out(bh::ld(a) * bh::ld(b) * bh::ld(c))
}

/// 1 - sqrt(1 - ((i - a) b)^2), a quarter circle's curve, negated when `negate`.
pub fn circle_curve(i: i32, a: impl Flt, b: impl Flt, negate: bool) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let t = (bh::int(i) - bh::ld(a)) * bh::ld(b);
    let curve = bh::ONE - (bh::ONE - t * t).sqrt();
    out(if negate { -curve } else { curve })
}

/// `count` points round a circle of radius `r`, three dwords apart: sin(angle) r then
/// cos(angle) r, the dword between left alone. The angle starts at 0 and grows by
/// `pi / c * d` each time, kept as a double. `count` at least 1 (the original would run 2^32
/// times on 0; here nothing is done).
/// (`out` holds at least `3 (count - 1) + 2` dwords: the slice's index check.)
pub fn circle_points(r: impl Flt, out: &mut [u32], count: u32, c: i32, d: impl Flt) {
    let r = r.bits();
    let d = d.bits();
    let mut angle = bh::X(0.0);
    for i in 0..count as usize {
        let (sin, cos) = angle.sin_cos();
        out[i * 3 + 1] = bh::st(cos * bh::ld(r));
        out[i * 3] = bh::st(sin * bh::ld(r));
        angle += bh::int(c).pi_over() * bh::ld(d);
    }
}

/// `-a`.
pub fn negated(a: impl Flt) -> f32 {
    let a = a.bits();
    out(-bh::ld(a))
}

/// `(b - a) k`.
pub fn sub_mul(b: impl Flt, a: impl Flt, k: impl Flt) -> f32 {
    let b = b.bits();
    let a = a.bits();
    let k = k.bits();
    out((bh::ld(b) - bh::ld(a)) * bh::ld(k))
}

/// `a + t - t cos(pi m / c)`: `(-cos(pi / c * m)) t + a + t`.
pub fn cosine_step(c: i32, m: i32, t: impl Flt, a: impl Flt) -> f32 {
    let t = t.bits();
    let a = a.bits();
    let angle = bh::int(c).pi_over() * bh::int(m);
    out(-angle.cos() * bh::ld(t) + bh::ld(a) + bh::ld(t))
}

/// Three floats at `row` each times `c` over their length: the root of
/// `x x + (y y + z z)`, `c / root`, then each times that, rounded once and stored in turn.
/// (On a copy of the three floats' bits.)
pub fn scale_row_bits(row: [u32; 3], c: impl Flt) -> [f32; 3] {
    let c = c.bits();
    let [x, y, z] = row.map(bh::ld);
    let k = bh::ld(c) / (x * x + (y * y + z * z)).sqrt();
    [out(k * x), out(k * y), out(k * z)]
}

/// `x + (k - (y - n)) s`, n the whole number y rounds to (as the control word says),
/// which is given back too.
pub fn frac_step(y: impl Flt, k: impl Flt, s: impl Flt, x: impl Flt) -> (i32, f32) {
    let y = y.bits();
    let k = k.bits();
    let s = s.bits();
    let x = x.bits();
    let y = bh::ld(y);
    let n = bh::to_int(y);
    (
        n,
        out((bh::ld(k) - (y - bh::int(n))) * bh::ld(s) + bh::ld(x)),
    )
}

/// `a` rounded to a whole number, kept a float (as the control word says).
pub fn round_float(a: impl Flt) -> f32 {
    let a = a.bits();
    out(bh::ld(a).rint())
}

/// `a` compared with `b`: the status word's C0 .. C2 (0x700), so 0x100 when `a` is less,
/// 0x500 when either is no number, 0 otherwise (equal sets only C3, which is masked off).
pub fn compare_status(a: impl Flt, b: impl Flt) -> u16 {
    let a = a.bits();
    let b = b.bits();
    bh::compare(bh::ld(a), bh::ld(b)) & 0x700
}

/// `a - n`, as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn float_less_int_to_int(a: u32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) - bh::int(n))
}

/// x0 + ((1 / (c - a)) (b - a)) (x2 - x0), where the long edge is at b.
pub fn split_x(a: impl Flt, b: impl Flt, c: impl Flt, x0: impl Flt, x2: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let x0 = x0.bits();
    let x2 = x2.bits();
    let ratio = bh::ONE / (bh::ld(c) - bh::ld(a));
    let part = ratio * (bh::ld(b) - bh::ld(a));
    out(part * (bh::ld(x2) - bh::ld(x0)) + bh::ld(x0))
}

/// The half (b - a) k as a float, and a plus it unrounded, then as a float.
pub fn half_and_middle(b: impl Flt, a: impl Flt, k: impl Flt) -> (f32, f32) {
    let b = b.bits();
    let a = a.bits();
    let k = k.bits();
    let half = (bh::ld(b) - bh::ld(a)) * bh::ld(k);
    (out(half), out(half + bh::ld(a)))
}

/// `pi / n`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn pi_over(n: i32) -> u32 {
    bh::st(bh::int(n).pi_over())
}

/// m - h cos(a [+ s]): `-cos(a [+ s]) h + m`.
pub fn less_cosine(a: impl Flt, s: Option<u32>, h: impl Flt, m: impl Flt) -> f32 {
    let a = a.bits();
    let h = h.bits();
    let m = m.bits();
    let angle = match s {
        Some(s) => bh::ld(a) + bh::ld(s),
        None => bh::ld(a),
    };
    out(-angle.cos() * bh::ld(h) + bh::ld(m))
}

/// `sin(a)`.
pub fn sine(a: impl Flt) -> f32 {
    let a = a.bits();
    out(bh::ld(a).sin())
}

/// `cos(a)`, negated with `negate`.
pub fn cosine(a: impl Flt, negate: bool) -> f32 {
    let a = a.bits();
    let cos = bh::ld(a).cos();
    out(if negate { -cos } else { cos })
}

/// `a * b + c`.
pub fn mul_add(a: impl Flt, b: impl Flt, c: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    out(bh::ld(a) * bh::ld(b) + bh::ld(c))
}

/// The length of (a, b, c), the last two squares added first, rounded.
pub fn int_length3(a: i32, b: i32, c: i32) -> i32 {
    let (a, b, c) = (bh::int(a), bh::int(b), bh::int(c));
    bh::to_int((a * a + (b * b + c * c)).sqrt())
}

/// The sum, or 0x80000000 when it does not fit.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
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

/// `a - b`, kept as an 80-bit value.
pub fn sub_80(a: impl Flt, b: impl Flt) -> [u8; 10] {
    let a = a.bits();
    let b = b.bits();
    bh::f80(bh::ld(a) - bh::ld(b))
}

/// `a - b` (a whole number), kept as an 80-bit value.
pub fn int_sub_80(a: i32, b: impl Flt) -> [u8; 10] {
    let b = b.bits();
    bh::f80(bh::int(a) - bh::ld(b))
}

/// `a + b`, as a whole number.
pub fn add_to_int(a: impl Flt, b: impl Flt) -> i32 {
    let a = a.bits();
    let b = b.bits();
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
pub fn sub_compare(a: impl Flt, b: impl Flt, negate: bool, c: impl Flt) -> u16 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let mut d = bh::ld(a) - bh::ld(b);
    if negate {
        d = -d;
    }
    bh::compare(d, bh::ld(c)) & 0x700
}

/// (r m + c) ((a - b) s + e) k with `v` [m, c, a, b, s, e, k], the factors unrounded.
pub fn random_step(r: i32, v: [u32; 7]) -> f32 {
    let first = bh::int(r) * bh::ld(v[0]) + bh::ld(v[1]);
    let second = ((bh::ld(v[2]) - bh::ld(v[3])) * bh::ld(v[4]) + bh::ld(v[5])) * bh::ld(v[6]);
    out(first * second)
}

/// `((y + a) - b)` squared.
pub fn int_plus_less_squared(y: i32, a: impl Flt, b: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let d = bh::int(y) + bh::ld(a) - bh::ld(b);
    out(d * d)
}

/// One of the marks' kind records (0x30 bytes, 12 floats): +0xC and +0x24 negated in
/// place, then each of the twelve times a / b (kept unrounded) rounded to a whole number in
/// place.
/// (On a copy of the record's twelve floats' bits.)
pub fn scale_record_bits(mut r: [u32; 12], a: impl Flt, b: impl Flt) -> [u32; 12] {
    let a = a.bits();
    let b = b.bits();
    for i in [3, 9] {
        r[i] = bh::st(-bh::ld(r[i]));
    }
    let ratio = bh::ld(a) / bh::ld(b);
    r.map(|v| bh::to_int(bh::ld(v) * ratio) as u32)
}

/// `a m + c` stored (the first), then `(a m + c) + d - e` stored (the second).
pub fn int_times_plus_on(a: i32, m: impl Flt, c: impl Flt, d: impl Flt, e: impl Flt) -> (f32, f32) {
    let m = m.bits();
    let c = c.bits();
    let d = d.bits();
    let e = e.bits();
    let t = bh::int(a) * bh::ld(m) + bh::ld(c);
    (out(t), out(t + bh::ld(d) - bh::ld(e)))
}

/// `a - b - b - b`.
pub fn sub_thrice(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    out(bh::ld(a) - bh::ld(b) - bh::ld(b) - bh::ld(b))
}

/// `a + b`, a whole number and a float.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_plus(a: i32, b: u32) -> u32 {
    bh::st(bh::int(a) + bh::ld(b))
}

/// `(t - c) k + c`: c eased toward t by k.
pub fn ease(t: impl Flt, c: impl Flt, k: impl Flt) -> f32 {
    let t = t.bits();
    let c = c.bits();
    let k = k.bits();
    out((bh::ld(t) - bh::ld(c)) * bh::ld(k) + bh::ld(c))
}

/// `a + b` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_plus_to_int(a: i32, b: u32) -> i32 {
    bh::to_int(bh::int(a) + bh::ld(b))
}

/// `0 - a + b` as a whole number.
pub fn zero_less_int_plus_int(a: i32, b: i32) -> i32 {
    bh::to_int(bh::X(0.0) - bh::int(a) + bh::int(b))
}

/// `f - a + b` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn less_int_plus_int(f: u32, a: i32, b: i32) -> i32 {
    bh::to_int(bh::ld(f) - bh::int(a) + bh::int(b))
}

/// 2 pi less a. The x87's `pi` constant has 64 bits (0xC90FDAA22168C235, one less when
/// the rounding is down or chop), so 2 pi is `2 PI` plus a small low part, and the
/// difference is rounded once, as the x87 rounds it.
pub fn two_pi_less(a: u32) -> f32 {
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
    out(bh::X(s) + bh::X(err + two_pi_low))
}

/// `((a m) + c) k`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_times_plus_times(a: i32, m: u32, c: u32, k: u32) -> u32 {
    bh::st((bh::int(a) * bh::ld(m) + bh::ld(c)) * bh::ld(k))
}

/// `a / n`.
pub fn over_int(a: impl Flt, n: i32) -> f32 {
    let a = a.bits();
    out(bh::ld(a) / bh::int(n))
}

/// `(±0 b + a c) k`, the zero's product kept.
pub fn zero_times_plus_times(
    a: impl Flt,
    b: impl Flt,
    c: impl Flt,
    k: impl Flt,
    negative: bool,
) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let k = k.bits();
    let zero = bh::X(if negative { -0.0 } else { 0.0 });
    out((zero * bh::ld(b) + bh::ld(a) * bh::ld(c)) * bh::ld(k))
}

/// `v` plus the remainder of `r` by `m` (`fprem`: the quotient truncated, the sign of
/// `r`). The x87 reduces by 63 bits at most in one step and the original ignores that it
/// stopped short, so for a `m` below 2^-33 (exponents 64 or more apart) its result is a
/// partial remainder that this does not reproduce: this is the complete remainder.
pub fn prem_plus(r: i32, m: impl Flt, v: impl Flt) -> f32 {
    let m = m.bits();
    let v = v.bits();
    out(bh::int(r).fprem(bh::ld(m)) + bh::ld(v))
}

/// `v` grown by `v a r k`: `v + (r k) (v a)`.
pub fn grown(r: i32, k: u64, v: impl Flt, a: impl Flt) -> f32 {
    let v = v.bits();
    let a = a.bits();
    let rk = bh::int(r) * bh::X(f64::from_bits(k));
    out(rk * (bh::ld(v) * bh::ld(a)) + bh::ld(v))
}

/// A rate by the difference of a and b: `k4 (one - one / (|a - b| k1 k2 + k3))`, whole.
pub fn rate_to_int(
    a: impl Flt,
    b: impl Flt,
    k1: impl Flt,
    k2: impl Flt,
    k3: impl Flt,
    one: impl Flt,
    k4: impl Flt,
) -> i32 {
    let a = a.bits();
    let b = b.bits();
    let k1 = k1.bits();
    let k2 = k2.bits();
    let k3 = k3.bits();
    let one = one.bits();
    let k4 = k4.bits();
    let d = (bh::ld(a) - bh::ld(b)).abs() * bh::ld(k1) * bh::ld(k2) + bh::ld(k3);
    bh::to_int((bh::ld(one) - bh::ld(one) / d) * bh::ld(k4))
}

/// `a m1 + b m2 + d`, kept as 80 bits.
pub fn pair_sum_80(a: i32, m1: impl Flt, b: i32, m2: impl Flt, d: impl Flt) -> [u8; 10] {
    let m1 = m1.bits();
    let m2 = m2.bits();
    let d = d.bits();
    let s = bh::int(a) * bh::ld(m1) + bh::int(b) * bh::ld(m2);
    bh::f80(s + bh::ld(d))
}

/// `e - f` (e in 80 bits) as a float.
pub fn ext_less(e: [u8; 10], f: impl Flt) -> f32 {
    let f = f.bits();
    out(bh::from_f80(e) - bh::ld(f))
}

/// Three products of three, each multiplied in order, summed in order: (p0 + p1) + p2.
fn three_sum(p: [[u32; 3]; 3]) -> bh::X {
    let prod = |q: [u32; 3]| bh::ld(q[0]) * bh::ld(q[1]) * bh::ld(q[2]);
    prod(p[0]) + prod(p[1]) + prod(p[2])
}

/// Three products of three, summed in order: (p0 + p1) + p2.
pub fn three_products(p: [[u32; 3]; 3]) -> f32 {
    out(three_sum(p))
}

/// `a b` stored (the first), `|a b| c` as a whole number (the second).
pub fn times_kept_abs_to_int(a: impl Flt, b: impl Flt, c: impl Flt) -> (f32, i32) {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let t = bh::ld(a) * bh::ld(b);
    (out(t), bh::to_int(t.abs() * bh::ld(c)))
}

/// `(a - b) k1 + b k2` stored (the first), `|that| c` as a whole number (the second).
pub fn blend_kept_abs_to_int(
    a: impl Flt,
    b: impl Flt,
    k1: impl Flt,
    k2: impl Flt,
    c: impl Flt,
) -> (f32, i32) {
    let a = a.bits();
    let b = b.bits();
    let k1 = k1.bits();
    let k2 = k2.bits();
    let c = c.bits();
    let t = (bh::ld(a) - bh::ld(b)) * bh::ld(k1) + bh::ld(b) * bh::ld(k2);
    (out(t), bh::to_int(t.abs() * bh::ld(c)))
}

/// `(f s - |s| [m]) k`.
pub fn times_less_abs_times(f: impl Flt, s: impl Flt, m: Option<u32>, k: impl Flt) -> f32 {
    let f = f.bits();
    let s = s.bits();
    let k = k.bits();
    let mut abs = bh::ld(s).abs();
    if let Some(m) = m {
        abs *= bh::ld(m);
    }
    out((bh::ld(f) * bh::ld(s) - abs) * bh::ld(k))
}

/// `d - m + f i`.
pub fn less_plus_int_times(d: impl Flt, m: impl Flt, f: impl Flt, i: i32) -> f32 {
    let d = d.bits();
    let m = m.bits();
    let f = f.bits();
    out(bh::ld(d) - bh::ld(m) + bh::ld(f) * bh::int(i))
}

/// `((a b) i [+ c]) s`, kept as 80 bits.
pub fn times_int_plus_times_80(
    a: impl Flt,
    b: impl Flt,
    i: i32,
    c: Option<u32>,
    s: impl Flt,
) -> [u8; 10] {
    let a = a.bits();
    let b = b.bits();
    let s = s.bits();
    let mut x = bh::ld(a) * bh::ld(b) * bh::int(i);
    if let Some(c) = c {
        x += bh::ld(c);
    }
    bh::f80(x * bh::ld(s))
}

/// `a m1 + b m2 + d - e`.
pub fn pair_sum_less(a: i32, m1: impl Flt, b: i32, m2: impl Flt, d: impl Flt, e: impl Flt) -> f32 {
    let m1 = m1.bits();
    let m2 = m2.bits();
    let d = d.bits();
    let e = e.bits();
    let s = bh::int(a) * bh::ld(m1) + bh::int(b) * bh::ld(m2);
    out(s + bh::ld(d) - bh::ld(e))
}

/// `d + a b c`.
pub fn product_plus(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let d = d.bits();
    out(bh::ld(d) + bh::ld(a) * bh::ld(b) * bh::ld(c))
}

/// `f i` as a whole number.
pub fn float_times_int_to_int(f: impl Flt, i: i32) -> i32 {
    let f = f.bits();
    bh::to_int(bh::ld(f) * bh::int(i))
}

/// A rate by a: `k4 (one - one / (a k1 k2 + k3))`, whole.
pub fn speed_rate_to_int(
    a: impl Flt,
    k1: impl Flt,
    k2: impl Flt,
    k3: impl Flt,
    one: impl Flt,
    k4: impl Flt,
) -> i32 {
    let a = a.bits();
    let k1 = k1.bits();
    let k2 = k2.bits();
    let k3 = k3.bits();
    let one = one.bits();
    let k4 = k4.bits();
    let d = bh::ld(a) * bh::ld(k1) * bh::ld(k2) + bh::ld(k3);
    bh::to_int((bh::ld(one) - bh::ld(one) / d) * bh::ld(k4))
}

/// `|a| + |b| + |a - c| + d` as a whole number (the first) and as a float (the second).
pub fn abs_sum_to_int_kept(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt) -> (i32, f32) {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let d = d.bits();
    let s = bh::ld(a).abs() + bh::ld(b).abs() + (bh::ld(a) - bh::ld(c)).abs() + bh::ld(d);
    (bh::to_int(s), out(s))
}

/// `|a| k1 + |b - c|`, times k2, stored (the first); its status word against k3 (the
/// second, C3, C2 and C0 only).
pub fn abs_blend_compare(
    a: impl Flt,
    k1: impl Flt,
    b: impl Flt,
    c: impl Flt,
    k2: impl Flt,
    k3: impl Flt,
) -> (u32, u16) {
    let a = a.bits();
    let k1 = k1.bits();
    let b = b.bits();
    let c = c.bits();
    let k2 = k2.bits();
    let k3 = k3.bits();
    let t = (bh::ld(a).abs() * bh::ld(k1) + (bh::ld(b) - bh::ld(c)).abs()) * bh::ld(k2);
    (bh::st(t), bh::compare(t, bh::ld(k3)))
}

/// `v a + b` as a whole number.
pub fn times_plus_to_int(v: impl Flt, a: impl Flt, b: impl Flt) -> i32 {
    let v = v.bits();
    let a = a.bits();
    let b = b.bits();
    bh::to_int(bh::ld(v) * bh::ld(a) + bh::ld(b))
}

/// `d + f i`, kept as 80 bits.
pub fn plus_int_times_80(d: impl Flt, f: impl Flt, i: i32) -> [u8; 10] {
    let d = d.bits();
    let f = f.bits();
    bh::f80(bh::ld(d) + bh::ld(f) * bh::int(i))
}

/// `three_products` kept as 80 bits.
pub fn three_products_80(p: [[u32; 3]; 3]) -> [u8; 10] {
    bh::f80(three_sum(p))
}

/// `a k` against c: the status word's C3, C2 and C0.
pub fn times_compare(a: impl Flt, k: impl Flt, c: impl Flt) -> u16 {
    let a = a.bits();
    let k = k.bits();
    let c = c.bits();
    bh::compare(bh::ld(a) * bh::ld(k), bh::ld(c))
}

/// `a k n` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn times_int_to_int(a: u32, k: u32, n: i32) -> i32 {
    bh::to_int(bh::ld(a) * bh::ld(k) * bh::int(n))
}

/// `a b + c` stored (the first) and negated (the second).
pub fn int_times_plus_and_negated(a: i32, b: impl Flt, c: impl Flt) -> (f32, f32) {
    let b = b.bits();
    let c = c.bits();
    let t = bh::int(a) * bh::ld(b) + bh::ld(c);
    (out(t), out(-t))
}

/// `a b - c - d`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_times_less_less(a: i32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(bh::int(a) * bh::ld(b) - bh::ld(c) - bh::ld(d))
}

/// `(|a| + |b|) k` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn abs_sum_times_to_int(a: u32, b: u32, k: u32) -> i32 {
    bh::to_int((bh::ld(a).abs() + bh::ld(b).abs()) * bh::ld(k))
}

/// The matrix in `m`'s cells, as far as a turn reads it (the first 15 dwords).
fn matrix_at(m: &(impl crate::math::cells::Cells + ?Sized)) -> [u32; 15] {
    crate::math::cells::load(m)
}

/// The point (a, b, c) (a on top of the x87 stack) by the matrix at `m` (its rows 0x10
/// apart, as floats): x = (m0 a + (m4 b + m8 c)) [+ m12], y from m1, m5, m9 [m13], z from
/// m2, m6, m10 [m14], unrounded.
fn matrix_turn(m: &[u32; 15], a: bh::X, b: bh::X, c: bh::X, translate: bool) -> [bh::X; 3] {
    let mut out = [bh::X(0.0); 3];
    for (col, o) in out.iter_mut().enumerate() {
        let at = |row: usize| bh::ld(m[row * 4 + col]);
        let mut x = at(0) * a + (at(1) * b + at(2) * c);
        if translate {
            x += at(3);
        }
        *o = x;
    }
    out
}

/// `effects::emit_exhaust_smoke`'s point: (-t0 k, (t1 - d) k, t2 k) turned by the matrix at `m` (its
/// rows 0x10 apart), placed by +0x30 .. and the place taken off again; then x, y, y less
/// `lift`, z.
pub fn turned_place_less(
    t: [u32; 3],
    k: impl Flt,
    d: impl Flt,
    lift: impl Flt,
    m: &(impl crate::math::cells::Cells + ?Sized),
) -> [u32; 4] {
    let k = k.bits();
    let d = d.bits();
    let lift = lift.bits();
    let kf = bh::ld(k);
    let a = -bh::ld(t[0]) * kf;
    let b = (bh::ld(t[1]) - bh::ld(d)) * kf;
    let c = bh::ld(t[2]) * kf;
    let m = matrix_at(m);
    let [x, y, z] = matrix_turn(&m, c, b, a, true);
    let at = |i: usize| bh::ld(m[i]);
    [
        bh::st(x - at(12)),
        bh::st(y - at(13)),
        bh::st(y - at(13) - bh::ld(lift)),
        bh::st(z - at(14)),
    ]
}

/// `r k` kept; then for each of three: `((p r k) ticks) + at`.
pub fn drifted(r: i32, k: impl Flt, p: [u32; 3], ticks: i32, at: [u32; 3]) -> [f32; 3] {
    let k = k.bits();
    let rk = bh::int(r) * bh::ld(k);
    let one = |i: usize| out(bh::ld(p[i]) * rk * bh::int(ticks) + bh::ld(at[i]));
    [one(0), one(1), one(2)]
}

/// `1 - 1 / (a k + 1)`.
pub fn one_less_inverse(a: impl Flt, k: impl Flt, one: impl Flt) -> f32 {
    let a = a.bits();
    let k = k.bits();
    let one = one.bits();
    let d = bh::ld(a) * bh::ld(k) + bh::ld(one);
    out(bh::ld(one) - bh::ld(one) / d)
}

/// `a k / n`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn times_over_int(a: u32, k: u32, n: i32) -> u32 {
    bh::st(bh::ld(a) * bh::ld(k) / bh::int(n))
}

/// `a b`, kept as 80 bits.
pub fn mul_80(a: impl Flt, b: impl Flt) -> [u8; 10] {
    let a = a.bits();
    let b = b.bits();
    bh::f80(bh::ld(a) * bh::ld(b))
}

/// The point (p0, p1, p2) turned and placed by the matrix at `m`; then x, y, y less
/// `lift`, z.
pub fn turned_place(
    p: [u32; 3],
    lift: impl Flt,
    m: &(impl crate::math::cells::Cells + ?Sized),
) -> [u32; 4] {
    let lift = lift.bits();
    let [x, y, z] = matrix_turn(
        &matrix_at(m),
        bh::ld(p[0]),
        bh::ld(p[1]),
        bh::ld(p[2]),
        true,
    );
    [bh::st(x), bh::st(y), bh::st(y - bh::ld(lift)), bh::st(z)]
}

/// `a b` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_times_int_to_int(a: i32, b: i32) -> i32 {
    bh::to_int(bh::int(a) * bh::int(b))
}

/// `(a + b + c + d) k`.
pub fn sum4_times(a: impl Flt, b: impl Flt, c: impl Flt, d: impl Flt, k: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let d = d.bits();
    let k = k.bits();
    out((bh::ld(a) + bh::ld(b) + bh::ld(c) + bh::ld(d)) * bh::ld(k))
}

/// The length of (p0, p1, p2): sqrt(p0 p0 + (p1 p1 + p2 p2)), unrounded.
fn length3(p: [u32; 3]) -> bh::X {
    let (x, y, z) = (bh::ld(p[0]), bh::ld(p[1]), bh::ld(p[2]));
    (x * x + (y * y + z * z)).sqrt()
}

/// The length of (a, b, c) as a whole number (the first); then `k3 (one - one / (length
/// k1 + k2))` as a whole number (the second).
pub fn length_and_rate(
    p: [u32; 3],
    k1: impl Flt,
    k2: impl Flt,
    one: impl Flt,
    k3: impl Flt,
) -> (i32, i32) {
    let k1 = k1.bits();
    let k2 = k2.bits();
    let one = one.bits();
    let k3 = k3.bits();
    let length = length3(p);
    let d = length * bh::ld(k1) + bh::ld(k2);
    (
        bh::to_int(length),
        bh::to_int((bh::ld(one) - bh::ld(one) / d) * bh::ld(k3)),
    )
}

/// (a, b, c) by one over its length, the factor kept unrounded.
pub fn normalised3(p: [u32; 3], one: impl Flt) -> [f32; 3] {
    let one = one.bits();
    let inverse = bh::ld(one) / length3(p);
    [0, 1, 2].map(|i| out(bh::ld(p[i]) * inverse))
}

/// `effects::cars_particles`'s third side: the cross product of (p0, p1, p2) and the vector at `w`
/// (unrounded) turned by the transposed matrix at `m` (rows 0x10 apart).
pub fn crossed_turned(
    p: [u32; 3],
    w: &(impl crate::math::cells::Cells + ?Sized),
    m: &(impl crate::math::cells::Cells + ?Sized),
) -> [u32; 3] {
    let [w0, w1, w2]: [u32; 3] = crate::math::cells::load(w);
    let (w0, w1, w2) = (bh::ld(w0), bh::ld(w1), bh::ld(w2));
    let (p0, p1, p2) = (bh::ld(p[0]), bh::ld(p[1]), bh::ld(p[2]));
    let cz = p0 * w1 - p1 * w0;
    let cy = p2 * w0 - p0 * w2;
    let cx = p1 * w2 - p2 * w1;
    let [x, y, z] = matrix_turn(&matrix_at(m), cx, cy, cz, false);
    [bh::st(x), bh::st(y), bh::st(z)]
}

/// The vector at `p` made of length 1: the length stored (the second), each component by
/// `1 / length`, rounded.
pub fn unit_and_length(p: [u32; 3]) -> ([f32; 3], f32) {
    let length = length3(p);
    let inverse = bh::X(1.0) / length;
    let unit = [0, 1, 2].map(|i| out(bh::ld(p[i]) * inverse));
    (unit, out(length))
}

/// `-a - b + c + d`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn negated_less_plus_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    bh::st(-bh::ld(a) - bh::ld(b) + bh::ld(c) + bh::ld(d))
}

/// The sine (the first) and cosine (the second) of `t`.
pub fn sine_and_cosine(t: impl Flt) -> (f32, f32) {
    let t = t.bits();
    let t = bh::ld(t);
    (out(t.sin()), out(t.cos()))
}

/// `effects::tilt_shape`'s turn of (y, z) by the stored sine and cosine: z s + y c and z c - y s.
pub fn turn_yz(y: impl Flt, z: impl Flt, s: impl Flt, c: impl Flt) -> (f32, f32) {
    let y = y.bits();
    let z = z.bits();
    let s = s.bits();
    let c = c.bits();
    let (y, z, s, c) = (bh::ld(y), bh::ld(z), bh::ld(s), bh::ld(c));
    (out(z * s + y * c), out(z * c - y * s))
}

/// `a k + b` as a whole number.
pub fn int_times_plus_int_to_int(a: i32, k: impl Flt, b: i32) -> i32 {
    let k = k.bits();
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
pub fn angle_and_wave(a: impl Flt, v: i32, kd: impl Flt, km: impl Flt, ks: impl Flt) -> (f32, f32) {
    let a = a.bits();
    let kd = kd.bits();
    let km = km.bits();
    let ks = ks.bits();
    let angle = bh::ld(a) + bh::int(v) / bh::ld(kd) * bh::ld(km);
    (out(angle), out(angle.sin() * bh::ld(ks)))
}

/// `b + (a - b) f`, as a whole number.
pub fn int_lerp_to_int(a: i32, b: i32, f: impl Flt) -> i32 {
    let f = f.bits();
    bh::to_int((bh::int(a) - bh::int(b)) * bh::ld(f) + bh::int(b))
}

/// `a / b k`.
pub fn int_over_int_times(a: i32, b: i32, k: impl Flt) -> f32 {
    let k = k.bits();
    out(bh::int(a) / bh::int(b) * bh::ld(k))
}

/// `effects::clip_quad`'s crossing: t = (m - fc) / (fs - fc), then for each (s, c) pair
/// (s - c) t + c, t kept unrounded.
pub fn clip_lerp3(m: impl Flt, fc: impl Flt, fs: impl Flt, pairs: [[u32; 2]; 3]) -> [f32; 3] {
    let m = m.bits();
    let fc = fc.bits();
    let fs = fs.bits();
    let t = (bh::ld(m) - bh::ld(fc)) / (bh::ld(fs) - bh::ld(fc));
    pairs.map(|[s, c]| out((bh::ld(s) - bh::ld(c)) * t + bh::ld(c)))
}

/// `effects::draw_mark_slots`'s projection: q = focal / z, kept; x' = q x + cx, y' = -(q y) + cy,
/// s = q k.
pub fn slot_project(
    focal: impl Flt,
    z: impl Flt,
    x: impl Flt,
    cx: impl Flt,
    y: impl Flt,
    cy: impl Flt,
    k: impl Flt,
) -> [u32; 3] {
    let focal = focal.bits();
    let z = z.bits();
    let x = x.bits();
    let cx = cx.bits();
    let y = y.bits();
    let cy = cy.bits();
    let k = k.bits();
    let q = bh::ld(focal) / bh::ld(z);
    [
        bh::st(q * bh::ld(x) + bh::ld(cx)),
        bh::st(-(q * bh::ld(y)) + bh::ld(cy)),
        bh::st(q * bh::ld(k)),
    ]
}

/// `i a b`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_times_times(i: i32, a: u32, b: u32) -> u32 {
    bh::st(bh::int(i) * bh::ld(a) * bh::ld(b))
}

/// `a + (b - a) r`.
pub fn toward(a: impl Flt, b: impl Flt, r: impl Flt) -> f32 {
    let a = a.bits();
    let b = b.bits();
    let r = r.bits();
    out(bh::ld(a) + (bh::ld(b) - bh::ld(a)) * bh::ld(r))
}

/// `(b - a) k + c m`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn diff_times_plus_times(b: u32, a: u32, k: u32, c: u32, m: u32) -> u32 {
    bh::st((bh::ld(b) - bh::ld(a)) * bh::ld(k) + bh::ld(c) * bh::ld(m))
}

/// `effects::draw_particle`'s turn: the point p (x, y, z, 80 bits) by the matrix at m read by
/// columns, each (m[c] x + (m[c+4] y + m[c+8] z)) + m[c+12]; the three rounded.
pub fn column_turn(p: &[[u8; 10]; 3], m: &(impl crate::math::cells::Cells + ?Sized)) -> [f32; 3] {
    let [x, y, z] = {
        matrix_turn(
            &matrix_at(m),
            bh::from_f80(p[0]),
            bh::from_f80(p[1]),
            bh::from_f80(p[2]),
            true,
        )
    };
    [out(x), out(y), out(z)]
}

/// `effects::draw_particle`'s square: q = focal / c, kept; z' = c zk; s = size q; Y = -(q b) + cy
/// and X = q a + cx kept unrounded, the corners Y - s, Y - s, Y + s, Y + s and X - s, X + s,
/// X + s, X - s (s as stored). Out: [s, z', y0 .. y3, x0 .. x3].
pub fn particle_square(a: impl Flt, b: impl Flt, c: impl Flt, k: [u32; 4], size: i32) -> [f32; 10] {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    // k: focal, cx, cy, zk
    let q = bh::ld(k[0]) / bh::ld(c);
    let zp = out(bh::ld(c) * bh::ld(k[3]));
    let s_bits = out(bh::int(size) * q);
    let s = bh::ld(s_bits.to_bits());
    let y = -(q * bh::ld(b)) + bh::ld(k[2]);
    let x = q * bh::ld(a) + bh::ld(k[1]);
    [
        s_bits,
        zp,
        out(y - s),
        out(y - s),
        out(y + s),
        out(y + s),
        out(x - s),
        out(x + s),
        out(x + s),
        out(x - s),
    ]
}

/// `a + v t`.
pub fn plus_times_int(a: impl Flt, v: impl Flt, t: i32) -> f32 {
    let a = a.bits();
    let v = v.bits();
    out(bh::ld(v) * bh::int(t) + bh::ld(a))
}

/// `a + (v + w) t`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn plus_sum_times_int(a: u32, v: u32, w: u32, t: i32) -> u32 {
    bh::st((bh::ld(v) + bh::ld(w)) * bh::int(t) + bh::ld(a))
}

/// `a - a k`.
pub fn less_own_times(a: impl Flt, k: impl Flt) -> f32 {
    let a = a.bits();
    let k = k.bits();
    out(bh::ld(a) - bh::ld(a) * bh::ld(k))
}

/// `k t + c` as a whole number.
pub fn int_times_int_plus_int(k: i32, t: i32, c: i32) -> i32 {
    bh::to_int(bh::int(k) * bh::int(t) + bh::int(c))
}

/// `sin` (or `cos`) of `r k1 k2 k3`, times m, plus a.
pub fn int_times3_wave_plus(r: i32, k: [u32; 3], m: impl Flt, a: impl Flt, cosine: bool) -> f32 {
    let m = m.bits();
    let a = a.bits();
    let x = bh::int(r) * bh::ld(k[0]) * bh::ld(k[1]) * bh::ld(k[2]);
    let wave = if cosine { x.cos() } else { x.sin() };
    out(wave * bh::ld(m) + bh::ld(a))
}

/// `(r k1 + a) k2 + b`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_times_plus_times_plus(r: i32, k1: u32, a: u32, k2: u32, b: u32) -> u32 {
    bh::st((bh::int(r) * bh::ld(k1) + bh::ld(a)) * bh::ld(k2) + bh::ld(b))
}

/// `effects::draw_turned_sprite`'s square: q = focal / c, kept; z' = c zk; s = size q; the half
/// sides (s, s) and (s, -s) turned by sin / cos (u' = v cos + u sin, v' = -v sin + u cos, the
/// sides as stored); Y = -(q b) + cy and X = q a + cx kept unrounded, the corners Y - v1,
/// Y - v2, Y + v1, Y + v2 and X - u1, X - u2, X + u1, X + u2. Out: [s, z', u1, v1, u2, v2,
/// y0 .. y3, x0 .. x3].
pub fn turned_sprite(
    a: impl Flt,
    b: impl Flt,
    c: impl Flt,
    k: [u32; 4],
    size: i32,
    sin: impl Flt,
    cos: impl Flt,
) -> [u32; 14] {
    let sin = sin.bits();
    let cos = cos.bits();
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
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
pub fn add_80(a: impl Flt, b: impl Flt) -> [u8; 10] {
    let a = a.bits();
    let b = b.bits();
    bh::f80(bh::ld(a) + bh::ld(b))
}

/// `effects::draw_flare_part`'s turn of a corner (x, y) by sin / cos about (cx, cy): x' = (x cos +
/// y sin) + cx, y' = (-x sin + y cos) + cy.
pub fn turn_about(
    x: impl Flt,
    y: impl Flt,
    sin: impl Flt,
    cos: impl Flt,
    cx: impl Flt,
    cy: impl Flt,
) -> [f32; 2] {
    let (x, y) = (x.bits(), y.bits());
    let sin = sin.bits();
    let cos = cos.bits();
    let cx = cx.bits();
    let cy = cy.bits();
    let (x, y, sn, cs) = (bh::ld(x), bh::ld(y), bh::ld(sin), bh::ld(cos));
    [
        out(x * cs + y * sn + bh::ld(cx)),
        out(-x * sn + y * cs + bh::ld(cy)),
    ]
}

/// `c - v k` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn times_subr_to_int(v: u32, k: u32, c: u32) -> i32 {
    bh::to_int(bh::ld(c) - bh::ld(v) * bh::ld(k))
}

/// `[-]a b + c`, left in 80 bits.
pub fn times_plus_80(a: impl Flt, b: impl Flt, c: impl Flt, negated: bool) -> [u8; 10] {
    let a = a.bits();
    let b = b.bits();
    let c = c.bits();
    let mut x = bh::ld(a);
    if negated {
        x = -x;
    }
    bh::f80(x * bh::ld(b) + bh::ld(c))
}

/// `k t + c` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn times_int_plus_int_to_int(k: u32, t: i32, c: i32) -> i32 {
    bh::to_int(bh::ld(k) * bh::int(t) + bh::int(c))
}

/// `v - v k t`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn damp(v: u32, k: u32, t: i32) -> u32 {
    bh::st(bh::ld(v) - bh::ld(v) * bh::ld(k) * bh::int(t))
}

/// `v + (i - v) k t`, as a whole number and as a float.
pub fn approach(i: i32, v: impl Flt, k: impl Flt, t: i32) -> (i32, f32) {
    let v = v.bits();
    let k = k.bits();
    let x = (bh::int(i) - bh::ld(v)) * bh::ld(k) * bh::int(t) + bh::ld(v);
    (bh::to_int(x), out(x))
}

/// `a - v t`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn less_times_int(a: u32, v: u32, t: i32) -> u32 {
    bh::st(bh::ld(a) - bh::ld(v) * bh::int(t))
}

/// `a + b k` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn plus_product_to_int(a: u32, b: u32, k: u32) -> i32 {
    bh::to_int(bh::ld(a) + bh::ld(b) * bh::ld(k))
}

/// `i a b` as a whole number.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn int_times_times_to_int(i: i32, a: u32, b: u32) -> i32 {
    bh::to_int(bh::int(i) * bh::ld(a) * bh::ld(b))
}

/// `-(a k + c)`, left in 80 bits.
pub fn times_plus_negated_80(a: impl Flt, k: impl Flt, c: impl Flt) -> [u8; 10] {
    let a = a.bits();
    let k = k.bits();
    let c = c.bits();
    bh::f80(-(bh::ld(a) * bh::ld(k) + bh::ld(c)))
}

/// `-((a - b) k + c)`, left in 80 bits.
pub fn less_times_plus_negated_80(a: impl Flt, b: impl Flt, k: impl Flt, c: impl Flt) -> [u8; 10] {
    let a = a.bits();
    let b = b.bits();
    let k = k.bits();
    let c = c.bits();
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
pub fn fraction_times_to_int(v: impl Flt, k: impl Flt) -> i32 {
    let v = v.bits();
    let k = k.bits();
    let i = bh::to_int(bh::ld(v));
    bh::to_int((bh::ld(v) - bh::int(i)) * bh::ld(k))
}

/// `a / b` as a whole number.
pub fn int_over_int_to_int(a: i32, b: i32) -> i32 {
    bh::to_int(bh::int(a) / bh::int(b))
}

/// `(a + b + c) k`.
#[cfg(all(test, target_arch = "x86"))] // inlined at its caller; the assembly comparison keeps it
pub fn sum3_times(a: u32, b: u32, c: u32, k: u32) -> u32 {
    bh::st((bh::ld(a) + bh::ld(b) + bh::ld(c)) * bh::ld(k))
}
