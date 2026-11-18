//! The touch screen in the menus, shared by the phone and tablet builds (Android and iOS) and
//! any machine with a touch screen: no platform, only fingers in and what the menus read out.
//!
//! The fingers are gestures (`Gestures`): a tap, a drag up or down (one step of the list for each
//! stretch of the screen's height), a drag sideways (a value stepped), two fingers tapped together
//! or the Back chip in the corner (back). A tap is turned into the item under it by `tapped`, from
//! the rows of the menu on show; a tap past the list's ends goes back.
//!
//! Positions are in the window's pixels (the menus' items' in the frame's: `frame_point`), times in
//! milliseconds.

/// A rectangle in pixels.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// The window and the parts of it the system keeps for itself (a notch, rounded corners, the
/// gesture bar).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Screen {
    pub width: f32,
    pub height: f32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Screen {
    pub fn new(width: f32, height: f32) -> Self {
        Screen {
            width,
            height,
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }

    /// The usable part.
    pub fn usable(&self) -> Rect {
        Rect::new(
            self.left,
            self.top,
            (self.width - self.left - self.right).max(1.0),
            (self.height - self.top - self.bottom).max(1.0),
        )
    }
}

/// A finger's change.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Down,
    Move,
    Up,
}

/// The Back chip of the menus, in the corner of the usable screen.
pub fn back_chip(screen: &Screen) -> Rect {
    let u = screen.usable();
    let s = (u.h * 0.27).max(1.0);
    Rect::new(u.x + 0.1 * s, u.y + 0.1 * s, 1.1 * s, 0.6 * s)
}

// ---- the menus ----

/// What the fingers mean in the menus.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Gesture {
    /// A tap at a point (the item there is chosen).
    Tap {
        x: f32,
        y: f32,
    },
    Up,
    Down,
    Left,
    Right,
    Back,
}

#[derive(Clone, Copy, Debug)]
struct Touch {
    id: u64,
    start: (f32, f32),
    at: (f32, f32),
    down_at: u64,
    /// The point the last step was taken from.
    anchor: (f32, f32),
    /// Which way it is dragging (`Some(true)`: sideways), once it has moved past the slop.
    sideways: Option<bool>,
}

/// The menus' recogniser.
#[derive(Debug)]
pub struct Gestures {
    touches: Vec<Touch>,
    /// Two fingers were down together since all were last up.
    pair: bool,
    /// Something moved past the slop since all were last up.
    moved: bool,
    first_down_at: u64,
    peak: usize,
}

/// A tap is shorter than this.
const TAP_MS: u64 = 450;

impl Default for Gestures {
    fn default() -> Self {
        Self::new()
    }
}

impl Gestures {
    pub const fn new() -> Self {
        Gestures {
            touches: Vec::new(),
            pair: false,
            moved: false,
            first_down_at: 0,
            peak: 0,
        }
    }

    /// A finger's change; the gestures it completes. `screen` sizes the slop and the steps.
    pub fn finger(
        &mut self,
        screen: &Screen,
        id: u64,
        phase: Phase,
        x: f32,
        y: f32,
        now: u64,
    ) -> Vec<Gesture> {
        let mut out = Vec::new();
        let unit = screen.width.min(screen.height);
        let slop = unit * 0.03;
        let step = unit * 0.11;
        match phase {
            Phase::Down => {
                if self.touches.is_empty() {
                    self.pair = false;
                    self.moved = false;
                    self.first_down_at = now;
                    self.peak = 0;
                }
                self.touches.retain(|t| t.id != id);
                self.touches.push(Touch {
                    id,
                    start: (x, y),
                    at: (x, y),
                    down_at: now,
                    anchor: (x, y),
                    sideways: None,
                });
                self.peak = self.peak.max(self.touches.len());
                if self.touches.len() >= 2 {
                    self.pair = true;
                }
            }
            Phase::Move => {
                let Some(t) = self.touches.iter_mut().find(|t| t.id == id) else {
                    return out;
                };
                t.at = (x, y);
                let (dx, dy) = (x - t.start.0, y - t.start.1);
                if t.sideways.is_none() && dx * dx + dy * dy > slop * slop {
                    t.sideways = Some(dx.abs() > dy.abs());
                    self.moved = true;
                    t.anchor = t.start;
                }
                // Only a lone finger drags.
                if self.pair {
                    return out;
                }
                match t.sideways {
                    Some(false) => {
                        // The list follows the finger: dragging up goes down the list.
                        while (y - t.anchor.1).abs() >= step {
                            let up = y < t.anchor.1;
                            out.push(if up { Gesture::Down } else { Gesture::Up });
                            t.anchor.1 += if up { -step } else { step };
                        }
                    }
                    Some(true) => {
                        let wide = step * 1.5;
                        while (x - t.anchor.0).abs() >= wide {
                            let left = x < t.anchor.0;
                            out.push(if left { Gesture::Left } else { Gesture::Right });
                            t.anchor.0 += if left { -wide } else { wide };
                        }
                    }
                    None => {}
                }
            }
            Phase::Up => {
                let Some(n) = self.touches.iter().position(|t| t.id == id) else {
                    return out;
                };
                let t = self.touches.remove(n);
                if self.touches.is_empty() && !self.moved {
                    let short = now.saturating_sub(self.first_down_at) < TAP_MS
                        && now.saturating_sub(t.down_at) < TAP_MS;
                    if short {
                        if self.pair && self.peak == 2 {
                            out.push(Gesture::Back);
                        } else if self.peak == 1 {
                            if back_chip(screen).contains(t.start.0, t.start.1) {
                                out.push(Gesture::Back);
                            } else {
                                out.push(Gesture::Tap {
                                    x: t.start.0,
                                    y: t.start.1,
                                });
                            }
                        }
                    }
                }
            }
        }
        out
    }

