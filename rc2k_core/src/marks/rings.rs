//! The marks' rings of 0x1000 entries (converted 2026-10-21 from `marks::allocate_ring`
//! 0x4CFAC5 and `marks::allocate_links` 0x4C5398): each entry linked to the one before
//! and the one after, round.

/// The entries of a ring.
pub const ENTRIES: u32 = 0x1000;
/// `g_marksRing`'s entries' size.
pub const RING_ENTRY: u32 = 0x4C;

/// Entry n's neighbours (before, after), round.
pub fn neighbours(n: u32) -> [u32; 2] {
    [
        n.wrapping_sub(1) & (ENTRIES - 1),
        n.wrapping_add(1) & (ENTRIES - 1),
    ]
}

/// `allocate_ring`'s links: each entry's (+0, +4) the addresses of the one before and
/// the one after, in a block at `block` of 0x4C-byte entries.
pub fn ring_links(block: u32) -> impl Iterator<Item = [u32; 2]> {
    (0..ENTRIES).map(move |n| neighbours(n).map(|k| block.wrapping_add(k * RING_ENTRY)))
}

/// `allocate_links`' two tables: entry n's before and after (numbers).
pub fn link_tables() -> ([u32; ENTRIES as usize], [u32; ENTRIES as usize]) {
    (
        std::array::from_fn(|n| neighbours(n as u32)[0]),
        std::array::from_fn(|n| neighbours(n as u32)[1]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rings() {
        assert_eq!(neighbours(0), [0xFFF, 1]);
        assert_eq!(neighbours(0xFFF), [0xFFE, 0]);
        let links: Vec<_> = ring_links(0x1000).collect();
        assert_eq!(links[0], [0x1000 + 0xFFF * 0x4C, 0x1000 + 0x4C]);
        assert_eq!(links[0xFFF], [0x1000 + 0xFFE * 0x4C, 0x1000]);
        let (before, after) = link_tables();
        assert_eq!((before[5], after[5], after[0xFFF]), (4, 6, 0));
    }
}
