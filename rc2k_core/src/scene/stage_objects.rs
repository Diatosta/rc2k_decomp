//! The stage objects' points as they are loaded.

/// The loading kinds whose points are turned.
pub const TURNED_KINDS: std::ops::Range<u32> = 0x190..0x258;

/// A point (x, z, y) turned a quarter and shifted for the turned kinds: (z, -x, y - `shift`),
/// wrapping; `None` (left as it is) for the others.
pub fn turn_point(kind: u32, shift: u32, p: (u32, u32, u32)) -> Option<(u32, u32, u32)> {
    TURNED_KINDS
        .contains(&kind)
        .then(|| (p.1, p.0.wrapping_neg(), p.2.wrapping_sub(shift)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turned_kinds_only() {
        assert_eq!(turn_point(0x18F, 1, (1, 2, 3)), None);
        assert_eq!(turn_point(0x258, 1, (1, 2, 3)), None);
        assert_eq!(
            turn_point(0x190, 5, (1, 2, 3)),
            Some((2, u32::MAX, u32::MAX - 1))
        );
        assert_eq!(turn_point(0x257, 0, (0, 7, 9)), Some((7, 0, 9)));
    }
}
