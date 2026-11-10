//! The scene's matrices and the functions that make them (converted 2026-10-21 from
//! `scene::multiply_matrices` 0x42720D, `copy_matrix_with_place` 0x4271E4, `set_place`
//! 0x4271AE, `set_turn_x` 0x426FE1, `set_turn_y` 0x42702A, `set_turn_z_16` 0x427072,
//! `set_fine_turn_x` 0x4270BA, `set_fine_turn_y` 0x42710C, `set_turn_z` 0x42715D,
//! `unit_vector` 0x420EF8, `cross_product` 0x420F8D, `square_matrix` 0x420E94,
//! `invert_turn` 0x420CC2, `set_light_direction` 0x427326, `make_object_matrix` 0x427812,
//! `set_view_matrices` 0x438FBF, `cross_product3` 0x465551 and `adjugate_matrix`
//! 0x4BCF6B).
//!
//! A matrix is sixteen floats, rows 0x10 bytes apart; a point is turned by the first
//! three rows and columns and moved by the fourth row's first three floats. Matrices and
//! vectors are `Places` (they may overlap, as the original's pointers may), each part
//! written before the next is worked out. As in `math::vector`, what the x87 keeps on its
//! stack is an `f64`, rounded to `f32` where the original stores it: the same bits at the
//! game's double precision (but for the sines and cosines, `scene::trig`).

use super::places::{Places, loaded};
use super::trig;
use crate::math::vector;

/// A matrix's bytes.
pub const MATRIX_LEN: u32 = 0x40;

/// Row `row`, column `column` of the matrix at `m`.
fn cell(m: u32, row: u32, column: u32) -> u32 {
    m.wrapping_add(row * 0x10 + column * 4)
}

/// `scene::multiply_matrices`: column by column for the first three, row by row in each,
/// `out`'s cell the row of `a` times the column of `b` (the last two products added
/// first); then `b`'s fourth-row float of that column added to `out`'s. `out`'s fourth
/// column is left as it is.
pub fn multiply(p: &mut impl Places, a: u32, b: u32, out: u32) {
    for column in 0..3 {
        for row in 0..4 {
            let r: [f32; 3] = p.floats(cell(a, row, 0));
            let c = [0, 1, 2].map(|k| p.float(cell(b, k, column)));
            p.set_float(cell(out, row, column), vector::dot(r, c) as f32);
        }
        add_into(p, cell(b, 3, column), cell(out, 3, column));
    }
}

/// `fld a; fadd out; fstp out`.
fn add_into(p: &mut impl Places, a: u32, out: u32) {
    let sum = p.float(a) as f64 + p.float(out) as f64;
    p.set_float(out, sum as f32);
}

/// `scene::copy_matrix_with_place`: `out` = `from`, dword by dword upwards, then the
/// first three floats of `other`'s fourth row added to `out`'s.
pub fn copy_with_place(p: &mut impl Places, from: u32, other: u32, out: u32) {
    copy_matrix(p, from, out);
    for n in 0..3 {
        add_into(p, cell(other, 3, n), cell(out, 3, n));
    }
}

/// Sixteen dwords from `from` to `out`, upwards.
fn copy_matrix(p: &mut impl Places, from: u32, out: u32) {
    for n in 0..16 {
        let d = p.dword(from.wrapping_add(4 * n));
        p.set_dword(out.wrapping_add(4 * n), d);
    }
}

