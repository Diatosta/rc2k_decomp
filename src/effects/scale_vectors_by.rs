//! `effects::scale_vectors_by`: two vectors scaled: `ViewSources.sources[0].scaled[0]` .. =
//! `ViewSources.sources[0].vector[0]` .. by `ViewSources.sources[0].factor`,
//! `ViewSources.sources[1].scaled[0]` .. = `ViewSources.sources[1].vector[0]` .. by
//! `ViewSources.sources[1].factor` (`fld; fmul`, three on the stack, stored last first).

use crate::effects::view_vectors_store::VectorSource;
use crate::math::x87;

pub(super) fn scale_vectors_by(effects: &crate::effects::owner::Effects) {
    let mut views = effects.frame.view_sources.borrow_mut();
    let [first, second] = &mut views.sources;
    scale_triples(&mut [first, second]);
}

/// Each record: its vector's three floats by its factor (`fld; fmul`) into its scaled vector.
pub(super) fn scale_triples(records: &mut [&mut VectorSource]) {
    for r in records {
        let k = r.factor;
        for n in 0..3 {
            r.scaled[n] = x87::mul(r.vector[n], k);
        }
    }
}
