//! The numbers the replay viewer draws itself on its panel's pictures. The panel's atlas holds
//! the digits 1 and 2 only (in the car item's boxes and in the camera icons' circles), so the
//! car item of a further car and the camera icons of other cars than the first two are those
//! pictures with their digit covered by a piece of their own plain face and the number drawn
//! over them in the game's heavy notice font (`gfx::notice_text`), in black.
//!
//! The atlas' own digits are that font's letters in black without their outline: a glyph of the
//! font is a white fill inside a black outline, and the atlas' digit is the fill alone, its
//! brightness the black's share, enlarged by 20/13 in the car item (the fill is 13 pixels high,
//! the digit 20 units) and by 9/8 in the camera icons. This module keeps the measures read off the
//! atlas' pixels (a fit of the font's glyphs to the atlas' 1 and 2 leaves the sum of squared
//! differences under 4 % of the digit's weight) that put a drawn digit where the atlas' own would
//! be: the drawn 1 and 2 land within a pixel of the originals. All lengths are in the picture's
//! own units (the picture is 48 by 48), counted from its top left.

/// Where a drawn digit goes on a picture, and what it covers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    /// The picture's own digit with a margin, as a rectangle (x, y, width, height) of the
    /// picture: covered by a piece of the plain face.
    pub covered: [u32; 4],
    /// A rectangle of the plain face (x, y, width, height) that fills `covered`: flat, and with
    /// flat texels all round it, so that stretching it shows no seam.
    pub face: [u32; 4],
    /// The picture's x that the centre of mass of the digit lands on: the atlas' digits sit by their
    /// weight (a 1's flag hangs to the left of its stem), not by the corners of their shape.
    pub centre_x: f32,
    /// The picture's y that the first row of the digit with any of its fill in lands on (all the
    /// heavy font's digits have the same top).
    pub top_y: f32,
    /// How many units one pixel of the font is.
    pub scale: f32,
}

/// The height of the heavy font's digits' fill, in the font's pixels (the round digits' is a
/// pixel more, as they overshoot).
pub const FONT_FILL_HEIGHT: f32 = 13.0;

/// The car item's picture: the first car's box "1" (the box's face is from x 2 to 45, y 8 to 38).
/// The digit of the atlas' "1" lies from x 12 to 28, y 13 to 32, the "2"'s from x 12 to 34 on the
/// same rows (picture "2" starts 1 unit to the left of picture "1"; the two digits' weights both
/// lie at x 23.3 to 23.4): 20 units high, the font's 13 pixels of fill enlarged by 20/13.
pub const CAR: Spot = Spot {
    covered: [10, 11, 20, 24],
    face: [32, 10, 4, 4],
    centre_x: 23.35,
    top_y: 12.75,
    scale: 20.0 / FONT_FILL_HEIGHT,
};

/// The first camera icon's picture (a camera with a circled 1) and the second's (a circled 2):
/// the digit's weight lies at x 31.7 and 32.6, its top at y 9, 15 units high: the fill enlarged
/// by 9/8.
pub const CAMERA_FIRST: Spot = Spot {
    covered: [23, 8, 15, 17],
    face: [40, 14, 3, 3],
    centre_x: 31.7,
    top_y: 9.0,
    scale: 9.0 / 8.0,
};
pub const CAMERA_SECOND: Spot = Spot {
    covered: [24, 9, 18, 16],
    face: [43, 13, 2, 2],
    centre_x: 32.6,
    top_y: 9.0,
    scale: 9.0 / 8.0,
};

/// The number the car item of a layout that shows one of the third to the eighth player alone
/// has no picture for (4 .. 9): the player's, counted from 1.
pub fn car_digit(layout: u8) -> Option<u8> {
    (4..=9).contains(&layout).then(|| layout - 1)
}

/// The camera icon of camera button `button` (0 .. 3), which edits the camera of player
/// `player` (counted from 0): the number to draw on it and where. The atlas holds the circled
/// 1 and 2 for the first two buttons, which need nothing while they edit the camera of the player
/// of their own number; the further buttons have pictures of the first two (the left icon's and
/// the right's) and always get a number.
pub fn camera_digit(button: usize, player: usize) -> Option<(u8, Spot)> {
    let number = u8::try_from(player + 1).ok()?;
    if button < 2 && player == button {
        return None;
    }
    let spot = if button.is_multiple_of(2) {
        CAMERA_FIRST
    } else {
        CAMERA_SECOND
    };
    Some((number, spot))
}

/// What a shape's pixels say about where to put it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ink {
    /// The x (the pixels' left edges at whole numbers) of the centre of mass of the shape: the
    /// pixels' x weighted by their coverage.
    pub centre_x: f32,
    /// The first row with any coverage.
    pub top: i32,
}

