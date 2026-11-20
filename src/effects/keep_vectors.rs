//! `effects::keep_vectors`: twelve dwords kept: `KeptView.pushed[1]` .. and `KeptView.pushed[0]`
//! into `KeptView.kept_a.vector[0]` .. and `KeptView.kept_a.factor`;
//! `ViewSources.sources[1].vector[0]` .. and `ViewSources.sources[1].factor` into
//! `KeptView.kept_c.vector[0]` .. and `KeptView.kept_c.factor`; `ViewSources.sources[0].vector[0]`
//! .. and `ViewSources.sources[0].factor` into `KeptView.kept_b.vector[0]` .. and
//! `KeptView.kept_b.factor`.

pub(super) fn keep_vectors(effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    let (k, s) = (&frame.kept_view, &frame.view_sources.sources);
    for (from, to) in [
        // (from, to)
        (&k.pushed[1], &k.kept_a.vector[0]),
        (&k.pushed[2], &k.kept_a.vector[1]),
        (&k.pushed[3], &k.kept_a.vector[2]),
        (&k.pushed[0], &k.kept_a.factor),
        (&s[1].vector[0], &k.kept_c.vector[0]),
        (&s[1].vector[1], &k.kept_c.vector[1]),
        (&s[1].vector[2], &k.kept_c.vector[2]),
        (&s[1].factor, &k.kept_c.factor),
        (&s[0].vector[0], &k.kept_b.vector[0]),
        (&s[0].vector[1], &k.kept_b.vector[1]),
        (&s[0].vector[2], &k.kept_b.vector[2]),
        (&s[0].factor, &k.kept_b.factor),
    ] {
        to.set(from.get());
    }
}
