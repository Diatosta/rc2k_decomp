//! The recorded results: what each helper's cases gave when it matched the x87's
//! instructions, kept as hashes (`test_support::frozen`) and checked again on the same seeded
//! inputs: `arithmetic` (the plain float helpers), `by_control_word` (the later helpers, one
//! hash per control word) and `loads_and_runs` (the load and run helpers, the 80-bit
//! conversions and the normalising).

mod arithmetic;
mod by_control_word;
mod loads_and_runs;

pub(super) use super::plain;

/// The control words the game's floats run under: nearest (double precision), chop, down
/// and up.
const ROUNDING_MODES: [u16; 4] = [0x027F, 0x0E7F, 0x067F, 0x0A7F];

/// A plain helper's result as the reference's: the bits of a float result, a whole number or
/// a status word as it is.
pub(in crate::math::x87_checks) trait AsBits {
    type Out;
    fn into_bits(self) -> Self::Out;
}

impl AsBits for f32 {
    type Out = u32;
    fn into_bits(self) -> u32 {
        self.to_bits()
    }
}

macro_rules! as_they_are {
    ($($t:ty),*) => {$(
        impl AsBits for $t {
            type Out = $t;
            fn into_bits(self) -> $t {
                self
            }
        }
    )*};
}
as_they_are!(u8, u16, u32, i32, bool, f64);

impl<A: AsBits, B: AsBits> AsBits for (A, B) {
    type Out = (A::Out, B::Out);
    fn into_bits(self) -> Self::Out {
        (self.0.into_bits(), self.1.into_bits())
    }
}

impl<A: AsBits, B: AsBits, C: AsBits> AsBits for (A, B, C) {
    type Out = (A::Out, B::Out, C::Out);
    fn into_bits(self) -> Self::Out {
        (self.0.into_bits(), self.1.into_bits(), self.2.into_bits())
    }
}

impl<A: AsBits, const N: usize> AsBits for [A; N] {
    type Out = [A::Out; N];
    fn into_bits(self) -> Self::Out {
        self.map(AsBits::into_bits)
    }
}
