//! The turns of the three kept vectors (`KeptView.kept_a.vector[0]` .., `KeptView.kept_c.vector[0]`
//! .., `KeptView.kept_b.vector[0]` ..) by two angles: `effects::turn_kept_vectors` and its two
//! steps `effects::turn_kept_xz` and `effects::turn_kept_zy`.

use crate::math::x87;
use crate::memory::session::PlainFloat;

/// The kept vectors.
fn vectors(effects: &crate::effects::owner::Effects) -> [&[PlainFloat; 3]; 3] {
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
    let (s, c) = (sc[0].get(), sc[1].get());
    for v in vectors(effects) {
        let (nb, na) = x87::turn_yz(v[b].get(), v[a].get(), s, c);
        v[b].set(nb);
        v[a].set(na);
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
