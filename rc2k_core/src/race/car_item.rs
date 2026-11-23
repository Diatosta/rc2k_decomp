//! The picture of the replay viewer's car item, the panel button that goes through the layouts:
//! for each layout a picture of how the screen is split between the players shown, each part
//! with its player's number.
//!
//! The panel's atlas has four pictures of the item, each a box with a black frame on a plain
//! face: the first car alone ("1"), the second alone ("2"), and the two together, the box split by
//! a line across (1 above 2) or by an upright line (1 left of 2). Every other layout's picture is
//! built from them: a further player alone is the "1" box with its digit covered and the player's
//! number drawn in its place; a pair of further players is the pair picture with both numbers
//! replaced; a group of three or four is the "1" box, its digit covered, divided by pieces of the
//! pair pictures' own dividers as the screen is divided (`view::Split`, the group's arrangement
//! that the layout names), each part with its player's number. All lengths are in the pictures'
//! units (48 by 48), counted from their top left.

use crate::Invalid;
use crate::race::layouts::{GROUP_MOST, Shown};
use crate::race::view::Split;
use crate::race::viewer_digits::{
    CAR, FONT_FILL_HEIGHT, FONT_WIDEST_DIGIT, Overlay, OwnDigit, PAIR_ABOVE_FIRST,
    PAIR_ABOVE_SECOND, PAIR_BESIDE_FIRST, PAIR_BESIDE_SECOND, PAIR_DIGIT_HEIGHT, Piece, Source,
    Spot,
};

/// The atlas' pictures of the car item, by their number in the panel's table of them: the first
/// car's box.
pub const FIRST_ALONE: u32 = 0;
/// The second car's box.
pub const SECOND_ALONE: u32 = 1;
/// The two cars' box split by a line across, the 1 above the 2.
pub const PAIR_ABOVE: u32 = 2;
/// The two cars' box split by an upright line, the 1 left of the 2.
pub const PAIR_BESIDE: u32 = 3;

/// The frame of the box (every picture's): the columns of its left and right lines and the rows
/// of its top and bottom lines. Its face lies between them.
pub const BOX_LEFT: u32 = 1;
pub const BOX_RIGHT: u32 = 46;
pub const BOX_TOP: u32 = 7;
pub const BOX_BOTTOM: u32 = 39;

/// A divider across, from the pair picture split across (`PAIR_ABOVE`): its line is row 23, with a
/// shaded row of face above and below it; flat from x 2 to 45, so a short stretch of it can be
/// made any length.
pub const DIVIDER_ACROSS: [u32; 4] = [10, 22, 4, 3];
/// The row of `DIVIDER_ACROSS` that is the line.
const DIVIDER_ACROSS_LINE: u32 = 1;
/// An upright divider, from the pair picture split upright (`PAIR_BESIDE`): its line is column 24,
/// with plain face either side; flat from y 8 to 38.
pub const DIVIDER_UPRIGHT: [u32; 4] = [24, 12, 1, 4];

/// The screen a group's split is worked out on before it is laid on the box: its sides divide
/// evenly into every number of rows and columns a split has (one to four).
const REFERENCE_SCREEN: u32 = 12;

/// A group's digits are this share of their part's height, to the nearest unit, at most (the
/// pair picture split across has 12 units of digit in a face of 15).
const DIGIT_HEIGHT_SHARE: (u32, u32) = (4, 5);
/// The face kept clear either side of a group's widest digit.
const DIGIT_SIDE_MARGIN: u32 = 1;

/// What the car item shows: one of the atlas' pictures, and what is laid over it.
#[derive(Clone, Debug, PartialEq)]
pub struct CarPicture {
    /// The atlas picture (`FIRST_ALONE` ..), drawn whole.
    pub base: u32,
    pub overlay: Overlay,
}

impl CarPicture {
    fn atlas(base: u32) -> Self {
        Self {
            base,
            overlay: Overlay::default(),
        }
    }
}

