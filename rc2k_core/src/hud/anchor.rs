//! Where the HUD's 800 x 600 layout lands on a screen of another shape.
//!
//! On a screen of exactly 4:3 the layout fills it. On a wider screen the layout keeps its shape:
//! one factor for both axes (the screen's height over 600), and the pieces keep to the left edge,
//! the middle or the right edge, so a dial stays in its corner at any width, 16:9 to 32:9 and
//! beyond. One player's HUD on a narrower screen does the same, at the same size, until the
//! corner pieces would reach the car (`CLEAR_ASPECT`); narrower than that the whole layout
//! shrinks just enough to stay clear.

/// The layout's width.
pub(crate) const VIRTUAL_WIDTH: f32 = 800.0;
/// The layout's height.
pub(crate) const VIRTUAL_HEIGHT: f32 = 600.0;

/// The edge or middle a piece keeps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Left,
    Centre,
    Right,
}

/// The box of a chosen ratio the HUD keeps to, centred on the screen: the whole screen, or a
/// box that is never wider than the screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoxRatio {
    #[default]
    Full,
    FourThree,
    SixteenNine,
    TwentyOneNine,
}

impl BoxRatio {
    /// The choices, in the order the page steps through them.
    pub const ALL: [BoxRatio; 4] = [
        BoxRatio::Full,
        BoxRatio::FourThree,
        BoxRatio::SixteenNine,
        BoxRatio::TwentyOneNine,
    ];

    /// The name `rc2k.toml` keeps.
    pub fn name(self) -> &'static str {
        match self {
            BoxRatio::Full => "full",
            BoxRatio::FourThree => "4:3",
            BoxRatio::SixteenNine => "16:9",
            BoxRatio::TwentyOneNine => "21:9",
        }
    }

    /// The choice a stored name means; the whole screen for any other.
    pub fn of(name: &str) -> BoxRatio {
        BoxRatio::ALL
            .into_iter()
            .find(|r| r.name() == name.trim())
            .unwrap_or_default()
    }

    /// The next choice, either way round.
    pub fn step(self, up: bool) -> BoxRatio {
        let at = BoxRatio::ALL.iter().position(|&r| r == self).unwrap_or(0);
        let n = BoxRatio::ALL.len();
        BoxRatio::ALL[if up { (at + 1) % n } else { (at + n - 1) % n }]
    }

    /// The box on a `width` x `height` screen: how far its left edge lies from the screen's
    /// and its width.
    pub fn area(self, width: u32, height: u32) -> (u32, u32) {
        let (a, b) = match self {
            BoxRatio::Full => return (0, width),
            BoxRatio::FourThree => (4, 3),
            BoxRatio::SixteenNine => (16, 9),
            BoxRatio::TwentyOneNine => (21, 9),
        };
        let box_width = (u64::from(height) * a / b).min(u64::from(width)) as u32;
        ((width - box_width) / 2, box_width)
    }
}

/// How a screen of the 800 x 600 layout's quads (the tyre change's, the replay viewer's) lies
/// across a screen wider than 4:3. On any other screen every quad is stretched over the width,
/// as it always was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spread {
    /// Stretched over the whole width (a backdrop).
    Fill,
    /// Keeping its shape, in the 4:3 area in the middle.
    Centred,
    /// Keeping its shape, at the left edge.
    Left,
    /// Keeping its shape, at the right edge.
    Right,
    /// A strip from layout x `from` to `to` whose ends keep to the screen as the pieces beside
    /// them do (`from_edge`, `to_edge`: where a piece at that x lies, at its own shape) and whose
    /// middle stretches between them; with `keep_shape` a small piece on the strip (a mark on a
    /// time bar) lies where the strip takes its centre but keeps its own width.
    Span {
        from: u32,
        from_edge: Edge,
        to: u32,
        to_edge: Edge,
        keep_shape: bool,
    },
}

