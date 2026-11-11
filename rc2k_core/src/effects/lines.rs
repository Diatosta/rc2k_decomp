//! The marks' measuring of lines between two objects' faces (from
//! `effects::measure_slot_lines` 0x4D1167, `effects::measure_kind_lines` 0x4D1225,
//! `effects::measure_lines` 0x4D0EB4, `effects::measure_starting_sums` 0x4D0DB5,
//! `effects::set_up_blocks` 0x4D05BD and `effects::copy_block_points` 0x4D0A82): each face's
//! points against the other object's same face's, their lengths summed into a settings
//! block's sums by the face's slot kind or kind; the blocks measured and their checks.
//!
//! As in `math::geometry`, what the x87 keeps on its stack is an `f64`: the same at the
//! game's double precision; at extended a sum may land one off at a rounding edge.

use crate::math::geometry::fistp64;

/// A block's slots (+0xD0 their kinds, +0xF0 their sums) and kinds (+0x130 the kinds,
/// +0x1B8 their sums, +0x240 their point counts).
pub const SLOTS: u32 = 8;
pub const KINDS: u32 = 0x11;

/// `fild x; fmul st, st; fild y; fmul st, st; fild z; fmul st, st; faddp; faddp; fsqrt;
/// fiadd sum; fistp`: the sum plus the length of (x, y, z).
pub fn length_plus(d: [i32; 3], sum: i32) -> i32 {
    let [x, y, z] = d.map(|v| v as f64 * v as f64);
    fistp64((x + (y + z)).sqrt() + sum as f64)
}

/// The differences of two points (x, y, z, wrapping), what the original stores.
pub fn difference(p: [u32; 3], q: [u32; 3]) -> [u32; 3] {
    std::array::from_fn(|n| p[n].wrapping_sub(q[n]))
}

/// Whether a slot (its kind's dword) takes a face of slot kind `face_kind` (the face's
/// +7): the dword's low byte.
pub fn slot_takes(slot: u32, face_kind: u8) -> bool {
    slot as u8 == face_kind
}

/// Whether a kind (its dword) takes a face of kind `face_kind` (+6) with line kind
/// `lines` (+0xB): the dword's low byte, and the line kind 1.
pub fn kind_takes(kind: u32, face_kind: u8, lines: u8) -> bool {
    kind as u8 == face_kind && lines == 1
}

/// The settings' blocks (`g_marksSettings` + 0x29C, 0x4C8 apart; +0x5C their count).
pub const BLOCKS_AT: u32 = 0x29C;
pub const BLOCK_LEN: u32 = 0x4C8;
pub const MOST_BLOCKS: u32 = 8;
/// The objects a block may name.
pub const OBJECTS: u32 = 0x100;

/// A block's place from the settings (wrapping, as the original's sums).
pub fn block_at(settings: u32, k: u32) -> u32 {
    settings
        .wrapping_add(BLOCK_LEN.wrapping_mul(k))
        .wrapping_add(BLOCKS_AT)
}

/// Refuses an object past 0xFF (the tables' end).
pub fn check_object(object: u32) -> Result<(), crate::Invalid> {
    if object >= OBJECTS {
        return Err(crate::Invalid("a block's object past 0xFF"));
    }
    Ok(())
}

/// A block to measure: its number, its two objects' first faces, its first object's
/// face count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measured {
    pub block: u32,
    pub first: [u32; 2],
    pub faces: u32,
}

/// What the game's tables tell the planning, asked in the original's order.
pub trait Blocks {
    /// Block k's +0xC (marked to be measured).
    fn marked(&self, k: u32) -> bool;
    /// Block k's two objects (+0, +4).
    fn objects(&self, k: u32) -> [u32; 2];
    /// An object's face count and first face.
    fn face_count(&self, object: u32) -> u32;
    fn first_face(&self, object: u32) -> u32;
}

/// The blocks `measure_lines` measures, the last first: every one (`every`) or the marked
/// ones. Refuses more than 8 blocks (signed), an object past 0xFF and faces past
/// `faces_total` (`g_faces`).
pub fn measured(
    count: u32,
    every: bool,
    b: &dyn Blocks,
    faces_total: u32,
) -> Result<Vec<Measured>, crate::Invalid> {
    let count = count as i32;
    if count > MOST_BLOCKS as i32 {
        return Err(crate::Invalid("more than 8 marks' blocks"));
    }
    let mut out = Vec::new();
    for k in (0..count.max(0) as u32).rev() {
        if !every && !b.marked(k) {
            continue;
        }
        let [a, c] = b.objects(k);
        if a >= OBJECTS || c >= OBJECTS {
            return Err(crate::Invalid("a block's object past 0xFF"));
        }
        let faces = b.face_count(a);
        let first = [b.first_face(a), b.first_face(c)];
        let n = (faces as i32).max(0) as u32;
        if first
            .iter()
            .any(|&f| f.checked_add(n).is_none_or(|end| end > faces_total))
        {
            return Err(crate::Invalid("a block's faces past g_faces"));
        }
        out.push(Measured {
            block: k,
            first,
            faces,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines() {
        assert_eq!(length_plus([3, 4, 0], 10), 15);
        assert_eq!(length_plus([1, 1, 0], 0), 1);
        assert_eq!(length_plus([0x10000, 0x10000, 0x10000], -5), 113512 - 5);
        assert_eq!(difference([1, 2, 3], [2, 2, 0]), [u32::MAX, 0, 3]);
        assert!(slot_takes(0x103, 3) && !slot_takes(4, 3));
        assert!(kind_takes(0x1C, 0x1C, 1) && !kind_takes(0x1C, 0x1C, 2));
    }

    struct Test;

    impl Blocks for Test {
        fn marked(&self, k: u32) -> bool {
            k != 1
        }
        fn objects(&self, k: u32) -> [u32; 2] {
            if k == 7 { [0x100, 0] } else { [k, k + 0x10] }
        }
        fn face_count(&self, object: u32) -> u32 {
            object * 2
        }
        fn first_face(&self, object: u32) -> u32 {
            object * 0x10
        }
    }

    #[test]
    fn blocks() {
        assert_eq!(block_at(0x1000, 2), 0x1000 + 0x29C + 2 * 0x4C8);
        assert!(check_object(0xFF).is_ok() && check_object(0x100).is_err());
        assert!(measured(9, true, &Test, 0x1000).is_err());
        assert!(measured(8, true, &Test, 0x1000).is_err());
        let m = measured(3, false, &Test, 0x1000).unwrap();
        assert_eq!(m.iter().map(|m| m.block).collect::<Vec<_>>(), [2, 0]);
        assert_eq!(
            m[0],
            Measured {
                block: 2,
                first: [0x20, 0x120],
                faces: 4
            }
        );
        assert!(measured(3, true, &Test, 0x123).is_err());
        assert_eq!(measured(0xFFFF_FFFF, true, &Test, 0), Ok(vec![]));
    }
}
