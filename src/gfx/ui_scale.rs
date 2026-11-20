//! The page-sized panels the program draws itself (the render dialog and its progress, the
//! notices and questions) with the game's fonts, which have one pixel size: they are laid out on
//! a virtual screen of about 600 lines and enlarged to the real one when it is taller, so they
//! take the same share of a 1080-line, a 1440-line or a phone's screen as of an 800 x 600 one.
//! Nothing changes at 600 lines or fewer: the layout is drawn as it is.

use crate::gfx::scaled_region::enlarge;
use crate::owners::Owners;

/// The height of the virtual screen a panel is laid out for.
const BASE_HEIGHT: i32 = 600;

/// The virtual screen of one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// Real pixels to a virtual one (1 at `BASE_HEIGHT` lines or fewer).
    pub factor: f32,
    /// The virtual screen's size: what the layout is made on.
    pub width: i32,
    pub height: i32,
    /// The real screen's size.
    pub screen: (i32, i32),
}

impl View {
    /// The virtual screen for a real one of `w` x `h`.
    pub fn of(w: i32, h: i32) -> Self {
        let factor = (h as f32 / BASE_HEIGHT as f32).max(1.0);
        Self {
            factor,
            width: ((w as f32 / factor) as i32).max(1),
            height: ((h as f32 / factor) as i32).max(1),
            screen: (w, h),
        }
    }

    /// The virtual screen for the game's screen now.
    pub fn now(o: Owners<'static>) -> Self {
        let screen = o.system.session.screen.get();
        Self::of(screen.width.get() as i32, screen.height.get() as i32)
    }

    /// The panel of `w` x `h` virtual pixels: laid out at `Panel::origin`, centred on the real
    /// screen at its enlarged size.
    pub fn panel(&self, w: i32, h: i32) -> Panel {
        let scaled = self.factor > 1.0;
        let (dw, dh) = if scaled {
            (
                (w as f32 * self.factor).round() as i32,
                (h as f32 * self.factor).round() as i32,
            )
        } else {
            (w, h)
        };
        let dest = [(self.screen.0 - dw) / 2, (self.screen.1 - dh) / 2, dw, dh];
        Panel {
            factor: self.factor,
            scaled,
            // Drawn where it will be when it is not enlarged, else in the top left corner.
            source: if scaled { [0, 0, w, h] } else { dest },
            dest,
        }
    }
}

/// One panel of a frame: drawn on its source rectangle, enlarged to its destination.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Panel {
    factor: f32,
    scaled: bool,
    source: [i32; 4],
    dest: [i32; 4],
}

impl Panel {
    /// The top left corner the panel is laid out from.
    pub fn origin(&self) -> (i32, i32) {
        (self.source[0], self.source[1])
    }

    /// A real screen position (the mouse) as a position in the panel's layout.
    pub fn point(&self, x: i32, y: i32) -> (i32, i32) {
        if !self.scaled {
            return (x, y);
        }
        (
            ((x - self.dest[0]) as f32 / self.factor).floor() as i32,
            ((y - self.dest[1]) as f32 / self.factor).floor() as i32,
        )
    }

    /// Ends the frame, with the surface locked: the panel drawn at the corner is enlarged into
    /// its place, and what is left of the corner outside it is painted `background` again.
    pub fn finish(&self, o: Owners<'static>, background: u32) {
        if !self.scaled {
            return;
        }
        let [dx, dy, dw, dh] = self.dest;
        enlarge(o.gfx, self.source, (dx, dy), (dw, dh));
        for rect in outside(self.source, self.dest) {
            crate::input::osk_draw::fill(o, rect, background);
        }
    }
}

/// The parts of rectangle `a` (x, y, width, height) that are not in `b`, as rectangles.
fn outside(a: [i32; 4], b: [i32; 4]) -> Vec<[i32; 4]> {
    let (left, top) = (a[0].max(b[0]), a[1].max(b[1]));
    let (right, bottom) = (
        (a[0] + a[2]).min(b[0] + b[2]),
        (a[1] + a[3]).min(b[1] + b[3]),
    );
    if left >= right || top >= bottom {
        return vec![a];
    }
    [
        [a[0], a[1], a[2], top - a[1]],
        [a[0], bottom, a[2], a[1] + a[3] - bottom],
        [a[0], top, left - a[0], bottom - top],
        [right, top, a[0] + a[2] - right, bottom - top],
    ]
    .into_iter()
    .filter(|r| r[2] > 0 && r[3] > 0)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_screen_is_drawn_as_it_is() {
        let v = View::of(800, 600);
        assert_eq!((v.factor, v.width, v.height), (1.0, 800, 600));
        assert_eq!(View::of(640, 480).height, 480);
    }

    #[test]
    fn a_tall_screen_gets_about_600_lines() {
        for (w, h) in [(1920, 1080), (2560, 1440), (5120, 1440), (2556, 1179)] {
            let v = View::of(w, h);
            assert!((595..=600).contains(&v.height), "{w}x{h}: {v:?}");
            assert!(v.width * 100 / v.height >= w * 100 / h - 2);
        }
        assert_eq!(View::of(1920, 1080).width, 1066);
    }

    #[test]
    fn the_mouse_follows_the_enlarged_panel() {
        let v = View::of(1920, 1080);
        let p = v.panel(720, 500);
        assert_eq!(p.origin(), (0, 0));
        let [dx, dy, dw, dh] = p.dest;
        assert_eq!((dw, dh), (1296, 900));
        assert_eq!(p.point(dx, dy), (0, 0));
        assert_eq!(p.point(dx + dw - 1, dy + dh - 1), (719, 499));
        // Not enlarged: the panel is where it is drawn, and the mouse is as it is.
        let small = View::of(800, 600).panel(720, 500);
        assert_eq!(small.origin(), (40, 50));
        assert_eq!(small.point(41, 51), (41, 51));
    }

    #[test]
    fn the_corner_left_over_is_what_the_enlarged_panel_does_not_cover() {
        let covered = |rects: &[[i32; 4]], x: i32, y: i32| {
            rects
                .iter()
                .any(|r| x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3])
        };
        for b in [
            [100, 50, 300, 200],
            [-20, -10, 60, 40],
            [30, 20, 10, 10],
            [500, 500, 5, 5],
        ] {
            let a = [0, 0, 120, 90];
            let rects = outside(a, b);
            for y in -15..110 {
                for x in -25..140 {
                    let in_a = covered(&[a], x, y);
                    let in_b = covered(&[b], x, y);
                    assert_eq!(covered(&rects, x, y), in_a && !in_b, "{b:?} at {x},{y}");
                }
            }
        }
    }
}