impl Spread {
    /// The factor and shift that take a layout x to the screen's x on a `width` x `height`
    /// screen: x times the factor plus the shift. The same for every spread on a screen of 4:3
    /// or narrower.
    pub fn across(self, width: u32, height: u32) -> (f64, f64) {
        let fill = f64::from(width) / f64::from(VIRTUAL_WIDTH);
        if !is_wide(width, height) {
            return (fill, 0.0);
        }
        let s = f64::from(height) / f64::from(VIRTUAL_HEIGHT);
        let spare = f64::from(width) - f64::from(VIRTUAL_WIDTH) * s;
        match self {
            Spread::Fill => (fill, 0.0),
            Spread::Centred => (s, spare / 2.0),
            Spread::Left => (s, 0.0),
            Spread::Right => (s, spare),
            Spread::Span {
                from,
                from_edge,
                to,
                to_edge,
                ..
            } => {
                let (from, to) = (f64::from(from), f64::from(to));
                if to <= from {
                    return (s, spare / 2.0);
                }
                let place = |x: f64, edge: Edge| s * x + edge.share() * spare;
                let factor = (place(to, to_edge) - place(from, from_edge)) / (to - from);
                (factor, place(from, from_edge) - factor * from)
            }
        }
    }

    /// The factor and shift for one quad lying from layout x `left` to `right`: as `across`,
    /// except that a `Span` piece that keeps its shape is moved with the strip (its centre where
    /// the strip takes it) but not widened.
    pub fn across_quad(self, width: u32, height: u32, left: f64, right: f64) -> (f64, f64) {
        let (factor, shift) = self.across(width, height);
        match self {
            Spread::Span {
                keep_shape: true, ..
            } if is_wide(width, height) => {
                let s = f64::from(height) / f64::from(VIRTUAL_HEIGHT);
                let centre = (left + right) / 2.0;
                (s, centre * factor + shift - s * centre)
            }
            _ => (factor, shift),
        }
    }
}

/// Where a layout x lies on a screen wider than 4:3 for a piece at its own shape: kept to the
/// screen's left edge, its middle or its right edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Centre,
    Right,
}

impl Edge {
    /// The share of the spare width that lies left of the 800-wide layout.
    fn share(self) -> f64 {
        match self {
            Edge::Left => 0.0,
            Edge::Centre => 0.5,
            Edge::Right => 1.0,
        }
    }
}

/// How a piece's anchor follows from where it lies in the layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// The left third keeps left, the right third right, the rest the middle.
    Thirds,
    /// The left half keeps left, the right half right (two players side by side, each with a
    /// 400-wide layout of their own).
    Halves,
    /// Everything keeps to the middle.
    Centre,
}

/// Whether the screen is wider than 4:3 (the layout then keeps its shape).
pub fn is_wide(width: u32, height: u32) -> bool {
    u64::from(width) * 3 > u64::from(height) * 4
}

/// Whether the screen (or part) is narrower than 4:3: the layout then keeps its shape (one
/// player's HUD at `layout_scale`, a part's at the width over 800), each piece keeping to the
/// top, bottom or middle (`narrow_y`).
pub fn is_narrow(width: u32, height: u32) -> bool {
    u64::from(width) * 3 < u64::from(height) * 4
}

/// How far the rev counter reaches in from the screen's left edge in the layout: its sprite
/// lies from x 33 and is 128 wide (the widest of the bottom left corner's pieces: the gear lamp
/// and the speed digits lie inside it).
pub(crate) const DIAL_REACH: f64 = 161.0;
/// How far the map reaches in from the right edge: its sprite lies from x 642 of 800.
pub(crate) const MAP_REACH: f64 = 158.0;

/// Half the car's width as a part of the screen's width in the first chase view of one player's
/// race, at the start. Measured on Langton with each of the game's 19 cars, at 800 x 600 and
/// 660 x 1088: the frame drawn with the cars and again with them left out, the car being the
/// pixels that differ (more than 24 of 255 in brightness; rows and columns holding at least 8
/// of them). The widest is the Citroen Saxo, 0.1319 of the width at 800 x 600 and 0.1303 at
/// 660 x 1088; the narrowest the Nissan Micra Maxi, 0.1013. (Two readings of 0.15 at 660 x 1088,
/// the Puma's and the 206's, were the countdown's digit changing between the two frames.) The
/// 3D's scale is made from the screen's width on a screen narrower than 4:3
/// (`scene::view_window::design_width`), so the car keeps this part of the width at every such
/// aspect: the two sizes agree. The value is the Saxo's, rounded up.
pub(crate) const CAR_HALF_WIDTH: f64 = 0.132;