/// The car item's picture for `layout`, which shows `shown` of the replay (none: no layout of this
/// replay's). A byte that names no layout at all is left as the picture's number (the panel's
/// table then refuses it); a group's layout that shows nothing of this replay is refused, as is a
/// group whose split is none of its count's.
pub fn car_picture(layout: u8, shown: Option<Shown>) -> Result<CarPicture, Invalid> {
    use crate::race::layouts::{FIRST_GROUP, LAST_LAYOUT};
    let Some(shown) = shown else {
        // A replay without this layout (a byte the file holds, not one the viewer's cycle gives):
        // what the byte would show of any replay.
        return match layout {
            0..=3 => Ok(CarPicture::atlas(u32::from(layout))),
            4..=9 => Ok(alone(usize::from(layout) - 2)),
            FIRST_GROUP..=LAST_LAYOUT => Err(Invalid(
                "a layout of a group or pair of players the replay does not have",
            )),
            other => Ok(CarPicture::atlas(u32::from(other))),
        };
    };
    match shown {
        Shown::Player(player) => Ok(alone(player)),
        Shown::Pair { first, stacked } => Ok(pair(first, stacked)),
        Shown::Group {
            first,
            count,
            split,
        } => {
            if !Split::many_layouts(count as u32).contains(&split) {
                return Err(Invalid("a group of players in a split it has not"));
            }
            Ok(group(first, count, split))
        }
    }
}

/// Player `player` (counted from 0) alone: the first two have their own boxes, a further one the
/// first's with its number.
fn alone(player: usize) -> CarPicture {
    match player {
        0 => CarPicture::atlas(FIRST_ALONE),
        1 => CarPicture::atlas(SECOND_ALONE),
        _ => CarPicture {
            base: FIRST_ALONE,
            overlay: Overlay::over_own(number(player), CAR),
        },
    }
}

/// The players from `first` as a pair, one above the other or side by side as the screen is
/// split; the pair after the first has its numbers over the picture's.
fn pair(first: usize, stacked: bool) -> CarPicture {
    let (base, own): (u32, [OwnDigit; 2]) = if stacked {
        (PAIR_ABOVE, [PAIR_ABOVE_FIRST, PAIR_ABOVE_SECOND])
    } else {
        (PAIR_BESIDE, [PAIR_BESIDE_FIRST, PAIR_BESIDE_SECOND])
    };
    let mut overlay = Overlay::default();
    if first != 0 {
        for (k, own) in own.into_iter().enumerate() {
            overlay.put_over(Source::Own, number(first + k), own);
        }
    }
    CarPicture { base, overlay }
}

/// The number shown for player `player` (counted from 0; a layout names at most the eighth).
fn number(player: usize) -> u8 {
    player as u8 + 1
}

/// A part of a group's picture: the lines round it (left and right columns, top and bottom rows)
/// and the player in it (counted within the group; none for a part the split leaves without one).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridPart {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub player: Option<usize>,
}

impl GridPart {
    /// The face inside the lines (x, y, width, height).
    pub fn face(&self) -> [u32; 4] {
        [
            self.left + 1,
            self.top + 1,
            self.right - self.left - 1,
            self.bottom - self.top - 1,
        ]
    }
}

/// A coordinate of the reference screen laid on the box between its lines `from` and `to`,
/// rounded to the nearest whole unit (a half up, as the pair pictures put their dividers).
fn on_box(at: u32, from: u32, to: u32) -> u32 {
    from + (2 * at * (to - from) + REFERENCE_SCREEN) / (2 * REFERENCE_SCREEN)
}

/// The parts of the box for a group of `count` players split by `split`, in the split's order:
/// each player's, then any the split has beside them (the fourth quarter of three players).
pub fn grid_parts(split: Split, count: usize) -> Vec<GridPart> {
    let screen = (REFERENCE_SCREEN, REFERENCE_SCREEN);
    (0..GROUP_MOST as u32)
        .filter_map(|k| split.part_of(count as u32, k, screen))
        .enumerate()
        .map(|(k, part)| GridPart {
            left: on_box(part.x, BOX_LEFT, BOX_RIGHT),
            right: on_box(part.x + part.width, BOX_LEFT, BOX_RIGHT),
            top: on_box(part.y, BOX_TOP, BOX_BOTTOM),
            bottom: on_box(part.y + part.height, BOX_TOP, BOX_BOTTOM),
            player: (k < count).then_some(k),
        })
        .collect()
}

/// The height in units of the digits of a group whose players' faces are `faces`: the pair
/// pictures' height where every face has room for it, else as high as the smallest face lets the
/// widest digit be, the same for every part.
pub fn group_digit_height(faces: impl IntoIterator<Item = [u32; 4]>) -> u32 {
    let (share, of) = DIGIT_HEIGHT_SHARE;
    faces
        .into_iter()
        .map(|[_, _, width, height]| {
            let by_height = (height * share + of / 2) / of;
            let room = width.saturating_sub(2 * DIGIT_SIDE_MARGIN) as f32;
            let by_width = (room * FONT_FILL_HEIGHT / FONT_WIDEST_DIGIT) as u32;
            by_height.min(by_width)
        })
        .fold(PAIR_DIGIT_HEIGHT, u32::min)
}

