//! Floats in places that may overlap, shared by the scene's and the effects' matrix helpers.
//! Functions take inputs and result by place and write each part before working out the
//! next, so a result over an input reads back what was just written. A place is a number
//! (the game-side adapter uses the address, unaligned places included); a dword is a
//! float's bits, copied as they are.

/// Dwords at numbered places, four bytes a float.
pub trait Places {
    fn dword(&self, at: u32) -> u32;
    fn set_dword(&mut self, at: u32, value: u32);

    fn float(&self, at: u32) -> f32 {
        f32::from_bits(self.dword(at))
    }

    fn set_float(&mut self, at: u32, value: f32) {
        self.set_dword(at, value.to_bits());
    }

    fn floats<const N: usize>(&self, at: u32) -> [f32; N] {
        std::array::from_fn(|n| self.float(at.wrapping_add(4 * n as u32)))
    }

    fn set_floats(&mut self, at: u32, values: &[f32]) {
        for (n, &v) in values.iter().enumerate() {
            self.set_float(at.wrapping_add(4 * n as u32), v);
        }
    }
}

/// A float as a load and store of it leave it: the same, but a signalling NaN made quiet.
pub fn loaded(bits: u32) -> u32 {
    if f32::from_bits(bits).is_nan() {
        bits | 0x0040_0000
    } else {
        bits
    }
}

/// For the tests: dwords from place 0 on (a place is a byte number, a multiple of four).
#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub struct TestPlaces(pub Vec<u32>);

#[cfg(test)]
impl Places for TestPlaces {
    fn dword(&self, at: u32) -> u32 {
        self.0[at as usize / 4]
    }

    fn set_dword(&mut self, at: u32, value: u32) {
        self.0[at as usize / 4] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_in_order() {
        let mut m = TestPlaces(vec![0; 4]);
        m.set_floats(4, &[1.0, 2.0, 3.0]);
        assert_eq!(m.floats::<3>(4), [1.0, 2.0, 3.0]);
        assert_eq!(m.dword(0), 0);
        m.set_dword(0, f32::NAN.to_bits() | 1);
        assert_eq!(m.float(0).to_bits(), f32::NAN.to_bits() | 1);
        assert_eq!(loaded(0x7F80_0001), 0x7FC0_0001);
        assert_eq!(loaded(0xFF80_0000), 0xFF80_0000);
        assert_eq!(loaded(1.5f32.to_bits()), 1.5f32.to_bits());
    }
}