/// The narrowest width over height at which one player's HUD at full size (the layout's height
/// the screen's height) keeps its bottom corner pieces clear of the car: the corner piece's
/// inner edge (its reach times the scale) meets the car's edge, half the screen less the car's
/// half width from the screen's edge. The pieces are taken as lying at the car's rows
/// throughout (they do only near this limit), so that a narrower screen never makes the HUD
/// grow again.
pub const CLEAR_ASPECT: f64 = max_reach() / (VIRTUAL_HEIGHT as f64 * (0.5 - CAR_HALF_WIDTH));

const fn max_reach() -> f64 {
    if DIAL_REACH > MAP_REACH {
        DIAL_REACH
    } else {
        MAP_REACH
    }
}

/// The size of one layout pixel for one player's HUD where it keeps its shape: the screen's
/// height over 600, but on a screen narrower than `CLEAR_ASPECT` that of such an aspect at the
/// screen's width, so the corner pieces' inner edges meet the car's.
pub fn layout_scale(width: u32, height: u32) -> f64 {
    let mut high = f64::from(height);
    if is_narrow(width, height) {
        high = high.min(f64::from(width) / CLEAR_ASPECT);
    }
    high / f64::from(VIRTUAL_HEIGHT)
}

/// The factors from the layout to the screen: across and down. Exactly 4:3 fills the screen; a
/// wider or narrower one keeps the shape (`layout_scale`; a narrow one is one player's: a split
/// screen's parts do not call this).
pub fn factors(width: u32, height: u32) -> (f64, f64) {
    if is_wide(width, height) || is_narrow(width, height) {
        let s = layout_scale(width, height);
        (s, s)
    } else {
        (
            f64::from(width) / f64::from(VIRTUAL_WIDTH),
            f64::from(height) / f64::from(VIRTUAL_HEIGHT),
        )
    }
}

/// How far the layout's left edge lies from the screen's: 0 on a screen of 4:3, negative on a
/// narrower one whose layout spills past the edges (a piece at an edge keeps to it).
pub fn offset(width: u32, height: u32, anchor: Anchor) -> f64 {
    if !is_wide(width, height) && !is_narrow(width, height) {
        return 0.0;
    }
    let spare = f64::from(width) - f64::from(VIRTUAL_WIDTH) * factors(width, height).0;
    match anchor {
        Anchor::Left => 0.0,
        Anchor::Centre => spare / 2.0,
        Anchor::Right => spare,
    }
}

/// The anchor of a piece whose middle lies at `x` in the layout.
pub(crate) fn anchor_of(x: f32, rule: Rule) -> Anchor {
    match rule {
        Rule::Centre => Anchor::Centre,
        Rule::Halves if x < VIRTUAL_WIDTH / 2.0 => Anchor::Left,
        Rule::Halves => Anchor::Right,
        Rule::Thirds if x < VIRTUAL_WIDTH / 3.0 => Anchor::Left,
        Rule::Thirds if x > VIRTUAL_WIDTH * 2.0 / 3.0 => Anchor::Right,
        Rule::Thirds => Anchor::Centre,
    }
}

/// How the pieces drawn next keep to the screen where it is wider than 4:3 (`hud::place`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    /// Each piece to the third of the layout it lies in.
    Thirds = 0,
    /// Each piece to its own half of the screen (two players side by side: each half is a part
    /// of its own, its layout kept in the shape of the part, `rc2k_core::hud::anchor::Beside`).
    Halves = 1,
    /// All to the middle.
    Middle = 2,
    /// Stretched over the whole width (the divider of two players one above the other).
    Whole = 3,
    /// The top bar over the whole width of the screen, whatever the HUD's box: its end caps and
    /// what lies on them keep to the edges, its middle stretches (`rc2k_core::hud::anchor::bar_x`).
    Bar = 4,
    /// A small piece on the top bar's middle part: its middle goes where the bar puts it and it
    /// keeps its shape.
    BarMarker = 5,
    /// Already in screen pixels (the HUD of three or four players, `hud::layout_many`).
    Pixels = 6,
    /// The middle of its own half of the screen (two players side by side), as one piece with
    /// what is drawn beside it under this rule.
    PartMiddle = 7,
}

