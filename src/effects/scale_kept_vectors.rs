//! `effects::scale_kept_vectors`: three kept vectors scaled by their kept factors
//! (`effects::scale_vectors_by`' pattern): `KeptView.kept_a.scaled[0]` .. =
//! `KeptView.kept_a.vector[0]` .. by `KeptView.kept_a.factor`, `KeptView.kept_c.scaled[0]` .. =
//! `KeptView.kept_c.vector[0]` .. by `KeptView.kept_c.factor`, `KeptView.kept_b.scaled[0]` .. =
//! `KeptView.kept_b.vector[0]` .. by `KeptView.kept_b.factor`.

use crate::effects::scale_vectors_by::scale_triples;

pub fn scale_kept_vectors(effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    scale_triples(&[
        &frame.kept_view.kept_a,
        &frame.kept_view.kept_c,
        &frame.kept_view.kept_b,
    ]);
}
