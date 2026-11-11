//! The loading bar's progress while the textures load (from
//! `scene::step_loading_bar` 0x446786).

/// The bar's ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarEnds {
    pub low: u32,
    pub high: u32,
}

impl BarEnds {
    /// The span (the high end less the low, wrapping).
    pub fn span(self) -> u32 {
        self.high.wrapping_sub(self.low)
    }

    /// The progress with `done` of `textures` system textures loaded: the low end plus
    /// done times the span (the product's low half) over the textures (unsigned), plus
    /// 1; none with no textures (the progress is left as it is).
    pub fn progress(self, done: u32, textures: u32) -> Option<u32> {
        if textures == 0 {
            return None;
        }
        let span = done.wrapping_mul(self.span());
        Some((span / textures).wrapping_add(self.low).wrapping_add(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress() {
        let ends = BarEnds { low: 10, high: 110 };
        assert_eq!(ends.progress(5, 0), None);
        assert_eq!(ends.progress(0, 20), Some(11));
        assert_eq!(ends.progress(20, 20), Some(111));
        assert_eq!(ends.progress(3, 7), Some(10 + 300 / 7 + 1));
        // the product's low half
        assert_eq!(
            BarEnds {
                low: 0,
                high: 0x1_0000
            }
            .progress(0x1_0000, 1),
            Some(1)
        );
    }
}
