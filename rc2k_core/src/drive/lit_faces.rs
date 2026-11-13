//! The stage's moving faces: each listed face's bytes stepped by its object's kind, at a pace by
//! its distance from the lead car; a face that is done leaves the list (the list's last takes its
//! place). The listed faces are the ones the car has touched (`physics::touch_face`): roadside
//! objects that blink, rise or are knocked away for a while. Kind 4's move (a point turned and
//! the ground looked for) stays with the caller.

/// The object whose faces only count down (+0x11).
pub const COUNTDOWN_OBJECT: u8 = 0x3B;

/// The face record's bytes the steps change (+8, +9, +0x11, +0x12, +0x13).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaceBytes {
    pub b8: u8,
    pub b9: u8,
    pub b11: u8,
    pub b12: u8,
    pub b13: u8,
}

/// Object 0x3B's step: +0x11 less one; true when it reaches 0 (the face leaves).
pub fn count_down(f: &mut FaceBytes) -> bool {
    f.b11 = f.b11.wrapping_sub(1);
    f.b11 == 0
}

/// The distance past which the pace is 1.
pub const FAR: u32 = 0x2_0000;

/// The pace from the distance: 1 far away, up to 17 close by.
pub fn pace(distance: u32) -> u32 {
    if distance > FAR {
        1
    } else {
        ((FAR - distance) >> 13) + 1
    }
}

/// The blink's length.
pub const BLINK_STEPS: u8 = 0x18;

/// Kinds 1 and 9: +0x11 counts up; at `BLINK_STEPS` +0x11 and +0x13 are cleared and the
/// face leaves (None); else +0x13 = `blink(count)`, returned.
pub fn blink(f: &mut FaceBytes, blink: impl FnOnce(u8) -> u8) -> Option<u8> {
    f.b11 = f.b11.wrapping_add(1);
    if f.b11 >= BLINK_STEPS {
        f.b11 = 0;
        f.b13 = 0;
        return None;
    }
    f.b13 = blink(f.b11);
    Some(f.b13)
}

/// The tops kind 2 and kinds 3, 8 rise +0x13 to.
pub const RISE_TOP: u8 = 0x30;
pub const ALIGNED_RISE_TOP: u8 = 0x40;

/// +0x13 up by one `pace` times, stopping at `top`: (true when it is at the top: the face
/// leaves; the steps left).
pub fn rise(f: &mut FaceBytes, pace: u32, top: u8) -> (bool, u32) {
    let mut left = pace;
    loop {
        if f.b13 == top {
            return (true, left);
        }
        f.b13 = f.b13.wrapping_add(1);
        left = left.wrapping_sub(1);
        if left == 0 {
            return (false, 0);
        }
    }
}

/// Kinds 3 and 8: +9 stepped (up to `pace` x 4 steps) toward making +8 + +9 + +0x12 a
/// multiple of 0x80 (down below 0x40 past it, up from 0x40 on); once it is, `rise` to
/// `ALIGNED_RISE_TOP`. Gives (the face leaves, the steps left, the last sum & 0x7F).
pub fn align(f: &mut FaceBytes, pace: u32) -> (bool, u32, u8) {
    let mut steps = pace << 2;
    let sum = loop {
        let sum_low = f.b9.wrapping_add(f.b8).wrapping_add(f.b12) & 0x7F;
        if sum_low == 0 {
            break sum_low;
        }
        let mut b9 = f.b9.wrapping_sub(1);
        if sum_low >= 0x40 {
            b9 = b9.wrapping_add(2);
        }
        f.b9 = b9;
        steps = steps.wrapping_sub(1);
        if steps == 0 {
            return (false, 0, sum_low);
        }
    };
    let (leave, steps_left) = rise(f, pace, ALIGNED_RISE_TOP);
    (leave, steps_left, sum)
}

