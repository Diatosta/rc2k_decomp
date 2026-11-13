//! The tables of 0x100 colours the lighting picks from (a colour is three dwords; an entry
//! packs their low bytes as 0xFFRRGGBB), and how much a light falls on a point.

/// The entries of a table.
pub const ENTRIES: usize = 0x100;

/// A part of a colour times n over 255 (signed, towards zero).
fn scaled(part: u32, n: u32) -> u32 {
    ((part as i32 as i64 * n as i64) / 255) as u32
}

/// Three parts' low bytes, 0xFF on top.
pub fn packed(parts: [u32; 3]) -> u32 {
    0xFF00_0000 | (parts[0] & 0xFF) << 16 | (parts[1] & 0xFF) << 8 | (parts[2] & 0xFF)
}

/// Entry n is `from` plus n over 255 of the way to `to` (signed, towards zero), part by part.
pub fn ramp(from: [u32; 3], to: [u32; 3]) -> [u32; ENTRIES] {
    std::array::from_fn(|n| {
        packed(std::array::from_fn(|part| {
            scaled(to[part].wrapping_sub(from[part]), n as u32).wrapping_add(from[part])
        }))
    })
}

/// Entry n is the colour times n over 255.
pub fn scale(colour: [u32; 3]) -> [u32; ENTRIES] {
    std::array::from_fn(|n| packed(colour.map(|part| scaled(part, n as u32))))
}

/// The point's three floats times the light's, added (the second and third products
/// first), times `factor`, rounded to a whole number.
pub fn light_dot(point: [f32; 3], light: [f32; 3], factor: f32) -> i32 {
    let p: [f64; 3] = std::array::from_fn(|n| point[n] as f64 * light[n] as f64);
    crate::math::geometry::fistp64((p[0] + (p[1] + p[2])) * factor as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables() {
        let t = scale([0xFF, 0x80, 0]);
        assert_eq!(
            [t[0], t[0xFF], t[0x80]],
            [0xFF00_0000, 0xFFFF_8000, 0xFF80_4000]
        );
        let r = ramp([0x10, 0xFF, 0], [0x20, 0, 0x1FF]);
        assert_eq!([r[0], r[0xFF]], [0xFF10_FF00, 0xFF20_00FF]);
        // Signed, towards zero: -0xFF * 1 / 255 = -1; 0x1FF * 1 / 255 = 2.
        assert_eq!(r[1], 0xFF10_FE02);
        assert_eq!(scale([u32::MAX, 0, 0])[0x80], 0xFF00_0000);
    }

    #[test]
    fn dot() {
        assert_eq!(light_dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], 0.5), 16);
        assert_eq!(light_dot([1.0, 1.0, 1.0], [0.5, 0.5, 1.5], 1.0), 2);
        assert_eq!(light_dot([f32::NAN, 1.0, 1.0], [1.0; 3], 1.0), i32::MIN);
    }
}