/// The `Ink` of a picture of `width` by `height` pixels whose pixel (x, y) is covered by
/// `coverage` (0 to 1) of the shape; none for an empty shape.
pub fn ink_of(width: i32, height: i32, coverage: impl Fn(i32, i32) -> f32) -> Option<Ink> {
    let (mut sum, mut weight, mut top) = (0.0f64, 0.0f64, None);
    for y in 0..height {
        for x in 0..width {
            let c = f64::from(coverage(x, y));
            if c > 0.0 {
                sum += c * (f64::from(x) + 0.5);
                weight += c;
                top.get_or_insert(y);
            }
        }
    }
    Some(Ink {
        centre_x: (sum / weight) as f32,
        top: top?,
    })
}

/// Where a shape with `ink` goes so that its weight is at `spot`'s x and its top at `spot`'s y,
/// `spot.scale` units to its pixel: the picture-unit x and y of the shape's own top left corner.
pub fn origin(ink: Ink, spot: &Spot) -> [f32; 2] {
    [
        spot.centre_x - ink.centre_x * spot.scale,
        spot.top_y - ink.top as f32 * spot.scale,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_far_cars_have_a_digit_over_the_first_picture() {
        assert_eq!(
            (3..=11).map(car_digit).collect::<Vec<_>>(),
            [
                None,
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                Some(8),
                None,
                None
            ]
        );
    }

    #[test]
    fn what_is_covered_lies_in_the_face_and_the_sample_is_outside_it() {
        // The car's box: the plain face is x 2 .. 45, y 8 .. 38.
        let inside = |[x, y, w, h]: [u32; 4], (x0, y0, x1, y1): (u32, u32, u32, u32)| {
            x >= x0 && y >= y0 && x + w <= x1 && y + h <= y1
        };
        let apart = |a: [u32; 4], b: [u32; 4]| {
            a[0] + a[2] <= b[0] || b[0] + b[2] <= a[0] || a[1] + a[3] <= b[1] || b[1] + b[3] <= a[1]
        };
        assert!(inside(CAR.covered, (2, 8, 46, 39)) && inside(CAR.face, (2, 8, 46, 39)));
        // The circles of the camera icons lie in x 21 .. 45, y 6 .. 28.
        for spot in [CAMERA_FIRST, CAMERA_SECOND] {
            assert!(inside(spot.covered, (21, 6, 46, 28)));
            assert!(inside(spot.face, (21, 6, 46, 28)));
        }
        for spot in [CAR, CAMERA_FIRST, CAMERA_SECOND] {
            assert!(apart(spot.covered, spot.face), "{spot:?}");
        }
        assert!((CAR.scale * FONT_FILL_HEIGHT - 20.0).abs() < 1e-4);
    }

    #[test]
    fn a_camera_icon_is_drawn_on_only_where_the_atlas_has_no_number() {
        // The first two buttons on their own players need nothing.
        assert_eq!(camera_digit(0, 0), None);
        assert_eq!(camera_digit(1, 1), None);
        // The same buttons on other players, and the further buttons, get the number of the
        // player on the picture of the left icon or the right.
        assert_eq!(camera_digit(0, 4), Some((5, CAMERA_FIRST)));
        assert_eq!(camera_digit(1, 1 + 4), Some((6, CAMERA_SECOND)));
        assert_eq!(camera_digit(1, 0), Some((1, CAMERA_SECOND)));
        assert_eq!(camera_digit(2, 2), Some((3, CAMERA_FIRST)));
        assert_eq!(camera_digit(3, 7), Some((8, CAMERA_SECOND)));
    }

    #[test]
    fn a_shape_is_found_by_its_weight_and_its_top() {
        // A 4 by 5 picture: an upright bar (x 2, rows 1 to 3, whole) with a flag at its left (x 1,
        // row 1, half covered).
        let on = |x: i32, y: i32| match (x, y) {
            (2, 1..=3) => 1.0,
            (1, 1) => 0.5,
            _ => 0.0,
        };
        let ink = ink_of(4, 5, on).unwrap();
        assert_eq!(ink.top, 1);
        // Pixel centres 2.5 (weight 3) and 1.5 (weight 0.5).
        assert!((ink.centre_x - (3.0 * 2.5 + 0.5 * 1.5) / 3.5).abs() < 1e-6);
        assert_eq!(ink_of(4, 5, |_, _| 0.0), None);
        // Put at the car's spot, the weight and the top land where the spot says.
        let [x, y] = origin(ink, &CAR);
        assert!((x + ink.centre_x * CAR.scale - CAR.centre_x).abs() < 1e-4);
        assert!((y + ink.top as f32 * CAR.scale - CAR.top_y).abs() < 1e-4);
    }
}
