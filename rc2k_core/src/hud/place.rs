//! Where a polygon's vertices land on the screen: which of the layout's mappings (`anchor`)
//! applies to the pieces drawn under the layout's current `Keep` rule, on the screen or part
//! being drawn into. The caller copies the vertices through `Placement::x` and `Placement::y`.

use super::anchor::{self, BoxRatio, Keep};
use super::quads::screen_scaled;

/// The layout's own size, which the mappings below are made for.
const LAYOUT: (f32, f32) = (anchor::VIRTUAL_WIDTH, anchor::VIRTUAL_HEIGHT);

/// What decides the mapping.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    /// The current `Keep` rule as stored (a value no `Keep` has is passed over like a
    /// third-keeping one on a wide screen).
    pub rule: u32,
    /// The width of the screen, or of the part, being drawn into.
    pub width: u32,
    /// The usable area (x, y, width, height) inside the screen or part, relative to its corner;
    /// all of it for the rules that stretch over the whole of it.
    pub inset: [u32; 4],
    /// The box the HUD keeps to.
    pub hud_box: BoxRatio,
    /// The virtual screen the vertices are given in (the layout's, or a window's pixels).
    pub virtual_screen: (f32, f32),
    /// The marker's scale and start along the top bar (`anchor::marker_edge_x`).
    pub marker_k: [f32; 2],
    /// The lowest and highest x of the polygon's vertices.
    pub x_span: (f32, f32),
    /// The lowest and highest y of the polygon's vertices.
    pub y_span: (f32, f32),
    /// Drawing into a part of the screen.
    pub in_part: bool,
    /// One player's HUD on the whole screen (`hud::batch::Layout::Single`): on a screen
    /// narrower than 4:3 it keeps to the edges at the full size `anchor::layout_scale` gives.
    pub one_player: bool,
}

/// The mapping of the vertices' x.
#[derive(Clone, Copy, Debug)]
enum XMap {
    /// Two players side by side: each half's own layout.
    Beside { left: f32, map: anchor::Beside },
    /// Stretched over the whole width.
    Whole { width: f32 },
    /// The top bar: its caps to the edges, its middle stretched.
    Bar {
        left: f32,
        box_width: u32,
        height: u32,
    },
    /// The marker: its shape kept, its right edge at `edge`.
    Marker { edge: f32, scale: f32, high: f32 },
    /// A screen of 4:3, or a narrower one with a split screen's layout: each axis by its own
    /// factor.
    Scaled {
        left: f32,
        box_width: u32,
        virtual_width: f32,
    },
    /// A wide screen, or one player's on a narrow one: the piece keeps to an edge or the middle.
    Anchored {
        left: f32,
        box_width: u32,
        height: u32,
        at: anchor::Anchor,
    },
}

/// The mapping of the vertices' y.
#[derive(Clone, Copy, Debug)]
enum YMap {
    /// The layout's shape kept at `scale` on a screen or part narrower than 4:3 (a part's width
    /// over 800; one player's `anchor::layout_scale`), the piece to the top, bottom or middle.
    Narrow { middle: f32, scale: f64 },
    /// The layout's shape kept across a wide screen.
    Shape,
    /// Each axis by its own factor.
    Scaled,
}

/// A polygon's mapping from the layout to the screen.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    x: XMap,
    y: YMap,
    /// The y origin.
    top: u32,
    box_width: u32,
    height: u32,
    virtual_height: f32,
    keeps_pixels: bool,
}