/// Where the top bar's middle part begins and ends in the layout: the bar's two end caps, which
/// hold the clocks and names, lie left and right of it.
pub(crate) const BAR_WINDOW: (f32, f32) = (137.0, 687.0);

/// A layout x of the top bar on a screen wider than 4:3, the bar stretched over the whole
/// width: the left cap keeps to the screen's left edge and the right cap to its right edge, both
/// in the layout's shape, and the middle part takes the width between them.
pub(crate) fn bar_x(x: f32, width: u32, height: u32) -> f32 {
    let s = layout_scale(width, height);
    let (low, high) = (f64::from(BAR_WINDOW.0), f64::from(BAR_WINDOW.1));
    let x = f64::from(x);
    let right = f64::from(width) - (f64::from(VIRTUAL_WIDTH) - x) * s;
    if x <= low {
        return (x * s) as f32;
    }
    if x >= high {
        return right as f32;
    }
    let (from, to) = (
        low * s,
        f64::from(width) - (f64::from(VIRTUAL_WIDTH) - high) * s,
    );
    (from + (x - low) / (high - low) * (to - from)) as f32
}

/// Where the top bar's marker sprite puts its right edge on a screen wider than 4:3: the marker
/// stands at the bar's place for its progress, the same one the bar's parts are laid out at, so
/// it meets a part's end exactly and reaches the bar's end when the stage is done.
///
/// `left` is the sprite's left edge in the layout, which the layout puts at `scale` times the
/// progress plus `start` (`HudBlock.marker_k`); the progress then goes to the bar's own place
/// (`stage_bars::BAR_START` .. `BAR_END`) and through `bar_x`.
pub(crate) fn marker_edge_x(left: f32, scale: f32, start: f32, width: u32, height: u32) -> f32 {
    use crate::hud::stage_bars;
    let progress = (f64::from(left) - f64::from(start)) / f64::from(scale);
    let (from, to) = (
        f64::from(stage_bars::BAR_START),
        f64::from(stage_bars::BAR_END),
    );
    bar_x((from + progress * (to - from)) as f32, width, height)
}

/// A layout y on a screen or part `height` high where the layout keeps its shape at `scale` (a
/// part's width over 800; one player's `layout_scale`): a piece whose middle `mid` lies in the
/// layout's top third keeps to the top, one in the bottom third to the bottom, the others to the
/// middle.
pub(crate) fn narrow_y(y: f32, mid: f32, height: u32, scale: f64) -> f32 {
    let (y, mid) = (f64::from(y), f64::from(mid));
    let third = f64::from(VIRTUAL_HEIGHT) / 3.0;
    (if mid < third {
        y * scale
    } else if mid > 2.0 * third {
        f64::from(height) - (f64::from(VIRTUAL_HEIGHT) - y) * scale
    } else {
        f64::from(height) / 2.0 + (y - f64::from(VIRTUAL_HEIGHT) / 2.0) * scale
    }) as f32
}

/// The width of each player's layout when two players are side by side (the 800-wide layout
/// holds both, the first in its left half).
pub(crate) const PART_WIDTH: f32 = VIRTUAL_WIDTH / 2.0;

/// The size of one layout pixel of the side by side layout across a part's HUD box `box_width`
/// wide and `part_height` high: the part's height over 600 while the box is wider than the
/// 400 x 600 layout, else the layout stretched over the box's width.
fn beside_scale(box_width: u32, part_height: u32) -> f64 {
    let s = f64::from(part_height) / f64::from(VIRTUAL_HEIGHT);
    if f64::from(box_width) * f64::from(VIRTUAL_HEIGHT)
        > f64::from(part_height) * f64::from(PART_WIDTH)
    {
        s
    } else {
        f64::from(box_width) / f64::from(PART_WIDTH)
    }
}

