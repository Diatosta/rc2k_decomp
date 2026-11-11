//! The light on the work car from the face under it (`physics::car_lighting`): the face's four
//! corner lights blended by the car's shares across the face (16.16), scaled by the texture's shade
//! bytes when it has them; a change held to `MOST_CHANGE` a step; colours added byte by byte with
//! saturation. Each step also returns the leftover values of the original's last products, which
//! later code of the original still reads.

/// The shares' whole (16.16 fixed point).
pub const WHOLE: u32 = 0x10000;
/// The most a held light changes in a step.
pub const MOST_CHANGE: u32 = 10;

/// The shares taken from the whole on the face's second half (the face's half flag 1).
pub fn half_shares(half: u32, s: u32, t: u32) -> (u32, u32) {
    if half == 1 {
        (WHOLE.wrapping_sub(s), WHOLE.wrapping_sub(t))
    } else {
        (s, t)
    }
}

/// lo + ((hi - lo) x s >> 16) (`imul`, `shrd` 16); and the product's high half.
pub fn blend(lo: u32, hi: u32, s: u32) -> (u32, u32) {
    let product = (hi.wrapping_sub(lo) as i32 as i64).wrapping_mul(s as i32 as i64);
    (
        (((product as u64) >> 16) as u32).wrapping_add(lo),
        (product >> 32) as u32,
    )
}

/// The four corners blended: corners 0 - 1 and 2 - 3 by `s`, then the two by `t`; the
/// two halves (kept for the ground store), the result and the product's high half.
pub fn corners(c: [u32; 4], s: u32, t: u32) -> ([u32; 2], u32, u32) {
    let (a, _) = blend(c[0], c[1], s);
    let (b, _) = blend(c[2], c[3], s);
    let (v, high) = blend(a, b, t);
    ([a, b], v, high)
}

/// A corner's light scaled by its shade byte (the face's +0x22 + k): ((byte >> 1) ^ 0x7F)
/// + 0x80 times the light (`mul`), >> 8; and the product's high half.
pub fn shaded(light: u32, shade: u8) -> (u32, u32) {
    let v = ((shade as u32 >> 1) ^ 0x7F) + 0x80;
    let product = v as u64 * light as u64;
    ((product as u32) >> 8, (product >> 32) as u32)
}

/// A held change's end: the light.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Held {
    pub light: u32,
}

/// The light's change from `old` held (with the car's +0xFB4 set): kept at `old` with
/// `keep` (`scene_switch`); else a change of `MOST_CHANGE` or more (by its size, signed)
/// made `MOST_CHANGE` that way.
pub fn hold_change(light: u32, old: u32, keep: bool) -> Held {
    if keep {
        return Held { light: old };
    }
    let gap = light.wrapping_sub(old);
    let size = if (gap as i32) < 0 {
        gap.wrapping_neg()
    } else {
        gap
    };
    if size < MOST_CHANGE {
        return Held { light };
    }
    let step = if (gap as i32) < 0 {
        MOST_CHANGE.wrapping_neg()
    } else {
        MOST_CHANGE
    };
    let v = step.wrapping_add(old);
    Held { light: v }
}

/// The light and colour tables' size: an index past it is refused.
pub const TABLE: u32 = 0x100;

/// A colour added to the light byte by byte with saturation: four bytes with MMX
/// (`g_hasMmx`), else the low three (the colour's top byte kept).
pub fn add_colour(colour: u32, light: u32, mmx: bool) -> u32 {
    let sat = |k: u32| (((colour >> k) & 0xFF) + ((light >> k) & 0xFF)).min(0xFF) << k;
    let top = if mmx { sat(24) } else { colour & 0xFF00_0000 };
    sat(0) | sat(8) | sat(16) | top
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blends() {
        assert_eq!(half_shares(1, 0x4000, 0), (0xC000, 0x10000));
        assert_eq!(half_shares(0, 0x4000, 0), (0x4000, 0));
        assert_eq!(blend(100, 200, 0x8000), (150, 0));
        assert_eq!(blend(200, 100, 0x8000), (150, u32::MAX));
        let (halves, v, _) = corners([0, 100, 100, 200], 0x8000, 0x8000);
        assert_eq!((halves, v), ([50, 150], 100));
        assert_eq!(shaded(0x100, 0), (0xFF, 0));
        assert_eq!(shaded(0x100, 0xFF), (0x80, 0));
    }

    #[test]
    fn holds() {
        assert_eq!(hold_change(50, 20, true).light, 20);
        assert_eq!(hold_change(25, 20, false), Held { light: 25 });
        assert_eq!(hold_change(50, 20, false).light, 30);
        assert_eq!(hold_change(0, 20, false).light, 10);
    }

    #[test]
    fn colours() {
        assert_eq!(add_colour(0x80F0_1010, 0x8020_2020, true), 0xFFFF_3030);
        assert_eq!(add_colour(0x80F0_1010, 0x8020_2020, false), 0x80FF_3030);
    }
}