    pub fn release_all(&mut self) {
        self.touches.clear();
    }
}

/// One item of the menu on show that can be chosen: its number (from 1, as the menu counts) and
/// the y of the middle of its row, in the frame's pixels.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Row {
    pub number: u32,
    pub y: f32,
}

/// What a tap on the menu means.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tapped {
    /// The item under the finger: highlighted, then chosen.
    Item(u32),
    /// Past the ends of the list: back.
    Outside,
    /// The menu has no list of rows (the main menu's carousel, a screen with its own loop): the
    /// highlighted item is chosen.
    Enter,
}

/// The row a tap at `y` is on: the nearest row when it is within `reach` of it (a row's height,
/// about), else past the list's end.
pub fn tapped(rows: &[Row], y: f32, reach: f32) -> Tapped {
    let nearest = rows
        .iter()
        .map(|r| ((r.y - y).abs(), r.number))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match nearest {
        None => Tapped::Enter,
        Some((distance, number)) if distance <= reach => Tapped::Item(number),
        Some(_) => Tapped::Outside,
    }
}

/// A point of the window (pixels) as a point of the frame shown in it (pixels), the frame
/// fitted into the window with its shape kept (`gfx::screen_fit`); none outside the picture.
pub fn frame_point(point: (f32, f32), frame: (u32, u32), window: (u32, u32)) -> Option<(f32, f32)> {
    let fit = crate::gfx::screen_fit::fit(frame, window);
    if fit.width == 0 || fit.height == 0 {
        return None;
    }
    let x = (point.0 - fit.x as f32) * frame.0 as f32 / fit.width as f32;
    let y = (point.1 - fit.y as f32) * frame.1 as f32 / fit.height as f32;
    (x >= 0.0 && y >= 0.0 && x < frame.0 as f32 && y < frame.1 as f32).then_some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone() -> Screen {
        Screen {
            width: 2400.0,
            height: 1080.0,
            left: 90.0,
            top: 0.0,
            right: 90.0,
            bottom: 0.0,
        }
    }

    fn centre(r: Rect) -> (f32, f32) {
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    fn run(g: &mut Gestures, events: &[(u64, Phase, f32, f32, u64)]) -> Vec<Gesture> {
        let screen = phone();
        events
            .iter()
            .flat_map(|&(id, p, x, y, t)| g.finger(&screen, id, p, x, y, t))
            .collect()
    }

    #[test]
    fn a_tap_is_a_tap_and_a_corner_tap_is_back() {
        let mut g = Gestures::default();
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 0),
                (1, Phase::Up, 1201.0, 501.0, 100),
            ],
        );
        assert_eq!(
            got,
            vec![Gesture::Tap {
                x: 1200.0,
                y: 500.0
            }]
        );
        let back = centre(back_chip(&phone()));
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, back.0, back.1, 1000),
                (1, Phase::Up, back.0, back.1, 1050),
            ],
        );
        assert_eq!(got, vec![Gesture::Back]);
        // A long press is nothing.
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 2000),
                (1, Phase::Up, 1200.0, 500.0, 3000),
            ],
        );
        assert!(got.is_empty());
    }

    #[test]
    fn a_vertical_drag_steps_the_list_and_a_sideways_one_the_value() {
        let mut g = Gestures::default();
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 800.0, 0),
                (1, Phase::Move, 1200.0, 700.0, 10),
                (1, Phase::Move, 1200.0, 500.0, 20),
                (1, Phase::Up, 1200.0, 500.0, 30),
            ],
        );
        // Up by 300 px with steps of about 119: two steps down the list, no tap.
        assert_eq!(got, vec![Gesture::Down, Gesture::Down]);
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1000.0, 500.0, 1000),
                (1, Phase::Move, 1400.0, 505.0, 1010),
                (1, Phase::Up, 1400.0, 505.0, 1020),
            ],
        );
        assert_eq!(got, vec![Gesture::Right, Gesture::Right]);
    }

    #[test]
    fn two_fingers_tapped_together_go_back() {
        let mut g = Gestures::default();
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 900.0, 500.0, 0),
                (2, Phase::Down, 1500.0, 500.0, 20),
                (1, Phase::Up, 900.0, 500.0, 90),
                (2, Phase::Up, 1500.0, 500.0, 100),
            ],
        );
        assert_eq!(got, vec![Gesture::Back]);
    }

    #[test]
    fn a_tap_is_the_row_under_it_or_back_past_the_ends() {
        let rows = [
            Row {
                number: 1,
                y: 100.0,
            },
            Row {
                number: 2,
                y: 130.0,
            },
            Row {
                number: 3,
                y: 160.0,
            },
        ];
        assert_eq!(tapped(&rows, 128.0, 16.0), Tapped::Item(2));
        assert_eq!(tapped(&rows, 150.0, 16.0), Tapped::Item(3));
        assert_eq!(tapped(&rows, 90.0, 16.0), Tapped::Item(1));
        assert_eq!(tapped(&rows, 60.0, 16.0), Tapped::Outside);
        assert_eq!(tapped(&rows, 400.0, 16.0), Tapped::Outside);
        assert_eq!(tapped(&[], 100.0, 16.0), Tapped::Enter);
    }

    #[test]
    fn the_window_is_mapped_into_the_frame() {
        // An 800 x 600 frame in a wide window: centred, the height deciding.
        let at = |p| frame_point(p, (800, 600), (2400, 1080));
        assert_eq!(at((1200.0, 540.0)), Some((400.0, 300.0)));
        assert_eq!(at((480.0, 0.0)), Some((0.0, 0.0)));
        // The side space is outside the picture.
        assert_eq!(at((100.0, 540.0)), None);
        assert_eq!(at((2300.0, 540.0)), None);
    }
}
