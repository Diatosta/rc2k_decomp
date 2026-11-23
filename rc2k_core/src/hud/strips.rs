//! The players' view strips: a view block's strip is shown while its flash is on, as three flat
//! quads side by side (the banner a view shows while a notice of its car is shown, `notice_shown` /
//! the notice's time: the strip flashes with the notice's period) scaled from 800 x 600 to the
//! screen; then two more at the top corners.

/// Each strip's texture: the first player's, the second's.
pub const TEXTURES: [u32; 2] = [0xC4, 0xC5];
/// What is added to the players' number for the second player's view.
pub const SECOND_VIEW: u32 = 3;

/// The flash's division.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flash {
    pub quotient: u32,
    pub remainder: u32,
    pub half: u32,
}

impl Flash {
    /// Whether the strip is on: the remainder below half the period.
    pub fn on(self) -> bool {
        self.remainder < self.half
    }
}

/// A block's strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strip {
    /// Not shown: the block's hide flag is set or its strip value is 0.
    Hidden,
    /// Shown without a flash: the period is at most 0 (signed).
    Steady,
    /// Flashing: the clock over the period (unsigned).
    Flashing(Flash),
}

impl Strip {
    pub fn shown(self) -> bool {
        match self {
            Strip::Hidden => false,
            Strip::Steady => true,
            Strip::Flashing(f) => f.on(),
        }
    }
}

/// A view block's strip from its `covered`, `shown`, `clock` and
/// `period` fields.
pub fn strip(covered: u32, shown: u32, clock: u32, period: u32) -> Strip {
    if covered != 0 || shown == 0 {
        return Strip::Hidden;
    }
    if (period as i32) <= 0 {
        return Strip::Steady;
    }
    Strip::Flashing(Flash {
        quotient: clock / period,
        remainder: clock % period,
        half: period >> 1,
    })
}

/// The views (`Strips.tops[0]`: twelve tops, the second six when the flat-layout switch is 0;
/// six lefts).
pub const VIEWS: u32 = 6;
/// The strip's height and each quad's width on the 800 x 600 screen.
pub const HEIGHT: u32 = 0x19;
pub const WIDTH: u32 = 0x78;
/// How far each quad's texture rows move on.
pub const ROWS_STEP: u32 = 0x32_0000;
/// The virtual screen: width and height.
pub(crate) const VIRTUAL: [u32; 2] = [0x320, 0x258];

/// The top's entry of a view (below `VIEWS`): the second six when the flat-layout switch is 0.
pub fn top_entry(view: u32, flat: u32) -> usize {
    (view + if flat == 0 { VIEWS } else { 0 }) as usize
}

/// A size scaled to the screen: `mul` by the screen's size, `div` by the virtual one
/// (unsigned, 64 bits between): the quotient and the remainder.
pub fn scaled(v: u32, screen: u32, of: u32) -> (u32, u32) {
    let n = u64::from(v) * u64::from(screen);
    ((n / u64::from(of)) as u32, (n % u64::from(of)) as u32)
}