/// How the pieces of two players side by side keep to their own halves of a screen wider than
/// 4:3: each half is a part of its own, wide as it is, and the piece whose middle lies at `mid`
/// in the layout keeps to the left edge or the right edge of its part (the left or the right
/// half of the part's 400-wide layout, inside the part's HUD box), or to its middle (`centred`,
/// for the pieces that belong together across the part's middle, the signs), the layout keeping
/// its shape as one player's does on a wide screen.
#[derive(Clone, Copy, Debug)]
pub struct Beside {
    /// The part's left edge on the screen.
    part_left: f64,
    /// The layout x the part's layout starts at.
    layout_left: f64,
    /// The part's HUD box: its left edge in the part.
    box_left: f64,
    /// A layout pixel across, and how far the piece's edge or middle lies from the box's left.
    scale: f64,
    shift: f64,
}

impl Beside {
    /// The map for the piece whose middle lies at layout x `mid` on a `width` x `height` screen.
    pub fn new(mid: f32, width: u32, height: u32, hud_box: BoxRatio, centred: bool) -> Beside {
        let second = mid >= PART_WIDTH;
        let part_width = width / 2;
        let part_left = if second { part_width } else { 0 };
        let layout_left = if second { PART_WIDTH } else { 0.0 };
        let (box_left, box_width) = hud_box.area(part_width, height);
        let scale = beside_scale(box_width, height);
        let spare = (f64::from(box_width) - f64::from(PART_WIDTH) * scale).max(0.0);
        let local = mid - layout_left;
        let shift = if centred {
            spare / 2.0
        } else if local < PART_WIDTH / 2.0 {
            0.0
        } else {
            spare
        };
        Beside {
            part_left: f64::from(part_left),
            layout_left: f64::from(layout_left),
            box_left: f64::from(box_left),
            scale,
            shift,
        }
    }

    /// A layout x on the screen.
    pub fn x(&self, x: f32) -> f32 {
        (self.part_left
            + self.box_left
            + self.shift
            + (f64::from(x) - self.layout_left) * self.scale) as f32
    }

    /// A length across the layout on the screen.
    pub fn across(&self, length: u32) -> f64 {
        f64::from(length) * self.scale
    }
}

/// A layout x on the screen.
pub fn screen_x(x: f32, width: u32, height: u32, anchor: Anchor) -> f32 {
    (f64::from(x) * factors(width, height).0 + offset(width, height, anchor)) as f32
}

/// A layout y on the screen.
pub fn screen_y(y: f32, width: u32, height: u32) -> f32 {
    (f64::from(y) * factors(width, height).1) as f32
}

/// A length across the layout on the screen, rounded as `scaled` of the strips and the mirror.
pub fn across(length: u32, width: u32, height: u32) -> Option<u32> {
    let (sx, _) = factors(width, height);
    let v = (f64::from(length) * sx).floor();
    (v >= 0.0 && v < f64::from(u32::MAX)).then_some(v as u32)
}

#[cfg(test)]
mod tests {
    #[test]
    fn narrow_and_wide_leave_4_3_between() {
        assert!(is_narrow(660, 1088) && is_narrow(1280, 1024));
        assert!(!is_narrow(800, 600) && !is_wide(800, 600));
        assert!(is_wide(1280, 720) && !is_narrow(1280, 720));
    }

    use super::*;

    #[test]
    fn the_limit_aspect_follows_from_the_measurements() {
        // 161 / (600 x (0.5 - 0.132)): the dial's reach over the room left of the car, per
        // layout row.
        assert!((CLEAR_ASPECT - 0.7292).abs() < 1e-3, "{CLEAR_ASPECT}");
        assert!(CLEAR_ASPECT < 0.75 && is_narrow(1, 1));
    }

