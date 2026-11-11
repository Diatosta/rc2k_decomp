//! The marks' slots (from `effects::add_mark_slot` 0x4C99F4): 0x20
//! slots of five dwords (a time, two values, 0, a float); a new one takes the last free
//! slot with the time 0x3C and a float of the slots' own random. As in `math::geometry`,
//! what the x87 keeps on its stack is an `f64`: the same at the game's double precision.

/// The slots (`g_markSlots`, 0x14 bytes each) and a new slot's time.
pub const SLOTS: usize = 0x20;
pub const TIME: u32 = 0x3C;

/// The last free slot (its first dword 0 or less, signed), from 0x1F down.
pub fn free_slot(first: impl Fn(usize) -> i32) -> Option<usize> {
    (0..SLOTS).rev().find(|&n| first(n) <= 0)
}

/// The slots' random's next (`g_marksSlotRandom` times 0xC5 plus 0x5D).
pub fn next_random(random: u32) -> u32 {
    random.wrapping_mul(0xC5).wrapping_add(0x5D)
}

/// A new slot (`what` eax, `at` esi): `fild` the random's low word, `fadd base`.
pub fn new_slot(what: u32, at: u32, random: u32, base: f32) -> [u32; 5] {
    let value = ((random & 0xFFFF) as f64 + base as f64) as f32;
    [TIME, what, at, 0, value.to_bits()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots() {
        assert_eq!(free_slot(|_| 0), Some(0x1F));
        assert_eq!(free_slot(|n| if n == 3 { -5 } else { 1 }), Some(3));
        assert_eq!(free_slot(|_| 1), None);
        assert_eq!(next_random(0), 0x5D);
        assert_eq!(next_random(u32::MAX), 0x5D_u32.wrapping_sub(0xC5));
        assert_eq!(
            new_slot(1, 2, 0x1_0005, 0.5),
            [0x3C, 1, 2, 0, 5.5f32.to_bits()]
        );
    }
}
