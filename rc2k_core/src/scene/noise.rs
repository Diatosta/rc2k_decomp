//! The scene's own generator (from `scene::noise_step` 0x440F4E and
//! `scene::next_noise` 0x440F3E): the game's random step (`math::random::step`) with
//! 0x87654321 mixed into each result, on a state of its own.

use crate::math::random;

/// What each step mixes in.
pub const MIXED_IN: u32 = 0x8765_4321;

/// `noise_step`: the state after `state`.
pub fn step(state: u32) -> u32 {
    random::step(state, MIXED_IN)
}

/// The generator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Noise {
    pub state: u32,
}

impl Noise {
    /// `next_noise`: the next number, which is the new state.
    pub fn draw(&mut self) -> u32 {
        self.state = step(self.state);
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The generator written out bit by bit.
    fn by_bits(mut state: u32) -> u32 {
        state ^= (state >> 22) & 1;
        state ^= (state >> 2) & 1;
        state ^= (state >> 1) & 1;
        state.rotate_right(1) ^ MIXED_IN
    }

    #[test]
    fn steps() {
        let mut s = 0x1234_5678u32;
        for _ in 0..1000 {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            assert_eq!(step(s), by_bits(s));
        }
        let mut n = Noise { state: 0 };
        assert_eq!(n.draw(), MIXED_IN);
        assert_eq!(n.state, MIXED_IN);
    }
}
