//! The race's sound tables, made ready once: whole numbers as floats, spans, reciprocals and
//! the wobble's sine table. Floats keep the x87 behaviour: `f64` for the stack, rounded to
//! `f32` where stored (`fsin` is not rounded by the precision control).

/// The dword's top-byte mask, and the dword made a float (one rounding) when it looks like a
/// whole number (that byte 0 or 0xFF).
pub fn whole_to_float(word: u32) -> (u32, Option<u32>) {
    let top = word & 0xFF00_0000;
    let float = (top == 0 || top == 0xFF00_0000).then(|| (word as i32 as f32).to_bits());
    (top, float)
}

/// A span's length: its second value less its first.
pub fn span(first: f32, second: f32) -> f32 {
    (second as f64 - first as f64) as f32
}

/// A record's value made its reciprocal.
pub fn reciprocal(v: f32) -> f32 {
    (1.0 / v as f64) as f32
}

/// The sine table's length.
pub const SINES: usize = 0x40;
/// The sines' scale (0.05).
pub(crate) const WOBBLE: f32 = f32::from_bits(0x3D4C_CCCD);

/// The sine table's step: pi / 32.
pub fn sine_step() -> f32 {
    (std::f64::consts::PI / 32.0) as f32
}

/// The table 1 + 0.05 sin(x) for x from 0 by the step (each x rounded to a float), and the
/// last x.
pub fn sines() -> ([f32; SINES], f32) {
    let step = sine_step();
    let mut x = 0.0f32;
    let mut table = [0.0f32; SINES];
    for v in &mut table {
        *v = (crate::math::portable::sin(x as f64) * WOBBLE as f64 + 1.0) as f32;
        x = (x as f64 + step as f64) as f32;
    }
    (table, x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wholes_and_spans() {
        assert_eq!(whole_to_float(5), (0, Some(5.0f32.to_bits())));
        assert_eq!(
            whole_to_float(0xFFFF_FFFB),
            (0xFF00_0000, Some((-5.0f32).to_bits()))
        );
        assert_eq!(whole_to_float(0x3F80_0000), (0x3F00_0000, None));
        // 0x00FFFFFF needs 24 bits: exact; above that rounded once.
        assert_eq!(
            whole_to_float(0x00FF_FFFF).1,
            Some(16_777_215.0f32.to_bits())
        );
        assert_eq!(span(1.0, 10.0), 9.0);
        assert_eq!(reciprocal(4.0), 0.25);
    }

    #[test]
    fn sine_table() {
        let (t, last) = sines();
        assert_eq!(t[0], 1.0);
        assert!((t[16] - 1.05).abs() < 1e-6);
        assert!((t[48] - 0.95).abs() < 1e-6);
        assert!((last - 2.0 * std::f32::consts::PI).abs() < 1e-5);
        assert_eq!(sine_step().to_bits(), 0x3DC9_0FDB);
    }
}