impl Placement {
    /// The mapping for the polygon `place` describes.
    pub fn of(place: &Place) -> Placement {
        let Place {
            rule,
            width: w,
            inset: [ox, oy, iw, ih],
            hud_box,
            virtual_screen: (kx, ky),
            marker_k,
            x_span: (low, high),
            y_span,
            in_part,
            one_player,
        } = *place;
        // The pieces keep to the HUD's box, centred on the screen (all of it by default); the
        // divider's own rule stretches over the whole screen.
        let (box_left, box_width) = if rule >= 3 {
            (0, w)
        } else {
            let (left, width) = hud_box.area(iw, ih);
            (ox + left, width)
        };
        let is_layout = (kx, ky) == LAYOUT;
        let narrow = anchor::is_narrow(box_width, ih) && is_layout;
        // One player's HUD on a narrow screen keeps its shape and its edges as on a wide one.
        let one_narrow = narrow && one_player && !in_part;
        let wide = (anchor::is_wide(box_width, ih) && is_layout) || one_narrow;
        // Two players side by side on a wide screen: each half is a part of its own.
        let beside = (rule == Keep::Halves as u32 || rule == Keep::PartMiddle as u32)
            && anchor::is_wide(iw, ih)
            && is_layout;
        let middle = (low + high) / 2.0;
        let x = if beside {
            XMap::Beside {
                left: ox as f32,
                map: anchor::Beside::new(middle, iw, ih, hud_box, rule == Keep::PartMiddle as u32),
            }
        } else if wide && rule == Keep::Whole as u32 {
            XMap::Whole { width: w as f32 }
        } else if wide && rule == Keep::Bar as u32 {
            XMap::Bar {
                left: ox as f32,
                box_width,
                height: ih,
            }
        } else if wide && rule == Keep::BarMarker as u32 {
            // The marker keeps its shape and puts its right edge at the bar's place for its
            // progress, where the bar's parts end.
            XMap::Marker {
                edge: ox as f32
                    + anchor::marker_edge_x(low, marker_k[0], marker_k[1], box_width, ih),
                scale: anchor::layout_scale(box_width, ih) as f32,
                high,
            }
        } else if !wide {
            XMap::Scaled {
                left: box_left as f32,
                box_width,
                virtual_width: kx,
            }
        } else {
            let rule = match rule {
                1 => anchor::Rule::Halves,
                2 => anchor::Rule::Centre,
                _ => anchor::Rule::Thirds,
            };
            XMap::Anchored {
                left: box_left as f32,
                box_width,
                height: ih,
                at: anchor::anchor_of(middle, rule),
            }
        };
        // A part, or a split screen, narrower than 4:3 keeps the layout's shape too: one factor
        // (its width over 800) down as across, a piece keeping to the top, bottom or middle by
        // where its own middle lies in the layout. A whole screen of exactly 4:3 is the layout
        // itself.
        let middle_y = (y_span.0 + y_span.1) / 2.0;
        let y = if one_narrow {
            YMap::Narrow {
                middle: middle_y,
                scale: anchor::layout_scale(box_width, ih),
            }
        } else if is_layout && (in_part || narrow) && !wide && !beside {
            YMap::Narrow {
                middle: middle_y,
                scale: f64::from(box_width) / f64::from(anchor::VIRTUAL_WIDTH),
            }
        } else if wide || beside {
            YMap::Shape
        } else {
            YMap::Scaled
        };
        Placement {
            x,
            y,
            top: oy,
            box_width,
            height: ih,
            virtual_height: ky,
            keeps_pixels: rule == Keep::Pixels as u32,
        }
    }

    /// Whether the pieces are already in screen pixels and are left where they are.
    pub fn keeps_pixels(&self) -> bool {
        self.keeps_pixels
    }

    /// The screen x of a vertex at layout `x`.
    pub fn x(&self, x: f32) -> f32 {
        match self.x {
            XMap::Beside { left, map } => left + map.x(x),
            XMap::Whole { width } => x * width / 800.0,
            XMap::Bar {
                left,
                box_width,
                height,
            } => left + anchor::bar_x(x, box_width, height),
            XMap::Marker { edge, scale, high } => edge + (x - high) * scale,
            XMap::Scaled {
                left,
                box_width,
                virtual_width,
            } => left + screen_scaled(x, box_width, virtual_width),
            XMap::Anchored {
                left,
                box_width,
                height,
                at,
            } => left + anchor::screen_x(x, box_width, height, at),
        }
    }

