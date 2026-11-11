//! Pictures made another size, smoothly (from `gfx::rescale_picture`
//! 0x4061D2 and `gfx::rescale_to_surface` 0x4063B7, which share the arithmetic).
//!
//! For every pixel of the new size: its place in the old picture as a whole number and a
//! 32-bit fraction, both stepped by old size / new size; of the fraction the top eight bits
//! weigh the pixel and its right neighbour, and the row and the row below (the last row is
//! its own row below). Three bytes of each pixel are made that way.

/// One part towards another: from + ((to - from) * weight >> 8), in 32 bits.
pub fn blend(from: i32, to: i32, weight: i32) -> i32 {
    from.wrapping_add(to.wrapping_sub(from).wrapping_mul(weight) >> 8)
}

/// (whole part, fraction as 32 bits) of a / b; `b` not 0.
pub fn step(a: u32, b: u32) -> (u32, u32) {
    (
        a / b,
        (((a % b) as u64) << 32).wrapping_div(b as u64) as u32,
    )
}

/// The sizes, none 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sizes {
    pub from_width: u32,
    pub from_height: u32,
    pub width: u32,
    pub height: u32,
}

/// What the rescaler leaves in its work variables (the counters end at 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Work {
    pub step_x: u32,
    pub step_x_fraction: u32,
    pub step_y: u32,
    pub step_y_fraction: u32,
    /// The last pixel's weights.
    pub weight_x: u32,
    pub weight_y: u32,
}

/// The picture rescaled. `at(row, byte)` reads byte `byte` of the old picture's row `row`
/// (a pixel is four bytes; the right neighbour of a row's last pixel is read as the four
/// bytes after it). `out` gets each new pixel's three parts, row by row. `None` for a size
/// of 0 (the original divides by it).
pub fn rescale(
    sizes: &Sizes,
    at: &dyn Fn(u32, u32) -> u8,
    out: &mut dyn FnMut([u8; 3]),
) -> Option<Work> {
    let Sizes {
        from_width,
        from_height,
        width,
        height,
    } = *sizes;
    if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
        return None;
    }
    let (step_x, step_x_fraction) = step(from_width, width);
    let (step_y, step_y_fraction) = step(from_height, height);
    let mut work = Work {
        step_x,
        step_x_fraction,
        step_y,
        step_y_fraction,
        ..Work::default()
    };
    let (mut y, mut y_fraction) = (0u32, 0u32);
    for _ in 0..height {
        let weight_y = (y_fraction >> 24) as i32;
        work.weight_y = weight_y as u32;
        let below = if y.wrapping_add(1) != from_height {
            y.wrapping_add(1)
        } else {
            y
        };
        let (mut x, mut x_fraction) = (0u32, 0u32);
        for _ in 0..width {
            let weight_x = (x_fraction >> 24) as i32;
            work.weight_x = weight_x as u32;
            let part_of =
                |row: u32, part: u32| at(row, x.wrapping_mul(4).wrapping_add(part)) as i32;
            let parts: [u8; 3] = std::array::from_fn(|part| {
                let part = part as u32;
                let upper = blend(part_of(y, part), part_of(y, part + 4), weight_x);
                let lower = blend(part_of(below, part), part_of(below, part + 4), weight_x);
                blend(upper, lower, weight_y) as u8
            });
            out(parts);
            let (fraction, carry) = x_fraction.overflowing_add(step_x_fraction);
            x_fraction = fraction;
            x = x.wrapping_add(step_x).wrapping_add(carry as u32);
        }
        let (fraction, carry) = y_fraction.overflowing_add(step_y_fraction);
        y_fraction = fraction;
        y = y.wrapping_add(step_y).wrapping_add(carry as u32);
    }
    Some(work)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_and_doubles() {
        assert_eq!(step(3, 2), (1, 0x8000_0000));
        // 2 x 1 to 4 x 1: 0x00, 0x80 (half way), 0xFF, then the 4 bytes past (0) blended.
        let row = [0u8, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0];
        let mut got = Vec::new();
        let sizes = Sizes {
            from_width: 2,
            from_height: 1,
            width: 4,
            height: 1,
        };
        let work = rescale(&sizes, &|_, b| row[b as usize], &mut |p| got.push(p[0])).unwrap();
        assert_eq!(got, [0x00, 0x7F, 0xFF, 0x7F]); // 0xFF + (-0xFF * 0x80 >> 8), rounded down
        assert_eq!(work.weight_x, 0x80);
        assert!(rescale(&Sizes { width: 0, ..sizes }, &|_, _| 0, &mut |_| ()).is_none());
    }
}