    #[test]
    fn a_narrow_screen_down_to_the_limit_keeps_the_normal_size() {
        // 4:3 itself, and narrower: the height over 600, whatever the width.
        for (w, h) in [
            (800, 600),
            (760, 600),
            (700, 600),
            (640, 600),
            (600, 700),
            (450, 600),
        ] {
            assert_eq!(layout_scale(w, h), f64::from(h) / 600.0, "{w} x {h}");
        }
        assert_eq!(factors(700, 600), (1.0, 1.0));
        // The corner pieces keep to the edges: the layout's right edge at the screen's.
        assert_eq!(screen_x(0.0, 700, 600, Anchor::Left), 0.0);
        assert_eq!(screen_x(800.0, 700, 600, Anchor::Right), 700.0);
        assert_eq!(screen_x(642.0, 700, 600, Anchor::Right), 542.0);
        assert_eq!(screen_x(400.0, 700, 600, Anchor::Centre), 350.0);
        // 4:3 itself is unchanged.
        assert_eq!(factors(800, 600), (1.0, 1.0));
        assert_eq!(offset(800, 600, Anchor::Right), 0.0);
    }

    #[test]
    fn a_screen_narrower_than_the_limit_shrinks_to_keep_clear_of_the_car() {
        // 660 x 1088: the scale of the limit aspect at the width (660 over 0.7292 x 600).
        let s = layout_scale(660, 1088);
        assert!((s - 660.0 / (CLEAR_ASPECT * 600.0)).abs() < 1e-12);
        assert!(s < 1088.0 / 600.0 && s > 825.0 / 1000.0);
        // The dial's inner edge meets the car's left edge (the car's half width either side of
        // the middle).
        let dial = DIAL_REACH * s;
        assert!(
            (dial - 660.0 * (0.5 - CAR_HALF_WIDTH)).abs() < 1e-9,
            "{dial}"
        );
        // Continuous at the limit: nearly the same scale just either side of it.
        let limit = (600.0 * CLEAR_ASPECT) as u32;
        let (a, b) = (layout_scale(limit - 1, 600), layout_scale(limit + 1, 600));
        assert!((a - b).abs() < 0.01 && b <= 1.0, "{a} {b}");
    }

    #[test]
    fn the_corner_pieces_stay_clear_of_the_car_at_every_narrow_shape() {
        for h in [480u32, 600, 720, 1088, 2000] {
            for w in (200..=(h * 4 / 3)).step_by(37) {
                let s = layout_scale(w, h);
                let room = f64::from(w) * (0.5 - CAR_HALF_WIDTH);
                assert!(s <= f64::from(h) / 600.0 + 1e-12, "{w} x {h}");
                assert!(DIAL_REACH * s <= room + 1e-9, "{w} x {h}");
                assert!(MAP_REACH * s <= room + 1e-9, "{w} x {h}");
            }
        }
    }

    #[test]
    fn the_bar_spans_a_narrow_screen_too() {
        assert_eq!(bar_x(0.0, 700, 600), 0.0);
        assert_eq!(bar_x(800.0, 700, 600), 700.0);
        // 660 x 1088: the caps keep their (shrunk) shape at the edges.
        let s = layout_scale(660, 1088);
        assert!((f64::from(bar_x(137.0, 660, 1088)) - 137.0 * s).abs() < 1e-3);
    }

    #[test]
    fn a_narrow_part_keeps_the_layouts_shape() {
        // A 400 x 800 part: one factor of 0.5 down as across.
        assert_eq!(narrow_y(20.0, 20.0, 800, 0.5), 10.0);
        assert_eq!(narrow_y(600.0, 560.0, 800, 0.5), 800.0);
        assert_eq!(narrow_y(500.0, 540.0, 800, 0.5), 800.0 - 50.0);
        assert_eq!(narrow_y(300.0, 300.0, 800, 0.5), 400.0);
    }

    #[test]
    fn narrow_screens_stretch_as_before() {
        assert!(!is_wide(800, 600) && !is_wide(1280, 1024));
        assert_eq!(factors(1024, 768), (1.28, 1.28));
        assert_eq!(screen_x(100.0, 1024, 768, Anchor::Right), 128.0);
    }

    #[test]
    fn wide_screens_keep_the_shape_and_the_corners() {
        assert!(is_wide(1920, 1080));
        assert_eq!(factors(1920, 1080), (1.8, 1.8));
        assert_eq!(screen_x(0.0, 1920, 1080, Anchor::Left), 0.0);
        assert_eq!(screen_x(800.0, 1920, 1080, Anchor::Right), 1920.0);
        assert_eq!(screen_x(0.0, 1920, 1080, Anchor::Centre), 240.0);
        assert_eq!(screen_x(800.0, 3840, 1080, Anchor::Right), 3840.0);
        assert_eq!(screen_y(600.0, 3840, 1080), 1080.0);
    }