/// Where a digit `height` units high goes in `face`: its weight in the face's middle, its height
/// centred (rounded up to a whole unit, so that its top row lies on the picture's rows).
pub fn centred_spot([x, y, width, height]: [u32; 4], digit_height: u32) -> Spot {
    Spot {
        centre_x: x as f32 + width as f32 / 2.0,
        top_y: (y + height.saturating_sub(digit_height) / 2) as f32,
        scale: digit_height as f32 / FONT_FILL_HEIGHT,
    }
}

/// The players from `first`, `count` of them, split by `split`.
fn group(first: usize, count: usize, split: Split) -> CarPicture {
    let parts = grid_parts(split, count);
    let mut overlay = Overlay::default();
    // The box's own digit covered first, then the dividers across, then the upright ones over
    // them, so that where they cross the upright line runs through the across one's shading.
    overlay.pieces.push(Piece {
        source: Source::Own,
        rect: CAR.cover.face,
        place: CAR.cover.covered,
    });
    for part in parts.iter().filter(|part| part.top > BOX_TOP) {
        overlay.pieces.push(Piece {
            source: Source::CarPicture(PAIR_ABOVE),
            rect: DIVIDER_ACROSS,
            place: [
                part.left + 1,
                part.top - DIVIDER_ACROSS_LINE,
                part.right - part.left - 1,
                DIVIDER_ACROSS[3],
            ],
        });
    }
    for part in parts.iter().filter(|part| part.left > BOX_LEFT) {
        overlay.pieces.push(Piece {
            source: Source::CarPicture(PAIR_BESIDE),
            rect: DIVIDER_UPRIGHT,
            place: [part.left, part.top, 1, part.bottom - part.top + 1],
        });
    }
    let played = || parts.iter().filter(|part| part.player.is_some());
    let height = group_digit_height(played().map(GridPart::face));
    for part in played() {
        let player = first + part.player.unwrap_or_default();
        overlay
            .numbers
            .push((number(player), centred_spot(part.face(), height)));
    }
    CarPicture {
        base: FIRST_ALONE,
        overlay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::layouts::{LAST_LAYOUT, shown};

    /// Whether `inner` (x, y, width, height) lies within x0 .. x1, y0 .. y1 (ends excluded).
    fn inside([x, y, w, h]: [u32; 4], (x0, y0, x1, y1): (u32, u32, u32, u32)) -> bool {
        x >= x0 && y >= y0 && x + w <= x1 && y + h <= y1
    }

    /// Whether two rectangles share no unit.
    fn apart(a: [u32; 4], b: [u32; 4]) -> bool {
        a[0] + a[2] <= b[0] || b[0] + b[2] <= a[0] || a[1] + a[3] <= b[1] || b[1] + b[3] <= a[1]
    }

    const PICTURE: (u32, u32, u32, u32) = (0, 0, 48, 48);
    const FACE: (u32, u32, u32, u32) = (BOX_LEFT + 1, BOX_TOP + 1, BOX_RIGHT, BOX_BOTTOM);

    /// Every split of three and four players.
    fn every_group() -> Vec<(usize, Split)> {
        let mut all = Vec::new();
        for count in [3usize, 4] {
            for &split in Split::many_layouts(count as u32) {
                all.push((count, split));
            }
        }
        all
    }

    fn group_of(first: usize, count: usize, split: Split) -> Option<Shown> {
        Some(Shown::Group {
            first,
            count,
            split,
        })
    }

    #[test]
    fn single_players_and_the_first_pair_are_the_atlas_pictures() {
        let picture = |layout, players| car_picture(layout, shown(layout, players)).unwrap();
        assert_eq!(picture(0, 1), CarPicture::atlas(FIRST_ALONE));
        assert_eq!(picture(1, 2), CarPicture::atlas(SECOND_ALONE));
        // The pictures follow the screen: 2 splits it side by side, 3 one above the other.
        assert_eq!(picture(2, 2), CarPicture::atlas(PAIR_BESIDE));
        assert_eq!(picture(3, 2), CarPicture::atlas(PAIR_ABOVE));
        // A further player alone: the first's box with its number over the box's own.
        for layout in 4..=9 {
            let player = usize::from(layout) - 2;
            assert_eq!(
                picture(layout, 8),
                CarPicture {
                    base: FIRST_ALONE,
                    overlay: Overlay::over_own(player as u8 + 1, CAR),
                }
            );
        }
    }

    #[test]
    fn the_second_pair_has_its_numbers_over_the_pair_pictures() {
        for (layout, base, own) in [
            (11, PAIR_BESIDE, [PAIR_BESIDE_FIRST, PAIR_BESIDE_SECOND]),
            (12, PAIR_ABOVE, [PAIR_ABOVE_FIRST, PAIR_ABOVE_SECOND]),
        ] {
            let picture = car_picture(layout, shown(layout, 6)).unwrap();
            assert_eq!(picture.base, base);
            let mut overlay = Overlay::default();
            overlay.put_over(Source::Own, 5, own[0]);
            overlay.put_over(Source::Own, 6, own[1]);
            assert_eq!(picture.overlay, overlay);
        }
    }

    #[test]
    fn a_byte_of_no_layout_of_the_replay() {
        assert_eq!(car_picture(2, None), Ok(CarPicture::atlas(2)));
        assert_eq!(car_picture(5, None), Ok(alone(3)));
        assert!(car_picture(10, None).is_err());
        assert!(car_picture(12, None).is_err());
        assert!(car_picture(LAST_LAYOUT, None).is_err());
        assert_eq!(
            car_picture(LAST_LAYOUT + 1, None),
            Ok(CarPicture::atlas(u32::from(LAST_LAYOUT) + 1))
        );
    }

    #[test]
    fn the_pair_pictures_dividers_are_where_the_grid_puts_them() {
        // Two parts side by side and one above the other land on the atlas' own dividers: column
        // 24 and row 23.
        let quarters = grid_parts(Split::Quarters, 4);
        assert_eq!(quarters[0].right, 24);
        assert_eq!(quarters[0].bottom, 23);
        assert_eq!(quarters[1].left, 24);
        assert_eq!(quarters[3].right, BOX_RIGHT);
        assert_eq!(quarters[3].bottom, BOX_BOTTOM);
        assert_eq!(DIVIDER_ACROSS[1] + DIVIDER_ACROSS_LINE, 23);
        assert_eq!(DIVIDER_UPRIGHT[0], 24);
        // The groups of three: the first across the top, the others below side by side.
        let wide_top = grid_parts(Split::WideTop, 3);
        assert_eq!(
            wide_top.iter().map(GridPart::face).collect::<Vec<_>>(),
            [[2, 8, 44, 15], [2, 24, 22, 15], [25, 24, 21, 15]]
        );
        assert_eq!(
            grid_parts(Split::Columns, 4)
                .iter()
                .map(|part| part.left)
                .collect::<Vec<_>>(),
            [1, 12, 24, 35]
        );
        assert_eq!(
            grid_parts(Split::Rows, 3)
                .iter()
                .map(|part| part.top)
                .collect::<Vec<_>>(),
            [7, 18, 28]
        );
        // Three players in quarters leave the fourth without one.
        let three = grid_parts(Split::Quarters, 3);
        assert_eq!(three.len(), 4);
        assert_eq!(three[3].player, None);
        assert_eq!(three[2].player, Some(2));
    }

    #[test]
    fn every_group_picture_fits_its_box_without_overlaps() {
        for (count, split) in every_group() {
            let picture = car_picture(10, group_of(0, count, split)).unwrap();
            assert_eq!(picture.base, FIRST_ALONE, "{split:?}");
            let parts = grid_parts(split, count);
            assert_eq!(
                parts.iter().filter(|part| part.player.is_some()).count(),
                count
            );
            // Faces inside the box's face, none overlapping another.
            for (k, part) in parts.iter().enumerate() {
                let face = part.face();
                assert!(inside(face, FACE), "{split:?} {face:?}");
                assert!(face[2] > 0 && face[3] > 0);
                for other in &parts[k + 1..] {
                    assert!(apart(face, other.face()), "{split:?}");
                }
            }
            // Every piece lies in the picture, every divider on the box's frame or inside it,
            // and a divider never runs through a face.
            for piece in &picture.overlay.pieces {
                assert!(inside(piece.place, PICTURE), "{split:?} {piece:?}");
                if piece.source != Source::Own {
                    assert!(
                        inside(
                            piece.place,
                            (BOX_LEFT, BOX_TOP, BOX_RIGHT + 1, BOX_BOTTOM + 1)
                        ),
                        "{piece:?}"
                    );
                }
                if piece.rect == DIVIDER_UPRIGHT {
                    for part in &parts {
                        assert!(apart(piece.place, part.face()), "{split:?} {piece:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn group_digits_are_centred_in_their_parts_and_fit_them() {
        for (count, split) in every_group() {
            for first in [0, GROUP_MOST] {
                let picture = car_picture(11, group_of(first, count, split)).unwrap();
                let parts = grid_parts(split, count);
                let numbers = &picture.overlay.numbers;
                assert_eq!(numbers.len(), count, "{split:?}");
                let scale = numbers[0].1.scale;
                for ((number, spot), part) in numbers.iter().zip(&parts) {
                    assert_eq!(usize::from(*number), first + part.player.unwrap() + 1);
                    // One height for the whole picture, at most the pairs'.
                    assert_eq!(spot.scale, scale);
                    let height = spot.scale * FONT_FILL_HEIGHT;
                    assert!(height <= PAIR_DIGIT_HEIGHT as f32 + 1e-4);
                    assert!(height >= 6.0, "{split:?}: digits {height} high");
                    let [x, y, w, h] = part.face();
                    // Centred: across by its weight, down within half a unit, on a whole row.
                    assert!((spot.centre_x - (x as f32 + w as f32 / 2.0)).abs() < 1e-4);
                    let middle = spot.top_y + height / 2.0;
                    assert!((middle - (y as f32 + h as f32 / 2.0)).abs() <= 0.5);
                    assert_eq!(spot.top_y.fract(), 0.0);
                    // Within the face, the widest digit with its margin either side.
                    assert!(spot.top_y >= y as f32 && spot.top_y + height <= (y + h) as f32);
                    let width = spot.scale * FONT_WIDEST_DIGIT;
                    assert!(width + 2.0 * DIGIT_SIDE_MARGIN as f32 <= w as f32 + 1e-4);
                }
            }
        }
    }

    #[test]
    fn the_digit_height_of_each_split() {
        let height = |split, count| {
            group_digit_height(
                grid_parts(split, count)
                    .iter()
                    .filter(|part| part.player.is_some())
                    .map(GridPart::face),
            )
        };
        // Parts as big as a pair's take the pair's digits; narrower or lower ones smaller ones.
        assert_eq!(height(Split::Quarters, 4), PAIR_DIGIT_HEIGHT);
        assert_eq!(height(Split::Quarters, 3), PAIR_DIGIT_HEIGHT);
        assert_eq!(height(Split::WideTop, 3), PAIR_DIGIT_HEIGHT);
        assert_eq!(height(Split::Columns, 3), 9);
        assert_eq!(height(Split::Rows, 3), 7);
        assert_eq!(height(Split::Columns, 4), 6);
        assert_eq!(height(Split::Rows, 4), 6);
    }

    #[test]
    fn each_arrangement_of_a_group_has_its_own_picture() {
        use crate::race::layouts::{FIRST_GROUP_LAYOUTS, SECOND_GROUP_LAYOUTS};
        // Three players: four pictures; seven players' last three: the same four with 5, 6, 7.
        for (layouts, players, first) in [(FIRST_GROUP_LAYOUTS, 3, 0), (SECOND_GROUP_LAYOUTS, 7, 4)]
        {
            let pictures: Vec<CarPicture> = layouts
                .iter()
                .map(|&layout| car_picture(layout, shown(layout, players)).unwrap())
                .collect();
            for (picture, &split) in pictures.iter().zip(Split::many_layouts(3)) {
                assert_eq!(picture, &group(first, 3, split));
            }
            for (k, picture) in pictures.iter().enumerate() {
                assert!(pictures[k + 1..].iter().all(|other| other != picture));
            }
        }
        // Four players: three pictures, and no fourth layout.
        let four: Vec<_> = FIRST_GROUP_LAYOUTS[..3]
            .iter()
            .map(|&layout| car_picture(layout, shown(layout, 4)).unwrap())
            .collect();
        assert_ne!(four[0], four[1]);
        assert_ne!(four[1], four[2]);
        assert!(car_picture(FIRST_GROUP_LAYOUTS[3], shown(FIRST_GROUP_LAYOUTS[3], 4)).is_err());
    }

    #[test]
    fn a_group_with_no_split_is_refused() {
        assert!(car_picture(10, group_of(0, 5, Split::Quarters)).is_err());
        assert!(car_picture(10, group_of(0, 4, Split::WideTop)).is_err());
    }
}
