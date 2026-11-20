//! The turns of the three kept vectors (`KeptView.kept_a.vector[0]` .., `KeptView.kept_c.vector[0]`
//! .., `KeptView.kept_b.vector[0]` ..) by two angles: `effects::turn_kept_vectors` and its two
//! steps `effects::turn_kept_xz` and `effects::turn_kept_zy`.

use crate::effects::view_vectors_store::KeptView;
use crate::math::x87;

/// One turn: for each kept vector, its (`b`, `a`) = (`a` s + `b` c, `a` c - `b` s) by the
/// stored sine and cosine `sc` (`x87::turn_yz` with y the `b` field, z the `a`).
fn turn_kept(a: usize, b: usize, sc: [f32; 2], kept: &mut KeptView) {
    let (s, c) = (sc[0], sc[1]);
    for v in [
        &mut kept.kept_a.vector,
        &mut kept.kept_c.vector,
        &mut kept.kept_b.vector,
    ] {
        let (nb, na) = x87::turn_yz(v[b], v[a], s, c);
        v[b] = nb;
        v[a] = na;
    }
}

/// `effects::turn_kept_xz`: x and z by the second angle.
pub fn turn_kept_xz(effects: &crate::effects::owner::Effects) {
    let mut kept = effects.frame.kept_view.borrow_mut();
    let sc = kept.second_sine_cosine;
    turn_kept(0, 2, sc, &mut kept);
}

/// `effects::turn_kept_zy`: z and y by the first angle.
pub fn turn_kept_zy(effects: &crate::effects::owner::Effects) {
    let mut kept = effects.frame.kept_view.borrow_mut();
    let sc = kept.first_sine_cosine;
    turn_kept(2, 1, sc, &mut kept);
}

/// `effects::turn_kept_vectors`: the sines and cosines, then both turns.
pub fn turn_kept_vectors(effects: &crate::effects::owner::Effects) {
    {
        let mut kept = effects.frame.kept_view.borrow_mut();
        let (s, c) = x87::sine_and_cosine(kept.first_angle);
        kept.first_sine_cosine = [s, c];
        let (s, c) = x87::sine_and_cosine(kept.second_angle);
        kept.second_sine_cosine = [s, c];
    }
    turn_kept_xz(effects);
    turn_kept_zy(effects);
}
