//! The frame rate limit chosen on the graphics page: no limit, the display's own pace, or a
//! number of frames a second. The page offers the rates the display can show evenly: up to the
//! display's own rate with the smooth frames on (a frame between two ticks is drawn blended),
//! up to the simulation's hundred ticks a second with them off (a frame between two ticks
//! would show nothing new).

use std::time::Duration;

/// The most frames a second shown without the smooth frames: the simulation's ticks. A frame
/// between two ticks would show nothing new.
pub const MOST_FRAMES: f64 = 100.0;

/// The most frames a second a rate may be, whatever the display.
const MOST_SMOOTH_FRAMES: f64 = 1000.0;

/// The most frames a second shown: the ticks' hundred, or with `smooth` frames what the display
/// has.
pub fn most_frames(smooth: bool) -> f64 {
    if smooth {
        MOST_SMOOTH_FRAMES
    } else {
        MOST_FRAMES
    }
}

/// The refreshes a second taken for a display that names none (or an absurd number).
pub const USUAL_REFRESH: f64 = 60.0;

/// What limits the frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FrameLimit {
    /// Nothing waits for a frame's time (a display that waits for its refresh still does).
    Off,
    /// One frame for each refresh of the display (every second or third on a display faster
    /// than the simulation's ticks).
    #[default]
    Display,
    /// This many frames a second.
    Rate(u32),
}

/// The refresh rate the pace is worked from: a usual one when the display names none.
pub fn usable_refresh(refresh: Option<f64>) -> f64 {
    refresh
        .filter(|rate| (24.0..=1000.0).contains(rate))
        .unwrap_or(USUAL_REFRESH)
}

/// The time of one frame at the display's pace: one refresh, or the fewest whole refreshes that
/// keep the frames at `most_frames(smooth)` a second or under.
pub fn display_period(refresh: Option<f64>, smooth: bool) -> Duration {
    let refresh = usable_refresh(refresh);
    let refreshes = (refresh / most_frames(smooth)).ceil().max(1.0);
    Duration::from_secs_f64(refreshes / refresh)
}

/// The frames a second the display's pace gives.
pub fn display_rate(refresh: Option<f64>, smooth: bool) -> f64 {
    1.0 / display_period(refresh, smooth).as_secs_f64()
}

impl FrameLimit {
    /// The time of one frame; `None` when nothing limits it.
    pub fn period(self, refresh: Option<f64>, smooth: bool) -> Option<Duration> {
        match self {
            FrameLimit::Off => None,
            FrameLimit::Display => Some(display_period(refresh, smooth)),
            // A rate chosen with the smooth frames on, kept with them off, is the ticks' hundred.
            FrameLimit::Rate(n) => Some(Duration::from_secs_f64(
                1.0 / f64::from(n.max(1)).min(most_frames(smooth)),
            )),
        }
    }

    /// The choices a display offers, slowest first: off, fixed rates up to what the display and
    /// the simulation allow, the rates that divide its refresh evenly (half, a third, a quarter)
    /// and its own pace.
    pub fn choices(refresh: Option<f64>, smooth: bool) -> Vec<FrameLimit> {
        let refresh = usable_refresh(refresh);
        let own = display_rate(Some(refresh), smooth);
        let most = refresh.min(most_frames(smooth));
        let mut rates: Vec<u32> = [30.0, 60.0]
            .into_iter()
            .chain((2..=4).map(|n| (refresh / f64::from(n)).round()))
            .filter(|rate| (24.0..=most).contains(rate))
            .map(|rate| rate as u32)
            .collect();
        rates.sort_unstable();
        rates.dedup();
        // A rate within a frame a second of the display's own pace is that pace.
        rates.retain(|&rate| (f64::from(rate) - own).abs() >= 1.0);
        let mut list = vec![FrameLimit::Off];
        list.extend(rates.into_iter().map(FrameLimit::Rate));
        let place = list
            .iter()
            .position(|l| matches!(l, FrameLimit::Rate(n) if f64::from(*n) > own))
            .unwrap_or(list.len());
        list.insert(place, FrameLimit::Display);
        list
    }

    /// The next choice (`up`) or the one before; the ends stay where they are. A limit that is
    /// not among the choices (it was saved for another display) is taken as the nearest rate
    /// below it.
    pub fn step(self, refresh: Option<f64>, smooth: bool, up: bool) -> FrameLimit {
        let list = FrameLimit::choices(refresh, smooth);
        let at = list.iter().position(|l| *l == self).unwrap_or_else(|| {
            let rate = |l: &FrameLimit| match l {
                FrameLimit::Off => f64::INFINITY,
                FrameLimit::Display => display_rate(refresh, smooth),
                FrameLimit::Rate(n) => f64::from(*n),
            };
            let mine = rate(&self);
            list.iter().rposition(|l| rate(l) <= mine).unwrap_or(0)
        });
        let to = if up {
            (at + 1).min(list.len() - 1)
        } else {
            at.saturating_sub(1)
        };
        list[to]
    }
}