/// Where the matrix functions' matrices, work and constants are (the game's variables;
/// the names are the hook crate's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub turn_x: u32,
    pub turn_y: u32,
    pub turn_z: u32,
    /// Its fourth row the place.
    pub placed: u32,
    pub turned: u32,
    pub lit: u32,
    pub view_turn: u32,
    pub object: u32,
    pub identity: u32,
    pub kept_place: u32,
    /// The view's turning undone.
    pub view_back: u32,
    pub view: u32,
    /// Each turning's sine and cosine.
    pub turn_x_work: u32,
    pub turn_y_work: u32,
    pub turn_z_work: u32,
    /// The light's direction, three floats.
    pub light_direction: u32,
    /// `dword_62400C`, the scratch several functions leave a value in.
    pub checked: u32,
    /// The squaring's three work rows (and the four dwords after them).
    pub squared: u32,
    /// The inverting's nine floats copied and nine made.
    pub inverted: u32,
    /// The tables of 0x1000 sines and cosines.
    pub sines: u32,
    pub cosines: u32,
    /// The fine turns' 65536.0 and 1 / (2 pi), and the 0.0 lengths are compared with.
    pub half_turns: u32,
    pub turn: u32,
    pub nothing: u32,
}

/// The axis of a turning matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    /// Its matrix and its work.
    pub fn places(self, l: &Layout) -> (u32, u32) {
        match self {
            Axis::X => (l.turn_x, l.turn_x_work),
            Axis::Y => (l.turn_y, l.turn_y_work),
            Axis::Z => (l.turn_z, l.turn_z_work),
        }
    }

    /// Where the cosine, the sine, the sine negated and the cosine go, in the order the
    /// table turnings write them.
    fn cells(self) -> [(u32, Part); 4] {
        use Part::*;
        match self {
            Axis::X => [
                (0x14, Cosine),
                (0x18, Sine),
                (0x24, Negated),
                (0x28, Cosine),
            ],
            Axis::Y => [(0, Cosine), (8, Negated), (0x20, Sine), (0x28, Cosine)],
            Axis::Z => [(0, Cosine), (4, Sine), (0x10, Negated), (0x14, Cosine)],
        }
    }
}

#[derive(Clone, Copy)]
enum Part {
    Sine,
    Cosine,
    Negated,
}

/// A turning's sine and cosine (as dwords, the matrices take them as they are).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turn {
    pub sine: u32,
    pub cosine: u32,
}

impl Turn {
    /// The sine with its sign bit flipped.
    pub fn negated(self) -> u32 {
        self.sine ^ 0x8000_0000
    }
}

/// The tables' entry of an angle of sixteen bits a turn (the low four dropped).
pub fn table_entry(angle: u32) -> u32 {
    (angle & 0xFFFF) >> 4
}

/// The turning's work and the four cells of its matrix written.
fn put_turn(p: &mut impl Places, axis: Axis, l: &Layout, t: Turn) {
    let (matrix, work) = axis.places(l);
    p.set_dword(work, t.sine);
    p.set_dword(work.wrapping_add(4), t.cosine);
    for (at, part) in axis.cells() {
        let value = match part {
            Part::Sine => t.sine,
            Part::Cosine => t.cosine,
            Part::Negated => t.negated(),
        };
        p.set_dword(matrix.wrapping_add(at), value);
    }
}

/// `scene::set_turn_x`, `set_turn_y`, `set_turn_z_16`: the turning about `axis` by an
/// angle of sixteen bits a turn, from the tables.
pub fn set_table_turn(p: &mut impl Places, l: &Layout, axis: Axis, angle: u32) -> Turn {
    let entry = 4 * table_entry(angle);
    let t = Turn {
        sine: p.dword(l.sines.wrapping_add(entry)),
        cosine: p.dword(l.cosines.wrapping_add(entry)),
    };
    put_turn(p, axis, l, t);
    t
}

/// `fild angle; fdiv h; fdiv h; fdiv turn; fsincos`, the two stored.
pub fn fine_sine_cosine(angle: i32, half_turns: f32, turn: f32) -> Turn {
    let (h, turn) = (half_turns as f64, turn as f64);
    let x = angle as f64 / h / h / turn;
    Turn {
        sine: (trig::sine(x) as f32).to_bits(),
        cosine: (trig::cosine(x) as f32).to_bits(),
    }
}

