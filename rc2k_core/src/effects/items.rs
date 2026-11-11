//! The squares' and windscreen drops' items (from `effects::spawn_windscreen_flake`
//! 0x4C0120, `pick_windscreen_flake` 0x4C0623, `spawn_windscreen_drop` 0x4BC22B and `pick_windscreen_drop`
//! 0x4BC6F1): 0x800 records each, a cursor (a byte offset into them) where the next is
//! looked for, made live at a place by lot.

use crate::effects::particles::next_random;

/// The items of each kind.
pub const ITEMS: u32 = 0x800;
/// The bytes of a square's and of a windscreen drop's item.
pub const WINDSCREEN_FLAKE_LEN: u32 = 0x3C;
pub const WINDSCREEN_DROP_LEN: u32 = 0x30;
/// The items looked at for a free one.
pub const WINDOW: u32 = 32;

/// Where a free item was found (its offset), if one was, and the cursor after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub at: Option<u32>,
    pub cursor: u32,
}

/// The first free item (`is_free` of its offset: its +0 is 0) of the `WINDOW` from the
/// cursor on, each step `len` on and back to 0 at the end (only there: a cursor that is
/// not a multiple of `len` goes past the end, as in the original); the cursor the item
/// after the one found, or `WINDOW` items on.
pub fn find_free(cursor: u32, len: u32, mut is_free: impl FnMut(u32) -> bool) -> Found {
    let end = ITEMS * len;
    let step = |at: u32| {
        let next = at.wrapping_add(len);
        if next == end { 0 } else { next }
    };
    let mut at = cursor;
    for _ in 0..WINDOW {
        if is_free(at) {
            return Found {
                at: Some(at),
                cursor: step(at),
            };
        }
        at = step(at);
    }
    Found {
        at: None,
        cursor: at,
    }
}

/// A place and kind by lot: x (bits 0 .. 10), z (11 .. 20) and the kind (3 bits).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pick {
    pub random: u32,
    pub x: u32,
    pub z: u32,
    pub kind: u32,
}

/// `effects::pick_windscreen_flake`'s: the squares' own random (`dword_8BC256`, times 0xC5 plus
/// 0x5D), the kind its bits 21 .. 23.
pub fn pick_windscreen_flake(random: u32) -> Pick {
    let s = random.wrapping_mul(0xC5).wrapping_add(0x5D);
    Pick {
        random: s,
        x: s & 0x7FF,
        z: (s >> 11) & 0x3FF,
        kind: (s >> 21) & 7,
    }
}

/// `effects::pick_windscreen_drop`'s: the marks' random (`g_marksSeed`), the kind its bits 22 ..
/// 24.
pub fn pick_windscreen_drop(seed: u32) -> Pick {
    let s = next_random(seed);
    Pick {
        random: s,
        x: s & 0x7FF,
        z: s.rotate_right(11) & 0x3FF,
        kind: s.rotate_right(22) & 7,
    }
}

/// A new square's floats: its place (`fild`), its size (the kind & 7 times `size[0]` plus
/// `size[1]`, stored).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewWindscreenFlake {
    pub x: f32,
    pub z: f32,
    pub size: f32,
    /// The kind & 7 (ecx after).
    pub kind: u32,
}

pub fn new_windscreen_flake(x: u32, z: u32, kind: u32, size: [f32; 2]) -> NewWindscreenFlake {
    let k = kind & 7;
    NewWindscreenFlake {
        x: x as i32 as f32,
        z: z as i32 as f32,
        size: (k as f64 * size[0] as f64 + size[1] as f64) as f32,
        kind: k,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows() {
        // The third item from the cursor free.
        let f = find_free(0x3C * 10, 0x3C, |at| at == 0x3C * 12);
        assert_eq!(
            f,
            Found {
                at: Some(0x3C * 12),
                cursor: 0x3C * 13
            }
        );
        // Wrapping at the end.
        let f = find_free(0x3C * (ITEMS - 1), 0x3C, |at| at == 0);
        assert_eq!(
            f,
            Found {
                at: Some(0),
                cursor: 0x3C
            }
        );
        // None free: 32 on.
        let f = find_free(0, 0x30, |_| false);
        assert_eq!(
            f,
            Found {
                at: None,
                cursor: 32 * 0x30
            }
        );
    }

    #[test]
    fn picks_and_squares() {
        let p = pick_windscreen_flake(0);
        assert_eq!((p.random, p.x, p.z, p.kind), (0x5D, 0x5D, 0, 0));
        let p = pick_windscreen_drop(0);
        assert_eq!((p.random, p.x), (0x47, 0x47));
        let s = new_windscreen_flake(5, u32::MAX, 0x1B, [1.5, 0.25]);
        assert_eq!((s.x, s.z, s.size, s.kind), (5.0, -1.0, 4.75, 3));
    }
}
