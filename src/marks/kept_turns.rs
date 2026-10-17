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

/// The kept vectors' dwords.
pub fn vector_dwords() -> Vec<usize> {
    VECTORS_RVA
        .iter()
        .flat_map(|&v| [v, v + 4, v + 8])
        .collect()
}