/// `scene::set_fine_turn_x`, `set_fine_turn_y`, `set_turn_z`: the angle (32 bits a turn)
/// left in the scratch, the turning about `axis` worked out.
pub fn set_fine_turn(p: &mut impl Places, l: &Layout, axis: Axis, angle: u32) -> Turn {
    p.set_dword(l.checked, angle);
    let t = fine_sine_cosine(angle as i32, p.float(l.half_turns), p.float(l.turn));
    put_turn(p, axis, l, t);
    t
}

/// `scene::set_place`: the place's whole numbers into `placed`'s fourth row, its third
/// (as it is) into the scratch.
pub fn set_place(p: &mut impl Places, l: &Layout, place: [u32; 3]) {
    let floats = place.map(|v| v as i32 as f32);
    p.set_floats(cell(l.placed, 3, 0), &floats);
    p.set_dword(l.checked, place[2]);
}

/// Whether two floats compare as equal on the x87 (`fcomp`'s C3), which they also do when
/// either is no number.
fn equal_or_unordered(a: f32, b: f32) -> bool {
    a == b || a.is_nan() || b.is_nan()
}

/// `scene::unit_vector`: the three floats at `from` made one long, into `out`; the length
/// (squares added, the last two first) into the scratch; all zeros when it compares equal
/// to the layout's nothing; minus zeros made zero.
pub fn unit_vector(p: &mut impl Places, l: &Layout, from: u32, out: u32) {
    let length = vector::length(p.floats(from)) as f32;
    p.set_float(l.checked, length);
    if equal_or_unordered(length, p.float(l.nothing)) {
        for n in 0..3 {
            p.set_dword(out.wrapping_add(4 * n), 0);
        }
        return;
    }
    for n in 0..3 {
        let v = p.float(from.wrapping_add(4 * n)) as f64 / length as f64;
        p.set_float(out.wrapping_add(4 * n), v as f32);
    }
    for n in 0..3 {
        if p.dword(out.wrapping_add(4 * n)) == 0x8000_0000 {
            p.set_dword(out.wrapping_add(4 * n), 0);
        }
    }
}

/// `fld a; fmul b; fld c; fmul d; fsubp`: a b - c d, unrounded.
fn product_difference(a: f32, b: f32, c: f32, d: f32) -> f64 {
    a as f64 * b as f64 - c as f64 * d as f64
}

/// `scene::adjugate_matrix`'s entries: where each goes and the four floats (byte offsets)
/// of a b - c d.
pub const COFACTORS: [(u32, [u32; 4]); 9] = [
    (0x00, [0x14, 0x28, 0x24, 0x18]),
    (0x10, [0x20, 0x18, 0x10, 0x28]),
    (0x20, [0x10, 0x24, 0x20, 0x14]),
    (0x04, [0x24, 0x08, 0x04, 0x28]),
    (0x14, [0x00, 0x28, 0x20, 0x08]),
    (0x24, [0x20, 0x04, 0x00, 0x24]),
    (0x08, [0x04, 0x18, 0x14, 0x08]),
    (0x18, [0x10, 0x08, 0x00, 0x18]),
    (0x28, [0x00, 0x14, 0x10, 0x04]),
];

/// `scene::adjugate_matrix`: matrix `m`'s 3 x 3 cofactors, transposed, into `out` (each
/// stored before the next is worked out: `out` may be `m`), then its place (+0x30 ..
/// +0x38) negated (`fld; fchs; fstp`). +0xC, +0x1C, +0x2C, +0x3C are left.
pub fn adjugate_matrix(p: &mut impl Places, m: u32, out: u32) {
    for (to, [a, b, c, d]) in COFACTORS {
        let f = |o: u32| p.float(m.wrapping_add(o));
        let v = product_difference(f(a), f(b), f(c), f(d)) as f32;
        p.set_float(out.wrapping_add(to), v);
    }
    for o in [0x30, 0x34, 0x38] {
        let v = loaded(p.dword(m.wrapping_add(o))) ^ 0x8000_0000;
        p.set_dword(out.wrapping_add(o), v);
    }
}

