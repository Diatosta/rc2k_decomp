//! The turns of the three kept vectors (`dword_76926E` .., `dword_7692C2` ..,
//! `dword_7692A6` ..) by two angles: `marks::turn_kept_vectors` (0x4C13CA) and its two
//! steps `marks::turn_kept_xz` (0x4C1411) and `marks::turn_kept_zy` (0x4C14D2).

use crate::marks::view_vectors::KEPT;
use crate::math::x87;
use crate::session::{Dword, Float};

/// The kept vectors, in the original's order.
fn vectors() -> [&'static [Dword; 3]; 3] {
    [
        &KEPT.kept_a.vector,
        &KEPT.kept_c.vector,
        &KEPT.kept_b.vector,
    ]
}

/// One turn: for each kept vector, its (`b`, `a`) = (`a` s + `b` c, `a` c - `b` s) by the
/// stored sine and cosine `sc` (`x87::turn_yz` with y the `b` field, z the `a`).
fn turn_kept(a: usize, b: usize, sc: &[Float; 2]) {
    let (s, c) = (sc[0].bits(), sc[1].bits());
    for v in vectors() {
        let (nb, na) = x87::turn_yz(v[b].get(), v[a].get(), s, c);
        v[b].set(nb);
        v[a].set(na);
    }
}

/// `marks::turn_kept_xz`: x and z by the second angle.
pub fn turn_kept_xz() {
    turn_kept(0, 2, &KEPT.second_sine_cosine)
}

/// `marks::turn_kept_zy`: z and y by the first angle.
pub fn turn_kept_zy() {
    turn_kept(2, 1, &KEPT.first_sine_cosine)
}

/// `marks::turn_kept_vectors`: the sines and cosines, then both turns.
pub fn turn_kept_vectors() {
    for (angle, to) in [
        (&KEPT.first_angle, &KEPT.first_sine_cosine),
        (&KEPT.second_angle, &KEPT.second_sine_cosine),
    ] {
        let (s, c) = x87::sine_and_cosine(angle.bits());
        to[0].set_bits(s);
        to[1].set_bits(c);
    }
    turn_kept_xz();
    turn_kept_zy();
}