/// Kind 4 moves while +0x11 is not 0 (else the face leaves); its offset's turn: (the
/// distance, +0x11 << 8; the angle, (+0x12 + 0x80) << 24).
pub fn fall_turn(f: &FaceBytes) -> Option<(u32, u32)> {
    (f.b11 != 0).then(|| (u32::from(f.b11) << 8, (u32::from(f.b12) + 0x80) << 24))
}

/// The most faces the list holds.
pub const LIST_ROOM: u32 = 0x100;

/// What `touch_face` does to a face by the contact's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    /// 2, 3, 8 (and 1, 9 when the car is fast enough): +9 0, +0x11 1, +0x13 0, +0x12 the
    /// low byte of `ContactWork.face_angle`; then the sound.
    Mark,
    /// 1, 9: `Mark` unless the car's speed is below the limit (or unordered; then nothing
    /// more).
    MarkIfFast,
    /// 4: +0x11 the fall (`fall_mark`), +0x13 0, +0x12 as `Mark`; then the sound.
    Fall,
    /// 0xFF: one in about 85 (`lucky`) +0x11 0xF and counted, no sound.
    Chance,
    /// The rest: nothing to the face; the sound.
    Sound,
}

/// The kind's step.
pub fn touch(kind: u32) -> Touch {
    match kind {
        1 | 9 => Touch::MarkIfFast,
        2 | 3 | 8 => Touch::Mark,
        4 => Touch::Fall,
        0xFF => Touch::Chance,
        _ => Touch::Sound,
    }
}

/// Kind 4's +0x11: `ContactWork.face_speed` >> 14, at most 0x40.
pub fn fall_mark(v: u32) -> u32 {
    (v >> 14).min(0x40)
}

/// Kind 0xFF's draw: the random number's low byte, and whether it is below 3.
pub fn lucky(random: u32) -> (u32, bool) {
    let r = random & 0xFF;
    (r, r < 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps() {
        let mut f = FaceBytes {
            b11: 1,
            ..Default::default()
        };
        assert!(count_down(&mut f));
        assert_eq!(pace(FAR + 1), 1);
        assert_eq!(pace(0), 17);
        assert_eq!(pace(FAR - 0x2000), 2);
        f.b11 = 0x16;
        assert_eq!(blink(&mut f, |n| n + 1), Some(0x18));
        assert_eq!(blink(&mut f, |_| unreachable!()), None);
        assert_eq!((f.b11, f.b13), (0, 0));
        f.b13 = 0x2E;
        assert_eq!(rise(&mut f, 5, RISE_TOP), (true, 3));
        f.b13 = 0x10;
        assert_eq!(rise(&mut f, 5, RISE_TOP), (false, 0));
        assert_eq!(f.b13, 0x15);
    }

    #[test]
    fn aligned() {
        let mut f = FaceBytes {
            b8: 0x7E,
            b9: 0,
            b12: 0,
            b13: 0x3F,
            ..Default::default()
        };
        // 0x7E: two steps up to 0x80, then +0x13 one up to the top (the pace spent).
        assert_eq!(align(&mut f, 1), (false, 0, 0));
        assert_eq!((f.b9, f.b13), (2, 0x40));
        let mut g = FaceBytes {
            b8: 0x10,
            ..Default::default()
        };
        assert_eq!(align(&mut g, 1), (false, 0, 0xD));
        assert_eq!(g.b9, 0xFC);
        assert_eq!(fall_turn(&g), None);
        g.b11 = 2;
        g.b12 = 0x10;
        assert_eq!(fall_turn(&g), Some((0x200, 0x9000_0000)));
        assert_eq!(touch(9), Touch::MarkIfFast);
        assert_eq!(touch(0xFF), Touch::Chance);
        assert_eq!(touch(10), Touch::Sound);
        assert_eq!(fall_mark(0x7F_FFFF), 0x40);
        assert_eq!(fall_mark(0x4000), 1);
        assert_eq!(lucky(0x102), (2, true));
        assert_eq!(lucky(3), (3, false));
    }
}