/// `scene::cross_product`: ebx `b`, ecx `c`, edx `out`: out[0] c2 b1 - b2 c1, out[1] c0 b2
/// - b0 c2, out[2] c1 b0 - b1 c0, each stored before the next.
pub fn cross(p: &mut impl Places, b: u32, c: u32, out: u32) {
    for (n, (x, y)) in [(2, 1), (0, 2), (1, 0)].into_iter().enumerate() {
        let get = |v: u32, k: u32| p.float(v.wrapping_add(4 * k));
        let v = product_difference(get(c, x), get(b, y), get(b, x), get(c, y));
        p.set_float(out.wrapping_add(4 * n as u32), v as f32);
    }
}

/// `scene::cross_product3`: esi `a`, edi `b`, ebp `out`: x a.y b.z - a.z b.y, y a.z b.x -
/// a.x b.z, z a.x b.y - a.y b.x, each stored before the next.
pub fn cross3(p: &mut impl Places, a: u32, b: u32, out: u32) {
    for (n, (x, y)) in [(1, 2), (2, 0), (0, 1)].into_iter().enumerate() {
        let get = |v: u32, k: u32| p.float(v.wrapping_add(4 * k));
        let v = product_difference(get(a, x), get(b, y), get(a, y), get(b, x));
        p.set_float(out.wrapping_add(4 * n as u32), v as f32);
    }
}

/// `scene::square_matrix`: a turning's three rows made one long and at right angles again
/// in the work rows (the third from its third row, the first its second crossed with
/// that, the second the third crossed with the first), then sixteen dwords of the work
/// back to the matrix.
pub fn square(p: &mut impl Places, l: &Layout, at: u32) {
    let first = l.squared;
    let second = first.wrapping_add(0x10);
    let third = first.wrapping_add(0x20);
    unit_vector(p, l, at.wrapping_add(0x20), third);
    cross(p, at.wrapping_add(0x10), third, first);
    unit_vector(p, l, first, first);
    cross(p, third, first, second);
    copy_matrix(p, first, at);
}

/// `scene::invert_turn`: the turning at `from` undone into `out`: its nine floats copied
/// to the work (`fld; fstp`), their cofactors laid out the other way round (also kept in
/// the work), then squared.
pub fn invert_turn(p: &mut impl Places, l: &Layout, from: u32, out: u32) {
    let m: [f32; 9] = std::array::from_fn(|n| {
        f32::from_bits(loaded(p.dword(cell(from, n as u32 / 3, n as u32 % 3))))
    });
    p.set_floats(l.inverted, &m);
    let [a, b, c, d, e, f, g, h, i] = m;
    let plain = |w, x, y, z| product_difference(w, x, y, z) as f32;
    let negated = |w, x, y, z| -product_difference(w, x, y, z) as f32;
    let made = [
        plain(e, i, f, h),
        negated(b, i, c, h),
        plain(b, f, c, e),
        negated(d, i, f, g),
        plain(a, i, c, g),
        negated(a, f, c, d),
        plain(d, h, e, g),
        negated(a, h, b, g),
        plain(a, e, b, d),
    ];
    p.set_floats(l.inverted.wrapping_add(0x24), &made);
    for (n, v) in made.into_iter().enumerate() {
        p.set_float(cell(out, n as u32 / 3, n as u32 % 3), v);
    }
    square(p, l, out);
}

/// `scene::set_light_direction`: the turnings about x and y by the light's two angles
/// (`light` x then y, each times 256 and negated), the first times the second into
/// `turned`, that times the view's turning into `lit`, and (0, 0, 1) through that
/// (`scene::rotate_point`) the light's direction.
pub fn set_light_direction(p: &mut impl Places, l: &Layout, light: [u32; 2]) {
    set_table_turn(p, l, Axis::X, (light[0] << 8).wrapping_neg());
    set_table_turn(p, l, Axis::Y, (light[1] << 8).wrapping_neg());
    multiply(p, l.turn_x, l.turn_y, l.turned);
    multiply(p, l.turned, l.view_turn, l.lit);
    // The fourth float of each row is not read.
    let m: [f32; 15] = std::array::from_fn(|n| {
        if n % 4 == 3 {
            0.0
        } else {
            p.float(l.lit.wrapping_add(4 * n as u32))
        }
    });
    let ahead = vector::rotate_point(&m, [0.0, 0.0, 1.0]);
    p.set_floats(l.light_direction, &ahead.map(|v| v as f32));
}

