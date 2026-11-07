//! `effects::scale_vectors_by` (0x4D16A3): two vectors scaled: `flt_764FA0` .. =
//! `dword_764F94` .. by `dword_764F90`, `flt_764FBC` .. = `dword_764FB0` .. by
//! `dword_764FAC` (`fld; fmul`, three on the stack, stored last first).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4D14B4`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::view_vectors::VectorSource;
use crate::math::x87;

/// The Rust version (see the header).
pub fn scale_vectors_by(effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    scale_triples(&[
        &frame.view_sources.sources[0],
        &frame.view_sources.sources[1],
    ]);
}

/// Each record: its vector's three floats by its factor (`fld; fmul`) into its scaled vector.
pub fn scale_triples(records: &[&VectorSource]) {
    for r in records {
        let k = r.factor.get();
        for n in 0..3 {
            r.scaled[n].set_bits(x87::mul(r.vector[n].get(), k));
        }
    }
}
