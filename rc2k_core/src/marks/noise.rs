//! The marks' noise tables (converted 2026-10-21 from `marks::seed_noise` 0x4C1697): 0x100
//! steps of the marks' seed (times 0x9D plus 0x47), each giving an entry, the last first,
//! of three tables: its low 18 bits, and those of it turned right 10 and 20.

/// The entries of a table.
pub const ENTRIES: usize = 0x100;

/// The seed's step.
pub fn next_seed(seed: u32) -> u32 {
    seed.wrapping_mul(0x9D).wrapping_add(0x47)
}

/// The three values a seed gives.
pub fn values(seed: u32) -> [u32; 3] {
    [0, 10, 20].map(|turn| seed.rotate_right(turn) & 0x3FFFF)
}

/// `seed_noise`: the three tables (entry 0xFF first) and the seed after.
pub fn seed_noise(mut seed: u32) -> ([[u32; ENTRIES]; 3], u32) {
    let mut tables = [[0; ENTRIES]; 3];
    for n in (0..ENTRIES).rev() {
        seed = next_seed(seed);
        for (table, v) in tables.iter_mut().zip(values(seed)) {
            table[n] = v;
        }
    }
    (tables, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables() {
        let (t, seed) = seed_noise(0);
        assert_eq!(t[0][0xFF], 0x47);
        assert_eq!(t[1][0xFF], 0);
        assert_eq!(t[2][0xFF], (0x47u32.rotate_right(20)) & 0x3FFFF);
        let mut s = 0;
        for _ in 0..0x100 {
            s = next_seed(s);
        }
        assert_eq!((seed, values(s)), (s, [t[0][0], t[1][0], t[2][0]]));
    }
}