/// An object's angles, place and size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    /// About y, then x (16 bits a turn).
    pub angles: [u32; 2],
    pub place: [u32; 3],
    /// Sixteenths.
    pub size: u32,
}

/// `fild size; fld1; fadd st, st` four times; `fdivp`: a whole number over 16, stored.
pub fn sixteenths(size: i32) -> f32 {
    (size as f64 / 16.0) as f32
}

/// `scene::make_object_matrix`: the light's direction (`light` as there); the turnings
/// about y and x by the object's angles negated; its place; y times x into `turned`,
/// scaled by the size (in sixteenths, left in the scratch); that with the place into
/// `object`.
pub fn make_object_matrix(p: &mut impl Places, l: &Layout, light: [u32; 2], o: Object) {
    set_light_direction(p, l, light);
    set_table_turn(p, l, Axis::Y, o.angles[0].wrapping_neg());
    set_table_turn(p, l, Axis::X, o.angles[1].wrapping_neg());
    set_place(p, l, o.place);
    multiply(p, l.turn_y, l.turn_x, l.turned);
    let factor = sixteenths(o.size as i32);
    p.set_float(l.checked, factor);
    for n in 0..16 {
        let at = l.turned.wrapping_add(4 * n);
        let v = p.float(at) as f64 * factor as f64;
        p.set_float(at, v as f32);
    }
    copy_with_place(p, l.turned, l.placed, l.object);
}