    /// The screen y of a vertex at layout `y`.
    pub fn y(&self, y: f32) -> f32 {
        self.top as f32
            + match self.y {
                YMap::Narrow { middle, scale } => anchor::narrow_y(y, middle, self.height, scale),
                YMap::Shape => anchor::screen_y(y, self.box_width, self.height),
                YMap::Scaled => screen_scaled(y, self.height, self.virtual_height),
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(rule: Keep, width: u32, height: u32) -> Place {
        Place {
            rule: rule as u32,
            width,
            inset: [0, 0, width, height],
            hud_box: BoxRatio::Full,
            virtual_screen: LAYOUT,
            marker_k: [1.0, 0.0],
            x_span: (0.0, 100.0),
            y_span: (0.0, 50.0),
            in_part: false,
            one_player: false,
        }
    }

    #[test]
    fn a_four_by_three_screen_is_filled_by_each_axis_alone() {
        let p = Placement::of(&place(Keep::Thirds, 1600, 1200));
        assert_eq!((p.x(400.0), p.y(300.0)), (800.0, 600.0));
    }

    #[test]
    fn a_wide_screen_keeps_the_shape_and_the_corner() {
        let p = Placement::of(&place(Keep::Thirds, 1920, 1080));
        // The piece in the left third keeps to the left edge, the layout's height by 1.8.
        assert_eq!((p.x(0.0), p.x(100.0), p.y(300.0)), (0.0, 180.0, 540.0));
        let mut right = place(Keep::Thirds, 1920, 1080);
        right.x_span = (700.0, 800.0);
        assert_eq!(Placement::of(&right).x(800.0), 1920.0);
    }

    #[test]
    fn a_split_screen_in_portrait_keeps_the_shape_and_the_top_and_bottom() {
        // 660 x 1088: one factor of 0.825 down as across; the top piece at the top, the bottom
        // piece at the bottom, the middle one about the middle.
        let mut top = place(Keep::Thirds, 660, 1088);
        top.y_span = (0.0, 100.0);
        assert_eq!(Placement::of(&top).y(100.0), 82.5);
        let mut bottom = place(Keep::Thirds, 660, 1088);
        bottom.y_span = (500.0, 600.0);
        let p = Placement::of(&bottom);
        assert_eq!((p.y(600.0), p.y(500.0)), (1088.0, 1088.0 - 82.5));
        let mut mid = place(Keep::Thirds, 660, 1088);
        mid.y_span = (250.0, 350.0);
        let p = Placement::of(&mid);
        assert_eq!((p.x(400.0), p.y(300.0)), (330.0, 544.0));
    }

    fn single(width: u32, height: u32) -> Place {
        Place {
            one_player: true,
            ..place(Keep::Thirds, width, height)
        }
    }

    #[test]
    fn one_player_on_a_slightly_narrow_screen_keeps_the_normal_size_at_the_edges() {
        // 700 x 600: the layout's size at 4:3 for the height (one factor of 1), the left piece
        // at the left edge, the right piece at the right edge, rows as at 4:3.
        let mut left = single(700, 600);
        left.x_span = (30.0, 160.0);
        left.y_span = (440.0, 568.0);
        let p = Placement::of(&left);
        assert_eq!(
            (p.x(33.0), p.x(161.0), p.y(440.0), p.y(568.0)),
            (33.0, 161.0, 440.0, 568.0)
        );
        let mut right = single(700, 600);
        right.x_span = (640.0, 770.0);
        right.y_span = (440.0, 568.0);
        let p = Placement::of(&right);
        // The map's inner edge 158 from the right edge, as at 800 x 600.
        assert_eq!((p.x(642.0), p.x(768.0), p.y(568.0)), (542.0, 668.0, 568.0));
        // The 4:3 screen itself is the layout.
        let mut right = single(800, 600);
        right.x_span = (640.0, 770.0);
        assert_eq!(Placement::of(&right).x(642.0), 642.0);
    }

    #[test]
    fn one_player_on_a_portrait_screen_shrinks_to_clear_the_car_and_keeps_the_corners() {
        use crate::hud::anchor::{CAR_HALF_WIDTH, CLEAR_ASPECT, DIAL_REACH};
        let s = 660.0 / (CLEAR_ASPECT * 600.0);
        // The bottom left piece (the dial): at the left edge, its bottom at the screen's bottom.
        let mut dial = single(660, 1088);
        dial.x_span = (33.0, 161.0);
        dial.y_span = (440.0, 568.0);
        let p = Placement::of(&dial);
        assert!((f64::from(p.x(0.0))).abs() < 1e-3);
        let edge = f64::from(p.x(161.0));
        assert!((edge - DIAL_REACH * s).abs() < 1e-2);
        assert!(edge <= 660.0 * (0.5 - CAR_HALF_WIDTH) + 1e-2, "{edge}");
        // Its rows: the layout's bottom third keeps to the bottom edge.
        assert!((f64::from(p.y(600.0)) - 1088.0).abs() < 1e-3);
        // The map at the right edge, clear of the car too.
        let mut map = single(660, 1088);
        map.x_span = (642.0, 768.0);
        map.y_span = (440.0, 568.0);
        let p = Placement::of(&map);
        assert!((f64::from(p.x(800.0)) - 660.0).abs() < 1e-3);
        assert!(f64::from(p.x(642.0)) >= 660.0 * (0.5 + CAR_HALF_WIDTH) - 1e-2);
        // A top piece keeps to the top, a middle one to the middle.
        let mut top = single(660, 1088);
        top.y_span = (0.0, 100.0);
        assert!((f64::from(Placement::of(&top).y(100.0)) - 100.0 * s).abs() < 1e-3);
        let mut mid = single(660, 1088);
        mid.x_span = (350.0, 450.0);
        mid.y_span = (250.0, 350.0);
        let p = Placement::of(&mid);
        assert_eq!((p.x(400.0), p.y(300.0)), (330.0, 544.0));
    }

    #[test]
    fn the_divider_stretches_over_the_whole_width() {
        let p = Placement::of(&place(Keep::Whole, 1920, 1080));
        assert_eq!(p.x(400.0), 960.0);
    }

    #[test]
    fn pieces_in_pixels_stay() {
        assert!(Placement::of(&place(Keep::Pixels, 1920, 1080)).keeps_pixels());
        assert!(!Placement::of(&place(Keep::Thirds, 1920, 1080)).keeps_pixels());
    }
}