/// Whether `limit` is the default (left out of the options file).
pub fn is_default(limit: &FrameLimit) -> bool {
    *limit == FrameLimit::default()
}

/// The options file's way: "off", "display" or a number.
pub mod as_option {
    use super::FrameLimit;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Text(String),
        Number(u32),
    }

    pub fn serialize<S: Serializer>(limit: &FrameLimit, to: S) -> Result<S::Ok, S::Error> {
        match limit {
            FrameLimit::Off => Raw::Text("off".into()),
            FrameLimit::Display => Raw::Text("display".into()),
            FrameLimit::Rate(n) => Raw::Number(*n),
        }
        .serialize(to)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(from: D) -> Result<FrameLimit, D::Error> {
        match Raw::deserialize(from)? {
            Raw::Text(t) if t == "off" => Ok(FrameLimit::Off),
            Raw::Text(t) if t == "display" => Ok(FrameLimit::Display),
            Raw::Number(n) if (1..=1000).contains(&n) => Ok(FrameLimit::Rate(n)),
            _ => Err(serde::de::Error::custom(
                "frame_limit is \"off\", \"display\" or a number of frames a second",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sixty_hertz_display_offers_what_divides_evenly() {
        use FrameLimit::*;
        assert_eq!(
            FrameLimit::choices(Some(60.0), false),
            vec![Off, Rate(30), Display]
        );
    }

    #[test]
    fn a_fast_display_pace_is_its_second_refresh() {
        use FrameLimit::*;
        // 144 Hz: every second refresh is 72 a second.
        assert_eq!(
            FrameLimit::choices(Some(144.0), false),
            vec![Off, Rate(30), Rate(36), Rate(48), Rate(60), Display]
        );
        assert!((display_rate(Some(144.0), false) - 72.0).abs() < 1e-6);
        assert_eq!(
            display_period(None, false),
            Duration::from_secs_f64(1.0 / 60.0)
        );
    }

    #[test]
    fn stepping_stops_at_the_ends_and_finds_a_limit_the_display_lacks() {
        let at = FrameLimit::Off;
        assert_eq!(at.step(Some(60.0), false, false), FrameLimit::Off);
        assert_eq!(at.step(Some(60.0), false, true), FrameLimit::Rate(30));
        assert_eq!(
            FrameLimit::Display.step(Some(60.0), false, true),
            FrameLimit::Display
        );
        // 100 is not offered at 60 Hz: the nearest below it is the display's own pace.
        assert_eq!(
            FrameLimit::Rate(100).step(Some(60.0), false, false),
            FrameLimit::Rate(30)
        );
    }

    #[test]
    fn smooth_frames_offer_the_displays_own_rate_and_its_half() {
        use FrameLimit::*;
        assert_eq!(
            FrameLimit::choices(Some(240.0), true),
            vec![Off, Rate(30), Rate(60), Rate(80), Rate(120), Display]
        );
        assert!((display_rate(Some(240.0), true) - 240.0).abs() < 1e-3);
        assert!((display_rate(Some(240.0), false) - 80.0).abs() < 1e-3);
        // A rate over the ticks' hundred is held there with the smooth frames off.
        assert_eq!(
            Rate(240).period(Some(240.0), false),
            Some(Duration::from_secs_f64(0.01))
        );
        assert_eq!(
            Rate(120).period(Some(240.0), true),
            Some(Duration::from_secs_f64(1.0 / 120.0))
        );
    }

    #[test]
    fn periods() {
        assert_eq!(FrameLimit::Off.period(Some(60.0), false), None);
        assert_eq!(
            FrameLimit::Rate(30).period(Some(60.0), false),
            Some(Duration::from_secs_f64(1.0 / 30.0))
        );
    }

    #[test]
    fn the_file_value_round_trips() {
        use crate::options::model::Display;
        for l in [FrameLimit::Off, FrameLimit::Display, FrameLimit::Rate(45)] {
            let display = Display {
                frame_limit: l,
                ..Display::default()
            };
            let text = toml::to_string(&display).unwrap();
            assert_eq!(toml::from_str::<Display>(&text).unwrap(), display);
        }
        // Left out when it is the default; a bad value is refused.
        assert!(
            !toml::to_string(&Display::default())
                .unwrap()
                .contains("frame_limit")
        );
        assert!(
            toml::from_str::<Display>("menu_mode=0\nrace_mode=0\nmipmapping=0\nframe_limit = 0")
                .is_err()
        );
        assert!(
            toml::from_str::<Display>(
                "menu_mode=0\nrace_mode=0\nmipmapping=0\nframe_limit = \"x\""
            )
            .is_err()
        );
    }
}
