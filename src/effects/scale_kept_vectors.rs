//! `effects::scale_kept_vectors`: three kept vectors scaled by their kept factors
//! (`effects::scale_vectors_by`' pattern): `KeptView.kept_a.scaled[0]` .. =
//! `KeptView.kept_a.vector[0]` .. by `KeptView.kept_a.factor`, `KeptView.kept_c.scaled[0]` .. =
//! `KeptView.kept_c.vector[0]` .. by `KeptView.kept_c.factor`, `KeptView.kept_b.scaled[0]` .. =
//! `KeptView.kept_b.vector[0]` .. by `KeptView.kept_b.factor`.

use crate::effects::scale_vectors_by::scale_triples;

pub(super) fn scale_kept_vectors(effects: &crate::effects::owner::Effects) {
    let mut kept = effects.frame.kept_view.borrow_mut();
    let kept = &mut *kept;
    scale_triples(&mut [&mut kept.kept_a, &mut kept.kept_c, &mut kept.kept_b]);
}
