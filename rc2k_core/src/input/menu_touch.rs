//! The touch screen in the menus, shared by the phone and tablet builds (Android and iOS) and
//! any machine with a touch screen: no platform, only fingers in and what the menus read out.
//!
//! The fingers are gestures (`Gestures`): a tap, a drag up or down (one step of the list for each
//! stretch of the screen's height), a drag sideways (a value stepped) and a double tap (back).
//! A tap is Enter on what is highlighted, wherever the finger was; it is held for the double-tap
//! window (`DOUBLE_MS`) so that the first tap of a double tap never acts. Nothing else acts: no
//! item under the finger is chosen and a tap beside a list does not go back.
//!
//! Positions are in the window's pixels, times in milliseconds.

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

// ---- the menus ----

/// What the fingers mean in the menus.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Gesture {
    /// A single tap, once it is known not to be the first of a double tap: Enter. The point is
    /// for a page that has items of its own (the installer's).
    Tap {
        x: f32,
        y: f32,
    },
    Up,
    Down,
    Left,
    Right,
    /// A double tap.
    Back,
}

#[derive(Clone, Copy, Debug)]
struct Touch {
    id: u64,
    start: (f32, f32),
    at: (f32, f32),
    down_at: u64,
    /// When the finger last went down or moved: a finger silent for `STALE_MS` is gone.
    last_at: u64,
    /// The point the last step was taken from.
    anchor: (f32, f32),
    /// Which way it is dragging (`Some(true)`: sideways), once it has moved past the slop.
    sideways: Option<bool>,
}

/// The menus' recogniser.
#[derive(Debug)]
pub struct Gestures {
    touches: Vec<Touch>,
    /// Two fingers were down together since all were last up (they do nothing).
    pair: bool,
    /// A tap that waits to see whether a second one follows: where, and when it lifted.
    pending: Option<(f32, f32, u64)>,
    /// Something moved past the slop since all were last up.
    moved: bool,
    /// A drag gave a step (a move) since all were last up.
    stepped: bool,
    first_down_at: u64,
    peak: usize,
    /// A sideways drag gives the key on the finger's side (a drag to the right is Right) instead
    /// of the carousel's way (a drag to the left brings the next one).
    direct_sideways: bool,
    /// A page that acts on a tap itself (the installer's, the port's dialogs): a single tap is
    /// given the moment the finger lifts, not after the double-tap window.
    immediate: bool,
    /// Fingers dropped by `drop_stale` since `take_stale` last asked.
    stale: usize,
}

/// A finger with no event for this long is taken as lifted whose lifting was never seen: the
/// menus have no use for a finger held still, and one left in the list would turn the next
/// finger into a pair, which taps nothing.
pub const STALE_MS: u64 = 1500;

/// A tap is shorter than this.
const TAP_MS: u64 = 450;