/// The strip's sizes on the screen (top, height, left, width: `Strips.sizes[0]` ..) from the
/// virtual ones, and what the last division leaves: the width, its remainder and the
/// virtual width.
pub fn sizes(
    v: [u32; 4],
    screen_width: u32,
    height: u32,
    rule: crate::hud::anchor::Rule,
    hud_box: crate::hud::anchor::BoxRatio,
) -> ([u32; 4], [u32; 3]) {
    use crate::hud::anchor;
    if rule == anchor::Rule::Halves && anchor::is_wide(screen_width, height) {
        // Two players side by side: the strip keeps to its own half of the screen, centred on it
        // (the strip is a notice for the whole part, not an item at the part's edge).
        let map = anchor::Beside::new(
            v[2] as f32 + v[3] as f32 / 2.0,
            screen_width,
            height,
            hud_box,
            true,
        );
        let down =
            |n: u32| (f64::from(n) * f64::from(height) / f64::from(VIRTUAL[1])).floor() as u32;
        let w = map.across(v[3]).floor() as u32;
        let left = f64::from(map.x(v[2] as f32)).floor() as u32;
        return ([down(v[0]), down(v[1]), left, w], [w, 0, VIRTUAL[0]]);
    }
    // The strip keeps to the HUD's box, centred on the screen.
    let (box_left, width) = hud_box.area(screen_width, height);
    if anchor::is_wide(width, height) {
        // The layout keeps its shape: the strip's size and corner by one factor, the corner
        // from the edge or the middle its place goes with.
        let (sx, sy) = anchor::factors(width, height);
        let middle = v[2] as f32 + v[3] as f32 / 2.0;
        let at = anchor::offset(width, height, anchor::anchor_of(middle, rule));
        let across = |n: u32| (f64::from(n) * sx).floor() as u32;
        let down = |n: u32| (f64::from(n) * sy).floor() as u32;
        let left = (f64::from(box_left) + at + f64::from(v[2]) * sx).floor() as u32;
        let w = across(v[3]);
        return ([down(v[0]), down(v[1]), left, w], [w, 0, VIRTUAL[0]]);
    }
    if anchor::is_narrow(width, height) {
        // Narrower than 4:3: the layout keeps its shape, one factor (the width over 800) down as
        // across, the strip keeping to the top, bottom or middle by where it lies.
        let s = f64::from(width) / f64::from(anchor::VIRTUAL_WIDTH);
        let mid = v[0] as f32 + v[1] as f32 / 2.0;
        let top = anchor::narrow_y(v[0] as f32, mid, width, height).floor() as u32;
        let across = |n: u32| (f64::from(n) * s).floor() as u32;
        let w = across(v[3]);
        return (
            [top, across(v[1]), box_left + across(v[2]), w],
            [w, 0, VIRTUAL[0]],
        );
    }
    let [(top, _), (h, _), (left, _), (w, r)] = [
        scaled(v[0], height, VIRTUAL[1]),
        scaled(v[1], height, VIRTUAL[1]),
        scaled(v[2], width, VIRTUAL[0]),
        scaled(v[3], width, VIRTUAL[0]),
    ];
    ([top, h, box_left + left, w], [w, r, VIRTUAL[0]])
}

/// A flat quad's inputs (`RaceVars.quad_colour[0]` ..: (offset, value) in the order written) at x
/// `left` of width `width`, the sizes' top and height, texture rows from
/// `rows`: the colour -1 last.
pub fn quad_inputs(left: u32, width: u32, sizes: &[u32; 4], rows: u32) -> Vec<(usize, u32)> {
    let mut out = vec![
        (0x28, left),
        (0x2C, left.wrapping_add(width)),
        (0x38, sizes[0]),
        (0x3C, sizes[0].wrapping_add(sizes[1])),
    ];
    for (n, v) in [0x8_0000u32, 0xF8_0000, 0xF8_0000, 0x8_0000]
        .into_iter()
        .enumerate()
    {
        out.push((8 + 4 * n, v));
    }
    let next = rows.wrapping_add(ROWS_STEP);
    out.extend([(0x18, rows), (0x1C, rows), (0x20, next), (0x24, next)]);
    out
}

/// The tail's quads (drawn by `hud::draw_view_strips` too): shown with the strips' bit 0x40 and
/// `RaceVars.split` (the players) 0.
pub fn tail_shown(shown: u32, players: u32) -> bool {
    shown & 0x40 != 0 && players == 0
}

