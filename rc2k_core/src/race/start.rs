//! The race's start and its panel (converted 2026-10-21 from `race::countdown_lights`
//! 0x45B008 and `race::car_label` 0x45B1B7).

/// The start lights of one view, from the milliseconds since the countdown began.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lights {
    /// `dword_6D1810`: the hundreds, at most 5.
    pub shown: u32,
    /// The lights lit: the hundreds, -1 from 6.
    pub lit: u32,
    pub fade: u32,
    pub size: u32,
    pub shift: u32,
    /// On a whole hundred (with lights lit).
    pub whole: bool,
}

/// The time held to 1500 ms; the hundreds and the rest.
pub fn steps(since: u32) -> (u32, u32) {
    let t = since.min(0x5DC);
    (t / 100, t % 100)
}

/// The lights at `since` (None: negative, nothing more is done): the rest x 2 taken from
/// 0x100 for fade and size (none on a whole hundred); with five shown, fade 0x80 x that
/// (held 0 .. 0x100) / 0x30 and shift half what that took away.
pub fn lights(since: u32) -> Option<Lights> {
    if (since as i32) < 0 {
        return None;
    }
    let (q, rem) = steps(since);
    let shown = q.min(5);
    let lit = if q < 6 { q } else { u32::MAX };
    let whole = rem == 0 && lit != u32::MAX;
    let rest = 0x100 - if whole { 0 } else { rem * 2 };
    let (fade, shift) = if shown == 5 {
        let v = (rest as i32).clamp(0, 0x100);
        let fade = (v as i64 * 0x80 / 0x30) as i32;
        (fade as u32, (v.wrapping_sub(fade) >> 1) as u32)
    } else {
        (rest, 0)
    };
    Some(Lights {
        shown,
        lit,
        fade,
        size: rest,
        shift,
        whole,
    })
}

/// " CAR nn" (each digit + '0' as a byte), or seven spaces for none.
pub fn car_label(number: u32, none: bool) -> [u8; 7] {
    if none {
        return [b' '; 7];
    }
    [
        b' ',
        b'C',
        b'A',
        b'R',
        b' ',
        ((number / 10) as u8).wrapping_add(b'0'),
        ((number % 10) as u8).wrapping_add(b'0'),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start() {
        assert_eq!(lights(u32::MAX), None);
        let l = lights(300).unwrap();
        assert_eq!(
            (l.shown, l.lit, l.whole, l.fade, l.size),
            (3, 3, true, 0x100, 0x100)
        );
        let l = lights(550).unwrap();
        assert_eq!(
            (l.shown, l.fade, l.shift),
            (
                5,
                0x9C * 0x80 / 0x30,
                ((0x9C - 0x9C * 0x80 / 0x30) >> 1) as u32
            )
        );
        assert_eq!(lights(2000).unwrap().lit, u32::MAX);
        assert_eq!(&car_label(42, false), b" CAR 42");
    }
}