/// The second tap of a double tap comes within this after the first lifted, and within
/// `DOUBLE_REACH` (of the screen's short side) of it. A single tap acts after this wait.
pub const DOUBLE_MS: u64 = 200;
const DOUBLE_REACH: f32 = 0.08;
/// How far a finger may slide during a tap, as a share of the screen's shorter side: a finger
/// tapping on a phone rolls a little, and a slide this short that made no step is still a tap.
const TAP_REACH: f32 = 0.06;

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
            pending: None,
            moved: false,
            stepped: false,
            first_down_at: 0,
            peak: 0,
            direct_sideways: false,
            immediate: false,
            stale: 0,
        }
    }

    /// Drops the fingers silent for `STALE_MS` (their lifting never came); how many.
    pub fn drop_stale(&mut self, now: u64) -> usize {
        let before = self.touches.len();
        self.touches
            .retain(|t| now.saturating_sub(t.last_at) < STALE_MS);
        let dropped = before - self.touches.len();
        self.stale += dropped;
        dropped
    }

    /// How many fingers `drop_stale` dropped since the last call.
    pub fn take_stale(&mut self) -> usize {
        std::mem::take(&mut self.stale)
    }

    /// A finger gone without a gesture (its lifting could not be given to `finger`).
    pub fn forget(&mut self, id: u64) {
        self.touches.retain(|t| t.id != id);
    }

    /// Pages that act on a tap at once (on the finger's lifting). A tap the page did not use
    /// (`tap_used`) still starts a double tap, whose second tap is Back.
    pub fn set_immediate(&mut self, on: bool) {
        self.immediate = on;
    }

    /// The page acted on the last tap (it was on a button): it is not the first of a double
    /// tap, so the next tap is a tap of its own.
    pub fn tap_used(&mut self) {
        self.pending = None;
    }

    /// Sideways drags on a page that is not a carousel: the key follows the finger's side.
    pub fn set_direct_sideways(&mut self, on: bool) {
        self.direct_sideways = on;
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
                self.drop_stale(now);
                if self.touches.is_empty() {
                    self.pair = false;
                    self.moved = false;
                    self.stepped = false;
                    self.first_down_at = now;
                    self.peak = 0;
                }
                self.touches.retain(|t| t.id != id);
                self.touches.push(Touch {
                    id,
                    start: (x, y),
                    at: (x, y),
                    down_at: now,
                    last_at: now,
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
                t.last_at = now;
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
                        // Dragging up goes up the list (the phone's own feel: a swipe pushes the
                        // selection the way the finger moves).
                        while (y - t.anchor.1).abs() >= step {
                            let up = y < t.anchor.1;
                            out.push(if up { Gesture::Up } else { Gesture::Down });
                            self.stepped = true;
                            t.anchor.1 += if up { -step } else { step };
                        }
                    }
                    Some(true) => {
                        let wide = step * 1.5;
                        while (x - t.anchor.0).abs() >= wide {
                            // The value follows the finger as a phone's carousel does: a drag
                            // to the left brings the next one (right), to the right the one before.
                            let left = x < t.anchor.0;
                            self.stepped = true;
                            out.push(if left != self.direct_sideways {
                                Gesture::Right
                            } else {
                                Gesture::Left
                            });
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
                let (dx, dy) = (x - t.start.0, y - t.start.1);
                let reach = TAP_REACH * unit;
                let still = !self.moved || (!self.stepped && dx * dx + dy * dy <= reach * reach);
                if self.touches.is_empty() && still {
                    let short = now.saturating_sub(self.first_down_at) < TAP_MS
                        && now.saturating_sub(t.down_at) < TAP_MS;
                    if short && self.peak == 1 {
                        let near = DOUBLE_REACH * unit;
                        match self.pending.take() {
                            Some((px, py, at))
                                if t.down_at.saturating_sub(at) < DOUBLE_MS
                                    && (px - t.start.0).powi(2) + (py - t.start.1).powi(2)
                                        <= near * near =>
                            {
                                out.push(Gesture::Back);
                            }
                            _ => {
                                self.pending = Some((t.start.0, t.start.1, now));
                                if self.immediate {
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
        }
        out
    }

    /// The single tap whose wait is over at `now`, if any.
    pub fn tick(&mut self, now: u64) -> Vec<Gesture> {
        match self.pending {
            Some((x, y, at)) if now.saturating_sub(at) >= DOUBLE_MS => {
                self.pending = None;
                if self.immediate {
                    // Given when it lifted.
                    return Vec::new();
                }
                vec![Gesture::Tap { x, y }]
            }
            _ => Vec::new(),
        }
    }

    /// Whether a tap waits for its double-tap window.
    pub fn waiting(&self) -> bool {
        self.pending.is_some()
    }

    /// Whether any finger is down.
    pub fn active(&self) -> bool {
        !self.touches.is_empty()
    }

    pub fn release_all(&mut self) {
        self.touches.clear();
        self.pending = None;
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

    fn run(g: &mut Gestures, events: &[(u64, Phase, f32, f32, u64)]) -> Vec<Gesture> {
        let screen = phone();
        events
            .iter()
            .flat_map(|&(id, p, x, y, t)| g.finger(&screen, id, p, x, y, t))
            .collect()
    }

    fn tick(g: &mut Gestures, now: u64) -> Vec<Gesture> {
        g.tick(now)
    }

    #[test]
    fn a_tap_that_rolls_a_little_is_still_a_tap_but_a_short_drag_is_not() {
        // The screen's shorter side is 1080: the slop is 32 pixels, a tap may slide 64.
        let mut g = Gestures::default();
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 0),
                (1, Phase::Move, 1240.0, 520.0, 40),
                (1, Phase::Up, 1245.0, 525.0, 100),
            ],
        );
        assert!(got.is_empty());
        assert_eq!(
            tick(&mut g, 100 + DOUBLE_MS),
            vec![Gesture::Tap {
                x: 1200.0,
                y: 500.0
            }]
        );
        let mut g = Gestures::default();
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 1000),
                (1, Phase::Move, 1270.0, 500.0, 1040),
                (1, Phase::Up, 1275.0, 500.0, 1100),
            ],
        );
        assert!(got.is_empty());
        assert!(!g.waiting());
        assert!(tick(&mut g, 1100 + DOUBLE_MS).is_empty());
    }

    #[test]
    fn a_single_tap_acts_once_its_wait_is_over_and_a_long_press_is_nothing() {
        let mut g = Gestures::default();
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 0),
                (1, Phase::Up, 1201.0, 501.0, 100),
            ],
        );
        // Held back: it could be the first of a double tap.
        assert!(got.is_empty());
        assert!(g.waiting());
        assert!(tick(&mut g, 100 + DOUBLE_MS - 1).is_empty());
        assert_eq!(
            tick(&mut g, 100 + DOUBLE_MS),
            vec![Gesture::Tap {
                x: 1200.0,
                y: 500.0
            }]
        );
        assert!(tick(&mut g, 5000).is_empty());
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 2000),
                (1, Phase::Up, 1200.0, 500.0, 3000),
            ],
        );
        assert!(got.is_empty());
        assert!(tick(&mut g, 9000).is_empty());
    }

    /// Milliseconds from the finger's lifting to the tap being given, with the clock ticking
    /// every 8 ms (a frame's turn of the event loop).
    fn delay_of_a_tap(immediate: bool) -> u64 {
        let mut g = Gestures::default();
        g.set_immediate(immediate);
        let lift = 100;
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 0),
                (1, Phase::Up, 1200.0, 500.0, lift),
            ],
        );
        if !got.is_empty() {
            return 0;
        }
        let mut now = lift;
        loop {
            now += 8;
            if !tick(&mut g, now).is_empty() {
                return now - lift;
            }
            assert!(now < 5000, "the tap never came");
        }
    }

    #[test]
    fn a_tap_on_a_page_that_acts_on_it_comes_at_once_where_the_menus_wait() {
        // The menus: the tap is felt after the double-tap window (200 ms, 8 ms steps: 200).
        assert_eq!(delay_of_a_tap(false), 200);
        // The pages' own buttons: at the lift.
        assert_eq!(delay_of_a_tap(true), 0);
    }

    #[test]
    fn on_an_immediate_page_a_double_tap_off_a_button_is_back_and_on_one_two_taps() {
        let mut g = Gestures::default();
        g.set_immediate(true);
        let first = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 0),
                (1, Phase::Up, 1200.0, 500.0, 60),
            ],
        );
        assert_eq!(
            first,
            vec![Gesture::Tap {
                x: 1200.0,
                y: 500.0
            }]
        );
        // Not used: the second tap is Back.
        let second = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 150),
                (1, Phase::Up, 1200.0, 500.0, 190),
            ],
        );
        assert_eq!(second, vec![Gesture::Back]);
        // Used by a button: the second tap is a tap of its own.
        let first = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 1000),
                (1, Phase::Up, 1200.0, 500.0, 1040),
            ],
        );
        assert_eq!(first.len(), 1);
        g.tap_used();
        let second = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 1100),
                (1, Phase::Up, 1200.0, 500.0, 1140),
            ],
        );
        assert_eq!(
            second,
            vec![Gesture::Tap {
                x: 1200.0,
                y: 500.0
            }]
        );
        // Left alone, nothing comes later.
        assert!(tick(&mut g, 5000).is_empty());
    }

    #[test]
    fn a_tap_in_a_corner_is_only_a_tap() {
        let mut g = Gestures::default();
        run(
            &mut g,
            &[
                (1, Phase::Down, 100.0, 60.0, 0),
                (1, Phase::Up, 100.0, 60.0, 50),
            ],
        );
        assert_eq!(tick(&mut g, 400), vec![Gesture::Tap { x: 100.0, y: 60.0 }]);
    }

    #[test]
    fn a_double_tap_goes_back_and_its_first_tap_never_acts() {
        let mut g = Gestures::default();
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 0),
                (1, Phase::Up, 1200.0, 500.0, 60),
                (1, Phase::Down, 1210.0, 505.0, 200),
                (1, Phase::Up, 1210.0, 505.0, 260),
            ],
        );
        assert_eq!(got, vec![Gesture::Back]);
        assert!(!g.waiting());
        assert!(tick(&mut g, 5000).is_empty());
        // A third tap right after is a new first tap.
        run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 300),
                (1, Phase::Up, 1200.0, 500.0, 340),
            ],
        );
        assert!(g.waiting());
    }

    #[test]
    fn two_taps_too_late_or_too_far_apart_are_two_single_taps() {
        let mut g = Gestures::default();
        run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 0),
                (1, Phase::Up, 1200.0, 500.0, 60),
            ],
        );
        // The wait ends before the second tap lands.
        assert_eq!(tick(&mut g, 400).len(), 1);
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 450),
                (1, Phase::Up, 1200.0, 500.0, 500),
            ],
        );
        assert!(got.is_empty());
        assert_eq!(tick(&mut g, 800).len(), 1);
        // Far apart (past the reach): the first is dropped for the second, never Back.
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 600.0, 500.0, 1000),
                (1, Phase::Up, 600.0, 500.0, 1050),
                (1, Phase::Down, 1800.0, 500.0, 1150),
                (1, Phase::Up, 1800.0, 500.0, 1200),
            ],
        );
        assert!(got.is_empty());
        assert_eq!(
            tick(&mut g, 1600),
            vec![Gesture::Tap {
                x: 1800.0,
                y: 500.0
            }]
        );
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
        // Up by 300 px with steps of about 119: two steps up the list, no tap.
        assert_eq!(got, vec![Gesture::Up, Gesture::Up]);
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1000.0, 500.0, 1000),
                (1, Phase::Move, 1400.0, 505.0, 1010),
                (1, Phase::Up, 1400.0, 505.0, 1020),
            ],
        );
        // A drag to the right brings the one before: the item follows the finger.
        assert_eq!(got, vec![Gesture::Left, Gesture::Left]);
        let got = run(
            &mut g,
            &[
                (1, Phase::Down, 1400.0, 500.0, 2000),
                (1, Phase::Move, 1000.0, 505.0, 2010),
                (1, Phase::Up, 1000.0, 505.0, 2020),
            ],
        );
        assert_eq!(got, vec![Gesture::Right, Gesture::Right]);
    }

    #[test]
    fn a_direct_sideways_drag_gives_the_key_on_the_fingers_side() {
        let mut g = Gestures::default();
        g.set_direct_sideways(true);
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
    fn a_finger_released_by_a_screen_change_taps_nothing() {
        let mut g = Gestures::default();
        let mut got = run(&mut g, &[(1, Phase::Down, 1200.0, 500.0, 0)]);
        assert!(g.active());
        // The screen changed: what the platform does with the fingers down.
        g.release_all();
        assert!(!g.active());
        got.extend(run(&mut g, &[(1, Phase::Up, 1200.0, 500.0, 60)]));
        assert!(got.is_empty());
        // A tap waiting for its window is dropped as well.
        run(
            &mut g,
            &[
                (1, Phase::Down, 1200.0, 500.0, 100),
                (1, Phase::Up, 1200.0, 500.0, 150),
            ],
        );
        g.release_all();
        assert!(tick(&mut g, 1000).is_empty());
    }

    #[test]
    fn a_finger_whose_lifting_never_came_does_not_turn_the_next_tap_into_a_pair() {
        let mut g = Gestures::default();
        run(&mut g, &[(1, Phase::Down, 900.0, 500.0, 0)]);
        // Its lifting is lost: a tap long after is still a tap.
        let got = run(
            &mut g,
            &[
                (2, Phase::Down, 1200.0, 500.0, 5000),
                (2, Phase::Up, 1200.0, 500.0, 5060),
            ],
        );
        assert!(got.is_empty());
        assert_eq!(g.take_stale(), 1);
        assert_eq!(
            tick(&mut g, 5060 + DOUBLE_MS),
            vec![Gesture::Tap {
                x: 1200.0,
                y: 500.0
            }]
        );
    }

    #[test]
    fn a_stale_finger_is_dropped_at_the_tick_and_a_moving_one_is_kept() {
        let mut g = Gestures::default();
        run(
            &mut g,
            &[
                (1, Phase::Down, 900.0, 500.0, 0),
                (2, Phase::Down, 1500.0, 500.0, 0),
                (2, Phase::Move, 1500.0, 520.0, 1400),
            ],
        );
        assert_eq!(g.drop_stale(STALE_MS - 1), 0);
        assert_eq!(g.drop_stale(STALE_MS), 1);
        assert!(g.active());
        assert_eq!(g.drop_stale(1400 + STALE_MS), 1);
        assert!(!g.active());
    }

    #[test]
    fn a_forgotten_finger_leaves_the_list() {
        let mut g = Gestures::default();
        run(&mut g, &[(1, Phase::Down, 900.0, 500.0, 0)]);
        g.forget(1);
        assert!(!g.active());
    }

    #[test]
    fn two_fingers_do_nothing() {
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
        assert!(got.is_empty());
        assert!(tick(&mut g, 2000).is_empty());
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
