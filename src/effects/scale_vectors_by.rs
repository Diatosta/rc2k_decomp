//! `effects::scale_vectors_by`: two vectors scaled: `ViewSources.sources[0].scaled[0]` .. =
//! `ViewSources.sources[0].vector[0]` .. by `ViewSources.sources[0].factor`,
//! `ViewSources.sources[1].scaled[0]` .. = `ViewSources.sources[1].vector[0]` .. by
//! `ViewSources.sources[1].factor` (`fld; fmul`, three on the stack, stored last first).

use crate::effects::view_vectors::VectorSource;
use crate::math::x87;

pub(super) fn scale_vectors_by(effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    scale_triples(&[
        &frame.view_sources.sources[0],
        &frame.view_sources.sources[1],
    ]);
}

/// Each record: its vector's three floats by its factor (`fld; fmul`) into its scaled vector.
pub(super) fn scale_triples(records: &[&VectorSource]) {
    for r in records {
        let k = r.factor.get();
        for n in 0..3 {
            r.scaled[n].set(x87::mul(r.vector[n].get(), k));
        }
    }
}
