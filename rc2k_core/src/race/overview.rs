//! The overview map a three-player split screen shows in the part no player has: the whole
//! course's map tiles fitted into one 128 x 128 picture, and where a car's place on the map lands
//! in that picture.

/// The picture's side, in pixels (the map pictures' size).
pub const SIDE: u32 = 128;
/// A map tile's side, in map pixels.
const TILE: u32 = super::map::TILE;

/// How the course's used tiles are fitted into the picture: the map pixel at the picture's top
/// left corner and the map pixels each picture pixel covers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub origin: (f32, f32),
    pub ratio: f32,
}

impl Fit {
    /// The fit that centres the used tiles' bounding box in the picture, the longer side filling
    /// it; `None` when no tile is used. `used(x, y)` says whether tile (x, y) is.
    pub fn of(used: impl Fn(u32, u32) -> bool) -> Option<Fit> {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
        for y in 0..super::map::TILES {
            for x in 0..super::map::TILES {
                if used(x, y) {
                    (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                }
            }
        }
        if x0 == u32::MAX {
            return None;
        }
        let (w, h) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
        let longest = w.max(h);
        let ratio = longest * TILE as f32 / SIDE as f32;
        let extent = SIDE as f32 * ratio;
        let tile = TILE as f32;
        Some(Fit {
            origin: (
                x0 as f32 * tile + (w * tile - extent) / 2.0,
                y0 as f32 * tile + (h * tile - extent) / 2.0,
            ),
            ratio,
        })
    }

    /// A map pixel's place in the picture, in picture pixels (not limited to the picture).
    pub fn in_picture(&self, map: (f32, f32)) -> (f32, f32) {
        (
            (map.0 - self.origin.0) / self.ratio,
            (map.1 - self.origin.1) / self.ratio,
        )
    }

    /// The map pixels picture pixel (`x`, `y`) covers: left, top, right, bottom (exclusive), at
    /// least one pixel each way, a negative edge held at 0.
    fn covered(&self, x: u32, y: u32) -> (u32, u32, u32, u32) {
        let edge = |origin: f32, n: u32| {
            let lo = (origin + n as f32 * self.ratio).floor().max(0.0) as u32;
            let hi = (origin + (n + 1) as f32 * self.ratio).ceil().max(0.0) as u32;
            (lo, hi.max(lo + 1))
        };
        let (left, right) = edge(self.origin.0, x);
        let (top, bottom) = edge(self.origin.1, y);
        (left, top, right, bottom)
    }

    /// The picture, row by row: each pixel `blend` of the map pixels `source` gives in the box it
    /// covers (only those it gives), `None` where none gives one.
    pub fn paint(
        &self,
        source: impl Fn(u32, u32) -> Option<u32>,
        blend: impl Fn(&[u32]) -> u32,
    ) -> Vec<Option<u32>> {
        let mut picture = Vec::with_capacity((SIDE * SIDE) as usize);
        let mut box_pixels = Vec::new();
        for y in 0..SIDE {
            for x in 0..SIDE {
                let (left, top, right, bottom) = self.covered(x, y);
                box_pixels.clear();
                for sy in top..bottom {
                    box_pixels.extend((left..right).filter_map(|sx| source(sx, sy)));
                }
                picture.push((!box_pixels.is_empty()).then(|| blend(&box_pixels)));
            }
        }
        picture
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_square_course_fills_the_picture() {
        // A 32 x 32 block of tiles at (10, 20).
        let fit = Fit::of(|x, y| (10..42).contains(&x) && (20..52).contains(&y)).unwrap();
        assert_eq!(fit.ratio, 4.0);
        assert_eq!(fit.origin, (160.0, 320.0));
        assert_eq!(fit.in_picture((160.0, 320.0)), (0.0, 0.0));
        assert_eq!(
            fit.in_picture((160.0 + 512.0, 320.0 + 512.0)),
            (128.0, 128.0)
        );
    }

    #[test]
    fn a_wide_course_is_centred_in_the_other_direction() {
        // 16 tiles wide, 8 high: 256 x 128 map pixels in a 2-pixel-to-one picture.
        let fit = Fit::of(|x, y| x < 16 && y < 8).unwrap();
        assert_eq!(fit.ratio, 2.0);
        // The course's vertical middle (64) is the picture's (64).
        assert_eq!(fit.in_picture((0.0, 64.0)).1, 64.0);
        assert_eq!(fit.in_picture((0.0, 0.0)).1, 32.0);
    }

    #[test]
    fn no_tiles_no_fit() {
        assert_eq!(Fit::of(|_, _| false), None);
    }

    #[test]
    fn a_box_with_a_pixel_gives_it() {
        let fit = Fit {
            origin: (0.0, 0.0),
            ratio: 4.0,
        };
        let picture = fit.paint(
            |x, y| (x < 8 && y < 4).then_some(if x == 6 { 8 } else { 4 }),
            |pixels| pixels.iter().sum::<u32>() / pixels.len() as u32,
        );
        // Picture pixel (1, 0) covers map pixels 4..8 x 0..4: sixteen, four of them (x = 6) of 8.
        assert_eq!(picture[1], Some(5));
        assert_eq!(picture[0], Some(4));
        assert_eq!(picture[2], None);
        assert_eq!(picture.len(), (SIDE * SIDE) as usize);
    }

    #[test]
    fn a_picture_pixel_always_covers_a_map_pixel() {
        let fit = Fit {
            origin: (-30.0, 5.5),
            ratio: 0.25,
        };
        let (l, t, r, b) = fit.covered(0, 0);
        assert!(r > l && b > t);
        assert_eq!(l, 0);
    }
}