/// The tail's two quads: (the virtual top, height, left, width), the texture rows.
pub const TAIL: [([u32; 4], u32); 2] = [
    ([2, 0xF, 4, 0x44], 0x96_0000),
    ([2, 0xF, 0x2B4, 0x44], 0xC8_0000),
];
/// The tail's texture.
pub const TAIL_TEXTURE: u32 = 0xC4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips() {
        let block = |fa8: u32, on: u32, clock: u32, period: u32| strip(fa8, on, clock, period);
        assert_eq!(block(1, 1, 0, 0), Strip::Hidden);
        assert_eq!(block(0, 0, 0, 0), Strip::Hidden);
        assert_eq!(block(0, 1, 5, 0x8000_0000), Strip::Steady);
        let s = block(0, 1, 23, 10);
        assert_eq!(
            s,
            Strip::Flashing(Flash {
                quotient: 2,
                remainder: 3,
                half: 5
            })
        );
        assert!(s.shown());
        assert!(!block(0, 1, 27, 10).shown());
    }

    #[test]
    fn side_by_side_strips_keep_to_their_halves() {
        use crate::hud::anchor::{BoxRatio, Rule};
        // 32:9 at 1080 high: the second player's strip, laid out at 0x1AA, lies in the right half.
        let (s, _) = sizes(
            [2, HEIGHT, 0x1AA, WIDTH],
            3840,
            1080,
            Rule::Halves,
            BoxRatio::Full,
        );
        assert!(s[2] >= 1920 && s[2] + s[3] <= 3840);
        // The first player's lies in the left half, at the 1.8 factor of the half's height, and
        // its three quads, laid out across the half's middle, are centred on the half.
        let (s, _) = sizes(
            [2, HEIGHT, 0x14, WIDTH],
            3840,
            1080,
            Rule::Halves,
            BoxRatio::Full,
        );
        assert_eq!(s[3], 216);
        assert_eq!(s[2] + 3 * s[3] / 2, 960);
        // The second player's, likewise in the right half.
        let (s, _) = sizes(
            [2, HEIGHT, 0x1A4, WIDTH],
            3840,
            1080,
            Rule::Halves,
            BoxRatio::Full,
        );
        assert_eq!(s[2] + 3 * s[3] / 2, 2880);
        // 4:3 stretches as before.
        let (s, _) = sizes(
            [2, HEIGHT, 4, WIDTH],
            1024,
            768,
            Rule::Halves,
            BoxRatio::Full,
        );
        assert_eq!((s[2], s[3]), (5, 0x99));
    }

    #[test]
    fn a_portrait_strip_keeps_its_shape() {
        use crate::hud::anchor::{BoxRatio, Rule};
        // 660 x 1088: one factor (660 / 800) across and down, the strip at the top.
        let (s, _) = sizes(
            [2, HEIGHT, 0, WIDTH],
            660,
            1088,
            Rule::Thirds,
            BoxRatio::Full,
        );
        assert_eq!((s[1], s[3]), (HEIGHT * 660 / 800, WIDTH * 660 / 800));
        assert!(s[0] < 8, "the strip keeps to the top: {s:?}");
    }

    #[test]
    fn view_strip() {
        assert_eq!(top_entry(2, 0), 8);
        assert_eq!(top_entry(5, 1), 5);
        assert_eq!(scaled(0x78, 1024, 0x320), (0x99, 480));
        let (s, regs) = sizes(
            [10, HEIGHT, 20, WIDTH],
            1024,
            768,
            crate::hud::anchor::Rule::Thirds,
            crate::hud::anchor::BoxRatio::Full,
        );
        assert_eq!(s, [12, 32, 25, 0x99]);
        assert_eq!(regs, [0x99, 480, 0x320]);
        let q = quad_inputs(25, 0x99, &s, 0x32_0000);
        assert_eq!(q.len(), 12);
        assert_eq!(q[1], (0x2C, 25 + 0x99));
        assert_eq!(q[3], (0x3C, 44));
        assert_eq!(q[11], (0x24, 0x64_0000));
        assert!(tail_shown(0x40, 0) && !tail_shown(0x40, 1) && !tail_shown(0x3F, 0));
    }
}
