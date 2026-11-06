//! `effects::keep_vectors` (0x4C1339): twelve dwords kept (`push`/`pop`): `dword_769252` ..
//! and `dword_76924E` into `dword_76926E` .. and `dword_76926A`; `dword_764FB0` .. and
//! `dword_764FAC` into `dword_7692C2` .. and `dword_7692BE`; `dword_764F94` .. and
//! `dword_764F90` into `dword_7692A6` .. and `dword_7692A2`.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`.

/// The Rust version (see the header).
pub fn keep_vectors() {
    let (k, s) = (
        &crate::effects::owner::effects().frame.kept_view,
        &crate::effects::owner::effects().frame.view_sources.sources,
    );
    for (from, to) in [
        // (from, to), in the original's order
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
