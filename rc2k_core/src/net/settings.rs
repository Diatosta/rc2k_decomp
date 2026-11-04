//! The network game's copy of player 1's settings (converted 2026-10-21 from
//! `net::take_player_settings` 0x48755D): 0x3A8 bytes (from `dword_75ACE8`: the car, the
//! driving aids and the HUD's switches, the controls, the co-driver, the view), filled
//! from player 1's record (`g_players[0]`) before each network session, in the original's
//! order. Offsets are into the settings and into the record.

/// The settings' bytes.
pub const LEN: usize = 0x3A8;

/// One copy from the record into the settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Copy {
    /// A dword.
    Dword { to: u32, from: u32 },
    /// Bytes, one at a time.
    Bytes { to: u32, from: u32, len: u32 },
}

/// The dwords: (the settings' offset, the record's), in the original's order. The
/// settings' 0xA4 is written twice: the second (the record's 0x1A0) stays.
pub const DWORDS: [(u32, u32); 91] = [
    (0x0, 0x8),
    (0x34, 0x10C),
    (0x38, 0x110),
    (0x3C, 0x114),
    (0x40, 0x118),
    (0x44, 0x11C),
    (0x48, 0x120),
    (0x31C, 0x288),
    (0x32C, 0x278),
    (0x4C, 0xD4),
    (0x50, 0xD8),
    (0x54, 0xDC),
    (0x58, 0xE0),
    (0x5C, 0xE4),
    (0x60, 0xE8),
    (0x64, 0xEC),
    (0x68, 0xF0),
    (0x6C, 0xF4),
    (0x70, 0xF8),
    (0x74, 0xFC),
    (0x78, 0x100),
    (0xA4, 0x104),
    (0x7C, 0x108),
    (0x110, 0x158),
    (0x114, 0x15C),
    (0x118, 0x160),
    (0x120, 0x168),
    (0x124, 0x164),
    (0x12C, 0x170),
    (0x128, 0x16C),
    (0x130, 0x174),
    (0x11C, 0x178),
    (0x324, 0x284),
    (0x80, 0x17C),
    (0x84, 0x180),
    (0x88, 0x184),
    (0x8C, 0x188),
    (0x90, 0x18C),
    (0x94, 0x190),
    (0x98, 0x194),
    (0x9C, 0x198),
    (0xA0, 0x19C),
    (0xA4, 0x1A0),
    (0xA8, 0x1A4),
    (0xAC, 0x1A8),
    (0xB0, 0x1AC),
    (0xB4, 0x1B0),
    (0xB8, 0x1B4),
    (0xBC, 0x1B8),
    (0xC0, 0x1BC),
    (0xC4, 0x1C0),
    (0xC8, 0x1C4),
    (0xCC, 0x1C8),
    (0xD0, 0x1CC),
    (0xD4, 0x1D0),
    (0xD8, 0x1D4),
    (0xDC, 0x1D8),
    (0xE0, 0x1DC),
    (0xE4, 0x1E0),
    (0xE8, 0x1E4),
    (0xEC, 0x1E8),
    (0xF0, 0x1EC),
    (0xF4, 0x1F0),
    (0xF8, 0x1F4),
    (0xFC, 0x1F8),
    (0x100, 0x1FC),
    (0x104, 0x200),
    (0x108, 0x204),
    (0x10C, 0x208),
    (0x134, 0x28),
    (0x138, 0x154),
    (0x13C, 0xC),
    (0x140, 0x2C),
    (0x144, 0x30),
    (0x148, 0x34),
    (0x14C, 0x38),
    (0x150, 0x3C),
    (0x154, 0x40),
    (0x158, 0x44),
    (0x15C, 0x48),
    (0x164, 0x50),
    (0x168, 0x54),
    (0x16C, 0x58),
    (0x170, 0x5C),
    (0x174, 0x60),
    (0x178, 0x64),
    (0x17C, 0x68),
    (0x180, 0x6C),
    (0x184, 0x70),
    (0x18C, 0x74),
    (0x190, 0x78),
];

/// The 0x20 bytes from the record's 0x24C, copied after the eighth dword.
pub const BLOCK: Copy = Copy::Bytes {
    to: 0x384,
    from: 0x24C,
    len: 0x20,
};
const BLOCK_AFTER: usize = 8;

/// The twelve bytes from the record's 0x1C, copied last.
pub const LAST: Copy = Copy::Bytes {
    to: 0x194,
    from: 0x1C,
    len: 12,
};

/// Every copy, in the original's order.
pub fn copies() -> Vec<Copy> {
    let mut all: Vec<Copy> = DWORDS
        .iter()
        .map(|&(to, from)| Copy::Dword { to, from })
        .collect();
    all.insert(BLOCK_AFTER, BLOCK);
    all.push(LAST);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        let all = copies();
        assert_eq!(all.len(), 93);
        assert_eq!(all[8], BLOCK);
        assert_eq!(all[0], Copy::Dword { to: 0, from: 8 });
        let mut written = vec![0u32; LEN];
        for c in &all {
            let (to, len) = match *c {
                Copy::Dword { to, .. } => (to, 4),
                Copy::Bytes { to, len, .. } => (to, len),
            };
            assert!((to + len) as usize <= LEN);
            for n in to..to + len {
                written[n as usize] += 1;
            }
        }
        // Only the one dword twice.
        assert_eq!(written.iter().filter(|&&n| n > 1).count(), 4);
        assert_eq!(written[0xA4], 2);
    }
}
