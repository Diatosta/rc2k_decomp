//! The turns of the three kept vectors (`dword_76926E` .., `dword_7692C2` ..,
//! `dword_7692A6` ..) by two angles: `effects::turn_kept_vectors` (0x4C13CA) and its two
//! steps `effects::turn_kept_xz` (0x4C1411) and `effects::turn_kept_zy` (0x4C14D2).

use crate::math::x87;
use crate::session::{PlainDword, PlainFloat};

/// The kept vectors, in the original's order.
fn vectors(effects: &crate::effects::owner::Effects) -> [&[PlainDword; 3]; 3] {
    let frame = &effects.frame;
    [
        &frame.kept_view.kept_a.vector,
        &frame.kept_view.kept_c.vector,
        &frame.kept_view.kept_b.vector,
    ]
}

/// One turn: for each kept vector, its (`b`, `a`) = (`a` s + `b` c, `a` c - `b` s) by the
/// stored sine and cosine `sc` (`x87::turn_yz` with y the `b` field, z the `a`).
fn turn_kept(a: usize, b: usize, sc: &[PlainFloat; 2], effects: &crate::effects::owner::Effects) {
    let (s, c) = (sc[0].bits(), sc[1].bits());
    for v in vectors(effects) {
        let (nb, na) = x87::turn_yz(v[b].get(), v[a].get(), s, c);
        v[b].set_bits(nb.to_bits());
        v[a].set_bits(na.to_bits());
    }
}

/// `effects::turn_kept_xz`: x and z by the second angle.
pub fn turn_kept_xz(effects: &crate::effects::owner::Effects) {
    turn_kept(0, 2, &effects.frame.kept_view.second_sine_cosine, effects);
}

/// `effects::turn_kept_zy`: z and y by the first angle.
pub fn turn_kept_zy(effects: &crate::effects::owner::Effects) {
    turn_kept(2, 1, &effects.frame.kept_view.first_sine_cosine, effects);
}

/// `effects::turn_kept_vectors`: the sines and cosines, then both turns.
pub fn turn_kept_vectors(effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    for (angle, to) in [
        (
            &frame.kept_view.first_angle,
            &frame.kept_view.first_sine_cosine,
        ),
        (
            &frame.kept_view.second_angle,
            &frame.kept_view.second_sine_cosine,
        ),
    ] {
        let (s, c) = x87::sine_and_cosine(angle);
        to[0].set(s);
        to[1].set(c);
    }
    turn_kept_xz(effects);
    turn_kept_zy(effects);
}
