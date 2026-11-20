pub(super) fn keep_vectors(effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    let mut k = frame.kept_view.borrow_mut();
    let s = frame.view_sources.borrow();
    let from = [
        k.pushed[1],
        k.pushed[2],
        k.pushed[3],
        k.pushed[0],
        s.sources[1].vector[0],
        s.sources[1].vector[1],
        s.sources[1].vector[2],
        s.sources[1].factor,
        s.sources[0].vector[0],
        s.sources[0].vector[1],
        s.sources[0].vector[2],
        s.sources[0].factor,
    ];
    k.kept_a.vector = [from[0], from[1], from[2]];
    k.kept_a.factor = from[3];
    k.kept_c.vector = [from[4], from[5], from[6]];
    k.kept_c.factor = from[7];
    k.kept_b.vector = [from[8], from[9], from[10]];
    k.kept_b.factor = from[11];
}
