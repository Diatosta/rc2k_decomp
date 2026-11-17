//! The smaller levels of a texture for the GL's texture arrays: each texel the average of the
//! four it covers in the level above, per channel (the filter `rasteriser::mip_chain` uses).

/// How many levels a texture of `width` x `height` has, the full one included: halving each side
/// (to a least of 1) until both are 1.
pub fn level_count(width: usize, height: usize) -> usize {
    (usize::BITS - width.max(height).max(1).leading_zeros()) as usize
}

/// A level's side `level` steps down from `side`.
pub fn side_at(side: usize, level: usize) -> usize {
    (side >> level).max(1)
}

/// The levels below the full one of a picture of RGBA `bytes` (`width` x `height`, all rows),
/// the largest first. Each is `(width, height, bytes)`.
pub fn lower_levels(bytes: &[u8], width: usize, height: usize) -> Vec<(usize, usize, Vec<u8>)> {
    let mut levels: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    let (mut w, mut h) = (width.max(1), height.max(1));
    if bytes.len() < w * h * 4 {
        return levels;
    }
    for _ in 1..level_count(width, height) {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let above: &[u8] = levels.last().map_or(bytes, |l| &l.2);
        let mut level = Vec::with_capacity(nw * nh * 4);
        for y in 0..nh {
            for x in 0..nw {
                let (x0, y0) = (x * 2, y * 2);
                let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
                for channel in 0..4 {
                    let at = |x: usize, y: usize| u32::from(above[(y * w + x) * 4 + channel]);
                    let sum = at(x0, y0) + at(x1, y0) + at(x0, y1) + at(x1, y1);
                    level.push(((sum + 2) >> 2) as u8);
                }
            }
        }
        levels.push((nw, nh, level));
        (w, h) = (nw, nh);
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_levels_down_to_one_texel() {
        assert_eq!(level_count(256, 256), 9);
        assert_eq!(level_count(256, 64), 9);
        assert_eq!(level_count(1, 1), 1);
        assert_eq!(side_at(64, 8), 1);
    }

    #[test]
    fn averages_four_texels() {
        let bytes: Vec<u8> = [
            0u8, 4, 8, 255, 4, 8, 12, 255, 8, 12, 16, 255, 12, 16, 20, 255,
        ]
        .to_vec();
        let levels = lower_levels(&bytes, 2, 2);
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0], (1, 1, vec![6, 10, 14, 255]));
    }

    #[test]
    fn a_wide_picture_keeps_one_row() {
        let bytes = vec![100u8; 4 * 4];
        let levels = lower_levels(&bytes, 4, 1);
        let sizes: Vec<_> = levels.iter().map(|l| (l.0, l.1)).collect();
        assert_eq!(sizes, vec![(2, 1), (1, 1)]);
    }
}
