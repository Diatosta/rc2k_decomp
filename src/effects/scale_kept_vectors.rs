//! `effects::scale_kept_vectors` (0x4C15F4): three kept vectors scaled by their kept factors
//! (`effects::scale_vectors_by`' pattern): `flt_76927A` .. = `dword_76926E` .. by
//! `dword_76926A`, `flt_7692CE` .. = `dword_7692C2` .. by `dword_7692BE`, `flt_7692B2` ..
//! = `dword_7692A6` .. by `dword_7692A2`.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::scale_vectors_by::scale_triples;

/// The Rust version (see the header).
pub fn scale_kept_vectors(effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    scale_triples(&[
        &frame.kept_view.kept_a,
        &frame.kept_view.kept_c,
        &frame.kept_view.kept_b,
    ]);
}