/// `scene::set_view_matrices`: the view's turning from the identity, the fine turnings
/// about y, x and z by the view's angles negated, y times x into `lit`, that times z into
/// the view's turning; the place negated, the placed matrix kept, times the turning into
/// the view; the turning undone into `view_back`. The place negated is returned (the
/// registers take two of it).
pub fn set_view_matrices(
    p: &mut impl Places,
    l: &Layout,
    angles: [u32; 3],
    place: [u32; 3],
) -> [u32; 3] {
    copy_matrix(p, l.identity, l.view_turn);
    set_fine_turn(p, l, Axis::Y, angles[0].wrapping_neg());
    set_fine_turn(p, l, Axis::X, angles[1].wrapping_neg());
    set_fine_turn(p, l, Axis::Z, angles[2].wrapping_neg());
    multiply(p, l.turn_y, l.turn_x, l.lit);
    multiply(p, l.lit, l.turn_z, l.view_turn);
    let place = place.map(u32::wrapping_neg);
    set_place(p, l, place);
    copy_matrix(p, l.placed, l.kept_place);
    multiply(p, l.placed, l.view_turn, l.view);
    invert_turn(p, l, l.view_turn, l.view_back);
    place
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::places::TestPlaces;

    fn layout() -> Layout {
        let m = |n: u32| n * 0x40;
        Layout {
            turn_x: m(0),
            turn_y: m(1),
            turn_z: m(2),
            placed: m(3),
            turned: m(4),
            lit: m(5),
            view_turn: m(6),
            object: m(7),
            identity: m(8),
            kept_place: m(9),
            view_back: m(10),
            view: m(11),
            turn_x_work: m(12),
            turn_y_work: m(12) + 8,
            turn_z_work: m(12) + 16,
            light_direction: m(12) + 24,
            checked: m(12) + 36,
            half_turns: m(12) + 40,
            turn: m(12) + 44,
            nothing: m(12) + 48,
            squared: m(13),
            inverted: m(14),
            sines: m(16),
            cosines: m(16) + 0x4000,
        }
    }

    fn places() -> TestPlaces {
        let l = layout();
        let mut p = TestPlaces(vec![0; (l.cosines as usize + 0x4000) / 4]);
        for n in 0..0x1000u32 {
            let a = n as f64 * std::f64::consts::TAU / 4096.0;
            p.set_float(l.sines + 4 * n, crate::math::portable::sin(a) as f32);
            p.set_float(l.cosines + 4 * n, crate::math::portable::cos(a) as f32);
        }
        let identity = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        for at in [l.identity, l.turn_x, l.turn_y, l.turn_z] {
            p.set_floats(at, &identity);
            p.set_float(at + 0x3C, 1.0);
        }
        p.set_float(l.half_turns, 65536.0);
        p.set_float(l.turn, (1.0 / std::f64::consts::TAU) as f32);
        p
    }

    #[test]
    fn adjugates() {
        let mut p = TestPlaces(vec![0; 32]);
        let m = [
            2.0, 0.0, 0.0, 7.0, 0.0, 3.0, 0.0, 7.0, 0.0, 0.0, 4.0, 7.0, 1.0, -2.0, 0.0, 7.0,
        ];
        p.set_floats(0, &m);
        adjugate_matrix(&mut p, 0, 0x40);
        assert_eq!(p.floats::<3>(0x40), [12.0, 0.0, 0.0]);
        assert_eq!(p.floats::<3>(0x50), [0.0, 8.0, 0.0]);
        assert_eq!(p.floats::<3>(0x60), [0.0, 0.0, 6.0]);
        assert_eq!(p.floats::<3>(0x70), [-1.0, 2.0, -0.0]);
        assert_eq!(p.dword(0x4C), 0);
        // In place: later entries read the earlier results.
        adjugate_matrix(&mut p, 0, 0);
        assert_eq!(p.float(0), 12.0);
        assert_eq!(p.float(0x14), 12.0 * 4.0);
    }

    #[test]
    fn multiplying_in_place() {
        let mut p = TestPlaces(vec![0; 48]);
        let a: Vec<f32> = (0..16).map(|n| n as f32 + 1.0).collect();
        p.set_floats(0, &a);
        p.set_floats(0x40, &a);
        // Apart, by a turning that is the identity with a place: the rows moved.
        let id = [1.0, 0.0, 0.0, 9.0, 0.0, 1.0, 0.0, 9.0, 0.0, 0.0, 1.0, 9.0];
        p.set_floats(0x80, &id);
        p.set_floats(0xB0, &[10.0, 20.0, 30.0, 9.0]);
        multiply(&mut p, 0, 0x80, 0x40);
        assert_eq!(p.floats::<4>(0x40), [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(p.floats::<4>(0x70), [23.0, 34.0, 45.0, 16.0]);
        // Over the second: column 0 of row 0 from the column as it was, row 1 from the
        // cell just written.
        let mut s = TestPlaces(vec![0; 32]);
        s.set_floats(0, &a);
        s.set_floats(0x40, &a);
        multiply(&mut s, 0, 0x40, 0x40);
        let first = 1.0 * 1.0 + (2.0 * 5.0 + 3.0 * 9.0);
        assert_eq!(s.float(0x40), first);
        assert_eq!(s.float(0x50), 5.0 * first + (6.0 * 5.0 + 7.0 * 9.0));
        // Column 1 of row 0 reads column 1, not yet written.
        assert_eq!(s.float(0x44), 1.0 * 2.0 + (2.0 * 6.0 + 3.0 * 10.0));
        // The fourth column is left.
        assert_eq!(s.float(0x4C), 4.0);
    }

    #[test]
    fn turns() {
        let l = layout();
        let mut p = places();
        let t = set_table_turn(&mut p, &l, Axis::Y, 0x4000);
        assert_eq!(f32::from_bits(t.sine), 1.0);
        assert_eq!(p.float(l.turn_y + 8), -1.0);
        assert_eq!(p.float(l.turn_y + 0x20), 1.0);
        assert_eq!(p.dword(l.turn_y_work), t.sine);
        let t = set_fine_turn(&mut p, &l, Axis::Z, 0x4000_0000);
        assert_eq!(p.dword(l.checked), 0x4000_0000);
        assert!((f32::from_bits(t.sine) - 1.0).abs() < 1e-6);
        assert_eq!(p.dword(l.turn_z + 0x10), t.negated());
    }

    #[test]
    fn unit_and_square() {
        let l = layout();
        let mut p = places();
        p.set_floats(0, &[3.0, 0.0, -4.0]);
        unit_vector(&mut p, &l, 0, 0x10);
        assert_eq!(p.floats::<3>(0x10), [0.6, 0.0, -0.8]);
        assert_eq!(p.float(l.checked), 5.0);
        p.set_floats(0, &[0.0, -0.0, 0.0]);
        unit_vector(&mut p, &l, 0, 0);
        assert_eq!(p.floats::<3>(0).map(f32::to_bits), [0; 3]);
        // A turning that has drifted, squared again.
        let m = [
            1.0, 0.01, 0.0, 0.0, -0.01, 1.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0,
        ];
        p.set_floats(0x40, &m);
        square(&mut p, &l, 0x40);
        assert_eq!(p.floats::<3>(0x60), [0.0, 0.0, 1.0]);
        let r: [f32; 3] = p.floats(0x50);
        assert!((vector::length(r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn inverting_and_the_view() {
        let l = layout();
        let mut p = places();
        let place = set_view_matrices(&mut p, &l, [0x1000_0000, 0x0800_0000, 0], [5, 6, 7]);
        assert_eq!(
            place,
            [
                5u32.wrapping_neg(),
                6u32.wrapping_neg(),
                7u32.wrapping_neg()
            ]
        );
        // The turning undone times the turning is near the identity.
        for row in 0..3 {
            for column in 0..3 {
                let r: [f32; 3] = p.floats(cell(l.view_back, row, 0));
                let c = [0, 1, 2].map(|k| p.float(cell(l.view_turn, k, column)));
                let want = if row == column { 1.0 } else { 0.0 };
                assert!((vector::dot(r, c) - want).abs() < 1e-5, "{row} {column}");
            }
        }
        assert_eq!(p.floats::<3>(cell(l.kept_place, 3, 0)), [-5.0, -6.0, -7.0]);
    }

    #[test]
    fn object_matrix() {
        let l = layout();
        let mut p = places();
        p.set_floats(
            l.view_turn,
            &[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        );
        let o = Object {
            angles: [0, 0],
            place: [1, 2, 3],
            size: 32,
        };
        make_object_matrix(&mut p, &l, [0, 0], o);
        assert_eq!(p.floats::<3>(l.light_direction), [0.0, 0.0, 1.0]);
        assert_eq!(p.float(l.checked), 2.0);
        assert_eq!(p.float(l.object), 2.0);
        assert_eq!(p.floats::<3>(cell(l.object, 3, 0)), [1.0, 2.0, 3.0]);
        assert_eq!(sixteenths(-8), -0.5);
    }

    #[test]
    fn crosses() {
        let mut p = TestPlaces(vec![0; 9]);
        p.set_floats(0, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        cross3(&mut p, 0, 12, 24);
        assert_eq!(p.floats::<3>(24), [0.0, 0.0, 1.0]);
        // ebx b, ecx c: b x c, as well.
        p.set_floats(24, &[0.0; 3]);
        cross(&mut p, 0, 12, 24);
        assert_eq!(p.floats::<3>(24), [0.0, 0.0, 1.0]);
        // Over its first input: the second part reads the first part written.
        p.set_floats(0, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        cross3(&mut p, 0, 12, 0);
        let x = 2.0 * 6.0 - 3.0 * 5.0;
        assert_eq!(
            p.floats::<3>(0),
            [
                x,
                3.0 * 4.0 - x * 6.0,
                x * 5.0 - (3.0 * 4.0 - x * 6.0) * 4.0
            ]
        );
    }
}