    #[test]
    fn the_box_is_never_wider_than_the_screen() {
        assert_eq!(BoxRatio::Full.area(3840, 1080), (0, 3840));
        assert_eq!(BoxRatio::SixteenNine.area(3840, 1080), (960, 1920));
        assert_eq!(BoxRatio::FourThree.area(3840, 1080), (1200, 1440));
        assert_eq!(BoxRatio::TwentyOneNine.area(1920, 1080), (0, 1920));
        assert_eq!(BoxRatio::SixteenNine.area(1024, 768), (0, 1024));
        assert_eq!(BoxRatio::of("16:9").step(true), BoxRatio::TwentyOneNine);
        assert_eq!(BoxRatio::of("x"), BoxRatio::Full);
    }

    #[test]
    fn spreads_keep_narrow_screens_and_place_wide_ones() {
        for spread in [Spread::Fill, Spread::Centred, Spread::Left, Spread::Right] {
            assert_eq!(spread.across(1024, 768), (1.28, 0.0));
        }
        assert_eq!(Spread::Fill.across(3840, 1080), (4.8, 0.0));
        assert_eq!(Spread::Centred.across(3840, 1080), (1.8, 1200.0));
        assert_eq!(Spread::Left.across(3840, 1080), (1.8, 0.0));
        assert_eq!(Spread::Right.across(3840, 1080), (1.8, 2400.0));
    }

    #[test]
    fn the_top_bar_reaches_both_edges() {
        assert_eq!(bar_x(0.0, 3840, 1080), 0.0);
        assert_eq!(bar_x(800.0, 3840, 1080), 3840.0);
        // The caps keep their shape at the edges.
        assert!((bar_x(137.0, 3840, 1080) - 246.6).abs() < 0.01);
        assert!((bar_x(687.0, 3840, 1080) - (3840.0 - 203.4)).abs() < 0.01);
        // The middle is stretched between them, continuously.
        let (a, b) = (bar_x(137.0, 3840, 1080), bar_x(687.0, 3840, 1080));
        assert!((bar_x(412.0, 3840, 1080) - (a + b) / 2.0).abs() < 0.01);
    }

    /// The marker's right edge follows the progress along the same places as the bar's parts, at
    /// every screen width.
    #[test]
    fn the_marker_follows_the_bars_parts() {
        use crate::hud::stage_bars::{BAR_END, BAR_START, WIDTH, place};
        // The layout's own numbers: the left edge at 550 times the progress plus 119.
        let left = |progress: f32| 119.0 + 550.0 * progress;
        for (w, h) in [(1920u32, 1080u32), (3440, 1440), (3840, 1080), (5120, 1440)] {
            // The start and the finished stage: the bar's ends.
            assert!(
                (marker_edge_x(left(0.0), 550.0, 119.0, w, h) - bar_x(BAR_START as f32, w, h))
                    .abs()
                    < 0.01
            );
            assert!(
                (marker_edge_x(left(1.0), 550.0, 119.0, w, h) - bar_x(BAR_END as f32, w, h)).abs()
                    < 0.01
            );
            // At a part's end the marker is within a pixel of the line the part's quads draw (the
            // line lies between the part's last column and the next part's first).
            for (miles, total) in [(1.0f32, 4.0f32), (2.5, 4.0), (3.0, 4.0), (7.3, 9.1)] {
                let line = f64::from(place(WIDTH, miles, total)) + 136.0;
                let s = f64::from(h) / 600.0;
                let at = f64::from(marker_edge_x(left(miles / total), 550.0, 119.0, w, h));
                let want = f64::from(bar_x(line as f32, w, h));
                assert!(
                    (at - want).abs() <= 1.0 * (f64::from(w) - 800.0 * s) / 550.0 + s,
                    "{w} {miles}: {at} {want}"
                );
            }
        }
    }

