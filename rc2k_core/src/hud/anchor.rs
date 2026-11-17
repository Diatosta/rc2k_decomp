//! Where the HUD's 800 x 600 layout lands on a screen of another shape.
//!
//! On a screen as wide as 4:3 or narrower the layout fills it, each axis scaled by its own
//! factor, as it always did. On a wider screen the layout keeps its shape: one factor for both
//! axes (the screen's height over 600), and the pieces keep to the left edge, the middle or the
//! right edge, so a dial stays in its corner at any width, 16:9 to 32:9 and beyond.

/// The layout's width.
pub const VIRTUAL_WIDTH: f32 = 800.0;
/// The layout's height.
pub const VIRTUAL_HEIGHT: f32 = 600.0;

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
    /// A strip from layout x `from` to `to` whose ends keep to the screen's edges as the pieces
    /// beside it do (`Left` / `Right`) and whose middle stretches between them; with `keep_shape`
    /// a small piece on the strip (a mark on a time bar) lies where the strip takes its left
    /// edge but keeps its own width.
    Span {
        from: u32,
        to: u32,
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
            Spread::Span { from, to, .. } => {
                let (from, to) = (f64::from(from), f64::from(to));
                if to <= from {
                    return (s, spare / 2.0);
                }
                let factor = (s * (to - from) + spare) / (to - from);
                (factor, s * from - factor * from)
            }
        }
    }

    /// The factor and shift for one quad whose leftmost layout x is `left`: as `across`, except
    /// that a `Span` piece that keeps its shape is moved with the strip but not widened.
    pub fn across_quad(self, width: u32, height: u32, left: f64) -> (f64, f64) {
        let (factor, shift) = self.across(width, height);
        match self {
            Spread::Span {
                keep_shape: true, ..
            } if is_wide(width, height) => {
                let s = f64::from(height) / f64::from(VIRTUAL_HEIGHT);
                (s, left * factor + shift - s * left)
            }
            _ => (factor, shift),
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

/// The factors from the layout to the screen: across and down.
pub fn factors(width: u32, height: u32) -> (f64, f64) {
    if is_wide(width, height) {
        let s = f64::from(height) / f64::from(VIRTUAL_HEIGHT);
        (s, s)
    } else {
        (
            f64::from(width) / f64::from(VIRTUAL_WIDTH),
            f64::from(height) / f64::from(VIRTUAL_HEIGHT),
        )
    }
}

/// How far the layout's left edge lies from the screen's: 0 unless the screen is wider than 4:3.
pub fn offset(width: u32, height: u32, anchor: Anchor) -> f64 {
    if !is_wide(width, height) {
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
pub fn anchor_of(x: f32, rule: Rule) -> Anchor {
    match rule {
        Rule::Centre => Anchor::Centre,
        Rule::Halves if x < VIRTUAL_WIDTH / 2.0 => Anchor::Left,
        Rule::Halves => Anchor::Right,
        Rule::Thirds if x < VIRTUAL_WIDTH / 3.0 => Anchor::Left,
        Rule::Thirds if x > VIRTUAL_WIDTH * 2.0 / 3.0 => Anchor::Right,
        Rule::Thirds => Anchor::Centre,
    }
}

/// Where the top bar's middle part begins and ends in the layout: the bar's two end caps, which
/// hold the clocks and names, lie left and right of it.
pub const BAR_WINDOW: (f32, f32) = (137.0, 687.0);

/// A layout x of the top bar on a screen wider than 4:3, the bar stretched over the whole
/// width: the left cap keeps to the screen's left edge and the right cap to its right edge, both
/// in the layout's shape, and the middle part takes the width between them.
pub fn bar_x(x: f32, width: u32, height: u32) -> f32 {
    let s = f64::from(height) / f64::from(VIRTUAL_HEIGHT);
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
    use super::*;

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
            to: 700,
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
            to: 700,
            keep_shape: true,
        };
        let (mf, mk) = mark.across_quad(w, h, 400.0);
        assert!((mf - 1.8).abs() < 1e-9);
        assert!((400.0 * mf + mk - (400.0 * f + k)).abs() < 1e-6);
        // 4:3: every spread stretches as before.
        assert_eq!(span.across(1600, 1200), Spread::Fill.across(1600, 1200));
    }
}
