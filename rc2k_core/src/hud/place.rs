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
    /// A screen no wider than 4:3: each axis by its own factor.
    Scaled {
        left: f32,
        box_width: u32,
        virtual_width: f32,
    },
    /// A wide screen: the piece keeps to an edge or the middle.
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
    /// A part narrower than 4:3: the layout's shape kept, the piece to the part's top, bottom or
    /// middle.
    Narrow { middle: f32 },
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
        let wide = anchor::is_wide(box_width, ih) && is_layout;
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
                scale: ih as f32 / 600.0,
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
        // A part narrower than 4:3 keeps the layout's shape too: one factor (the part's width
        // over 800) down as across, a piece keeping to the part's top, bottom or middle by where
        // its own middle lies in the layout.
        let narrow_part = in_part && !wide && !beside && is_layout;
        let y = if narrow_part {
            YMap::Narrow {
                middle: (y_span.0 + y_span.1) / 2.0,
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
                YMap::Narrow { middle } => anchor::narrow_y(y, middle, self.box_width, self.height),
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