    #[test]
    fn side_by_side_pieces_keep_to_their_own_halves() {
        // 32:9 at 1080 high: each half is 1920 x 1080, a layout pixel 1.8, the half's layout 720
        // wide with 1200 spare.
        let at = |x: f32, mid: f32| Beside::new(mid, 3840, 1080, BoxRatio::Full, false).x(x);
        assert_eq!(at(0.0, 10.0), 0.0);
        assert!((at(100.0, 100.0) - 180.0).abs() < 1e-3);
        assert!((at(300.0, 300.0) - (1200.0 + 540.0)).abs() < 1e-3);
        // A centred piece lies across the part's middle.
        let centred = Beside::new(100.0, 3840, 1080, BoxRatio::Full, true);
        assert!((centred.x(100.0) - (600.0 + 180.0)).abs() < 1e-3);
        assert!((at(400.0, 390.0) - 1920.0).abs() < 1e-3);
        // The second player's layout starts at its half's left edge and ends at the screen's.
        assert_eq!(at(400.0, 410.0), 1920.0);
        assert!((at(800.0, 790.0) - 3840.0).abs() < 1e-3);
        assert!((at(500.0, 500.0) - (1920.0 + 180.0)).abs() < 1e-3);
        // The HUD box 4:3 keeps to the middle of the half.
        let boxed = Beside::new(10.0, 3840, 1080, BoxRatio::FourThree, false);
        assert!((boxed.x(0.0) - 240.0).abs() < 1e-3);
        // A half of 4:3's own shape stretches the layout over it.
        let narrow = Beside::new(10.0, 1024, 768, BoxRatio::Full, false);
        assert_eq!(narrow.x(400.0), 512.0);
        assert_eq!(narrow.across(100), 128.0);
    }

    #[test]
    fn anchors_follow_the_place() {
        assert_eq!(anchor_of(50.0, Rule::Thirds), Anchor::Left);
        assert_eq!(anchor_of(400.0, Rule::Thirds), Anchor::Centre);
        assert_eq!(anchor_of(700.0, Rule::Thirds), Anchor::Right);
        assert_eq!(anchor_of(399.0, Rule::Halves), Anchor::Left);
        assert_eq!(anchor_of(400.0, Rule::Halves), Anchor::Right);
        assert_eq!(anchor_of(10.0, Rule::Centre), Anchor::Centre);
    }

    #[test]
    fn a_span_keeps_its_ends_beside_the_edge_pieces() {
        // 32:9 at 1080 high: the layout's 800 x 600 is 1440 wide, 2400 spare.
        let (w, h) = (3840, 1080);
        let span = Spread::Span {
            from: 100,
            from_edge: Edge::Left,
            to: 700,
            to_edge: Edge::Right,
            keep_shape: false,
        };
        let (f, k) = span.across(w, h);
        let (lf, lk) = Spread::Left.across(w, h);
        let (rf, rk) = Spread::Right.across(w, h);
        assert!((100.0 * f + k - (100.0 * lf + lk)).abs() < 1e-6);
        assert!((700.0 * f + k - (700.0 * rf + rk)).abs() < 1e-6);
        // A mark at 400 keeps its width (1.8 a layout pixel) and lies on the strip.
        let mark = Spread::Span {
            from: 100,
            from_edge: Edge::Left,
            to: 700,
            to_edge: Edge::Right,
            keep_shape: true,
        };
        let (mf, mk) = mark.across_quad(w, h, 395.0, 405.0);
        assert!((mf - 1.8).abs() < 1e-9);
        assert!((400.0 * mf + mk - (400.0 * f + k)).abs() < 1e-6);
        // A strip from the left edge to the middle ends where a centred piece lies.
        let half = Spread::Span {
            from: 0,
            from_edge: Edge::Left,
            to: 400,
            to_edge: Edge::Centre,
            keep_shape: false,
        };
        let (hf, hk) = half.across(w, h);
        let (cf, ck) = Spread::Centred.across(w, h);
        assert!((400.0 * hf + hk - (400.0 * cf + ck)).abs() < 1e-6);
        // 4:3: every spread stretches as before.
        assert_eq!(span.across(1600, 1200), Spread::Fill.across(1600, 1200));
    }
}
