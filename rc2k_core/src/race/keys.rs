//! The race's view and sound keys: the first key down (tested in a fixed order) and its effect.

/// A race key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// The next camera view.
    NextView,
    /// The split screen's layout (not with one player).
    Split,
    /// The volume down.
    VolumeDown,
    /// The volume up.
    VolumeUp,
    /// One of `BIT_KEYS`: a bit of the view flags flipped.
    Bit(usize),
    /// Bit 0x40 of the first view's flags.
    Shared,
}

/// The keys that flip a view-flags bit: (bit, counter when now clear, when set), in test order.
pub const BIT_KEYS: [(u32, u32, u32); 7] = [
    (0x10, 0x117, 0x118),
    (0x08, 0x119, 0x11A),
    (0x80, 0x11B, 0x11C),
    (0x01, 0x11D, 0x11E),
    (0x04, 0x11F, 0x120),
    (0x02, 0x121, 0x122),
    (0x20, 0x123, 0x124),
];

/// The shared key's bit and its counters (clear, set).
pub const SHARED_BIT: (u32, u32, u32) = (0x40, 0x125, 0x126);

/// The first key down (`down` asked in order, until one is).
pub fn first_key(mut down: impl FnMut(Key) -> bool) -> Option<Key> {
    [Key::NextView, Key::Split, Key::VolumeDown, Key::VolumeUp]
        .into_iter()
        .chain((0..BIT_KEYS.len()).map(Key::Bit))
        .chain([Key::Shared])
        .find(|&k| down(k))
}

/// The split's layout flipped (^ 3) and its counter (0x127 when now 1, else 0x128).
pub fn split_flip(split: u32) -> (u32, u32) {
    let split = split ^ 3;
    (split, if split == 1 { 0x127 } else { 0x128 })
}

/// A volume step: `(v + step) & !3`, held to 0 ..= 0x40.
pub fn volume_step(v: u32, step: i32) -> u32 {
    ((v.wrapping_add(step as u32) & !3) as i32).clamp(0, 0x40) as u32
}

/// The volume keys' step and counter: master 0x113 / 0x114, music (with the second view's keys)
/// 0x115 / 0x116.
pub fn volume_key(up: bool, music: bool) -> (i32, u32) {
    let base = if music { 0x115 } else { 0x113 };
    if up { (4, base + 1) } else { (-4, base) }
}

pub fn flip_bit(flags: u32, (bit, clear, set): (u32, u32, u32)) -> (u32, u32) {
    let flags = flags ^ bit;
    (flags, if flags & bit != 0 { set } else { clear })
}

/// The camera views with a counter of their own (below this).
pub const COUNTED_VIEWS: u32 = 7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        assert_eq!(first_key(|_| false), None);
        assert_eq!(first_key(|k| k == Key::Shared), Some(Key::Shared));
        assert_eq!(
            first_key(|k| matches!(k, Key::Bit(_) | Key::Shared)),
            Some(Key::Bit(0))
        );
        let mut asked = Vec::new();
        first_key(|k| {
            asked.push(k);
            k == Key::VolumeDown
        });
        assert_eq!(asked, [Key::NextView, Key::Split, Key::VolumeDown]);
        assert_eq!(split_flip(2), (1, 0x127));
        assert_eq!(split_flip(1), (2, 0x128));
        assert_eq!(volume_step(0x3E, 4), 0x40);
        assert_eq!(volume_step(2, -4), 0);
        assert_eq!(volume_step(0x40, 4), 0x40);
        assert_eq!(volume_key(true, false), (4, 0x114));
        assert_eq!(volume_key(false, true), (-4, 0x115));
        assert_eq!(flip_bit(0, BIT_KEYS[0]), (0x10, 0x118));
        assert_eq!(flip_bit(0x10, BIT_KEYS[0]), (0, 0x117));
    }
}
