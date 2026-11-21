//! The pages' tinted backdrops (bottom panel, fade overlay, two-column panel, column rules
//! and boxes): each a list of quads drawn by `scene::draw_tinted_quad`, in order.

/// A tinted quad: its place, size, colour and see-through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quad {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub colour: u32,
    pub see: u32,
}

/// The pages' see-through blue.
pub const BLUE: u32 = 0x11166A;
const GREY: u32 = 0x1F1F1F;
const BOX: u32 = 0x4FCF;
const YELLOW: u32 = 0xFF_FF00;

fn q(x: u32, y: u32, w: u32, h: u32, colour: u32, see: u32) -> Quad {
    Quad {
        x,
        y,
        w,
        h,
        colour,
        see,
    }
}

/// A page's backdrop: the page above 0x1E3 and the bottom bar round its middle, the
/// middle's strip grey.
pub fn bottom_panel(see: u32) -> [Quad; 5] {
    [
        q(0, 0, 0x320, 0x1E3, BLUE, see),
        q(0, 0x1E3, 0xED, 0x75, BLUE, see),
        q(0xED, 0x1F6, 0x233, 0x62, BLUE, see),
        q(0x233, 0x1E3, 0xED, 0x13, BLUE, see),
        q(0x128, 0x1E2, 0xD2, 0x15, GREY, see),
    ]
}

/// The fades' quad over the whole 800 x 600 screen.
pub fn fade_overlay(see: u32) -> Quad {
    q(0, 0, 0x320, 0x258, BLUE, see)
}

/// A two-column page's top bar, side bars, bottom bar and middle rule (then its rules and
/// boxes).
pub fn two_column_panel(see: u32) -> [Quad; 5] {
    [
        q(0, 0, 0x320, 0x45, BLUE, see),
        q(0, 0x45, 0x43, 0x258, BLUE, see),
        q(0x2DE, 0x45, 0x42, 0x213, BLUE, see),
        q(0x43, 0x215, 0x29B, 0x43, BLUE, see),
        q(0x18F, 0x45, 3, 0x1D0, BLUE, see),
    ]
}

/// The two columns' x.
pub const COLUMNS: [u32; 2] = [0x43, 0x192];
/// The rows' first y and their step.
pub(crate) const RULES_Y: u32 = 0x5F;
pub const BOXES_Y: u32 = 0x45;
pub const ROW_STEP: u32 = 0x1D;
/// The rows in a column.
pub const ROWS: u32 = 0x10;

/// The rules between the sixteen rows of each column.
pub fn column_rules(see: u32) -> Vec<Quad> {
    COLUMNS
        .iter()
        .flat_map(|&x| (0..ROWS).map(move |n| q(x, RULES_Y + n * ROW_STEP, 0x14C, 3, BLUE, see)))
        .collect()
}

const BORDER: [(i32, i32, u32, u32); 4] = [
    (0, 0, 0x14C, 1),
    (0, 0, 1, 0x1A),
    (0x14C, 0, 1, 0x1B),
    (0, 0x1A, 0x14C, 1),
];
const OUTER: [(i32, i32, u32, u32); 4] = [
    (-1, -1, 0x14E, 1),
    (-1, -1, 1, 0x1C),
    (0x14D, -1, 1, 0x1D),
    (-1, 0x1B, 0x14E, 1),
];

/// A row's box (numbered 1 .. 32, at x, y): the box, its black border, and for the
/// `chosen` row its yellow border and one a pixel outside it.
pub fn row_box(x: u32, y: u32, see: u32, chosen: bool) -> Vec<Quad> {
    let at = |d: i32, m: u32| m.wrapping_add(d as u32);
    let mut out = vec![q(x, y, 0x14D, 0x1B, BOX, see)];
    out.extend(
        BORDER
            .iter()
            .map(|&(dx, dy, w, h)| q(at(dx, x), at(dy, y), w, h, 0, 0xFF)),
    );
    if chosen {
        out.extend(
            BORDER
                .iter()
                .chain(&OUTER)
                .map(|&(dx, dy, w, h)| q(at(dx, x), at(dy, y), w, h, YELLOW, 0xFF)),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels() {
        let r = column_rules(9);
        assert_eq!((r.len(), r[15].y, r[16].x), (32, 0x5F + 15 * 0x1D, 0x192));
        assert_eq!(
            (
                row_box(10, 20, 1, false).len(),
                row_box(10, 20, 1, true).len()
            ),
            (5, 13)
        );
        assert_eq!(row_box(10, 20, 1, true)[9].x, 9);
    }
}
