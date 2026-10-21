//! The turns of the three kept vectors (`dword_76926E` .., `dword_7692C2` ..,
//! `dword_7692A6` ..) by two angles: `marks::turn_kept_vectors` (0x4C13CA) and its two
//! steps `marks::turn_kept_xz` (0x4C1411) and `marks::turn_kept_zy` (0x4C14D2).

use crate::framework::{game_read, game_write, rva};
use crate::math::x87;

/// The kept vectors.
pub const VECTORS_RVA: [usize; 3] = [rva(0x76926E), rva(0x7692C2), rva(0x7692A6)];
/// The angles (`flt_76923E`, `flt_76924A`) and their sines and cosines (`flt_769236`,
/// `flt_76923A`; `flt_769242`, `flt_769246`).
pub const ANGLES_RVA: [usize; 2] = [rva(0x76923E), rva(0x76924A)];
pub const FIRST_RVA: [usize; 2] = [rva(0x769236), rva(0x76923A)];
pub const SECOND_RVA: [usize; 2] = [rva(0x769242), rva(0x769246)];

/// One turn: for each kept vector, its (`b`, `a`) = (`a` s + `b` c, `a` c - `b` s) by the
/// stored sine and cosine at `sc` (`x87::turn_yz` with y the `b` field, z the `a`).
pub unsafe fn turn_kept(a: usize, b: usize, sc: [usize; 2]) {
    unsafe {
        let (s, c): (u32, u32) = (game_read(sc[0]), game_read(sc[1]));
        for v in VECTORS_RVA {
            let (nb, na) = x87::turn_yz(game_read(v + b), game_read(v + a), s, c);
            game_write(v + b, nb);
            game_write(v + a, na);
        }
    }
}

/// `marks::turn_kept_xz`: x and z by the second angle.
pub unsafe fn turn_kept_xz() {
    unsafe { turn_kept(0, 8, SECOND_RVA) }
}

/// `marks::turn_kept_zy`: z and y by the first angle.
pub unsafe fn turn_kept_zy() {
    unsafe { turn_kept(8, 4, FIRST_RVA) }
}

/// `marks::turn_kept_vectors`: the sines and cosines, then both turns.
pub unsafe fn turn_kept_vectors() {
    unsafe {
        for (angle, to) in ANGLES_RVA.into_iter().zip([FIRST_RVA, SECOND_RVA]) {
            let (s, c) = x87::sine_and_cosine(game_read(angle));
            game_write(to[0], s);
            game_write(to[1], c);
        }
        turn_kept_xz();
        turn_kept_zy();
    }
}

// ---- the converted versions (`rc2k_core::marks::view`) ----

use rc2k_core::marks::view::{KeptVectors, SineCosine};

/// The kept vectors now.
pub unsafe fn kept_vectors() -> KeptVectors {
    KeptVectors(VECTORS_RVA.map(|v| unsafe { game_read::<[u32; 3]>(v) }.map(f32::from_bits)))
}

/// The kept vectors' `parts` written back, in that order (nothing overlaps).
unsafe fn put_kept_vectors(k: &KeptVectors, parts: &[usize]) {
    for (v, at) in k.0.iter().zip(VECTORS_RVA) {
        for &n in parts {
            unsafe { game_write(at + 4 * n, v[n].to_bits()) };
        }
    }
}

/// A stored sine and cosine.
unsafe fn stored(sc: [usize; 2]) -> SineCosine {
    unsafe {
        SineCosine {
            sine: f32::from_bits(game_read(sc[0])),
            cosine: f32::from_bits(game_read(sc[1])),
        }
    }
}

/// The converted `turn_kept_xz` (plain floats): the hook's.
// Converted: `rc2k_core::marks::view::KeptVectors::turn_xz`
pub unsafe fn turn_kept_xz_plain() {
    unsafe {
        let mut k = kept_vectors();
        k.turn_xz(stored(SECOND_RVA));
        put_kept_vectors(&k, &[2, 0]);
    }
}

/// The converted `turn_kept_zy` (plain floats): the hook's.
// Converted: `rc2k_core::marks::view::KeptVectors::turn_zy`
pub unsafe fn turn_kept_zy_plain() {
    unsafe {
        let mut k = kept_vectors();
        k.turn_zy(stored(FIRST_RVA));
        put_kept_vectors(&k, &[1, 2]);
    }
}

/// The converted `turn_kept_vectors` (plain floats): the hook's.
// Converted: `rc2k_core::marks::view::turn_kept_vectors`
pub unsafe fn turn_kept_vectors_plain() {
    unsafe {
        let angles = ANGLES_RVA.map(|a| f32::from_bits(game_read(a)));
        let mut k = kept_vectors();
        let turns = rc2k_core::marks::view::turn_kept_vectors(&mut k, angles);
        for (t, to) in turns.into_iter().zip([FIRST_RVA, SECOND_RVA]) {
            game_write(to[0], t.sine.to_bits());
            game_write(to[1], t.cosine.to_bits());
        }
        // Between them the two turns write every part.
        put_kept_vectors(&k, &[0, 1, 2]);
    }
}

/// The kept vectors' dwords.
pub fn vector_dwords() -> Vec<usize> {
    VECTORS_RVA
        .iter()
        .flat_map(|&v| [v, v + 4, v + 8])
        .collect()
}
