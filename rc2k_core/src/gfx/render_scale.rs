//! The share of the screen's size the race's 3D is drawn at, chosen on the graphics page. The
//! picture is drawn smaller and enlarged to the screen; the menus' pictures, the dashboard and
//! the text stay at full size.

/// The choices, in percent of the screen's width and height, smallest first.
pub const CHOICES: [u32; 3] = [50, 75, 100];

/// The share when nothing was chosen.
pub const DEFAULT: u32 = 100;

/// The nearest choice at or below `percent` (a value from a hand-edited file); the smallest when
/// it is below them all.
pub fn nearest(percent: u32) -> u32 {
    CHOICES
        .into_iter()
        .rev()
        .find(|&c| c <= percent)
        .unwrap_or(CHOICES[0])
}

/// The next choice (`up`) or the one before; the ends stay where they are.
pub fn step(percent: u32, up: bool) -> u32 {
    let at = CHOICES
        .iter()
        .position(|&c| c == nearest(percent))
        .unwrap_or(0);
    let to = if up {
        (at + 1).min(CHOICES.len() - 1)
    } else {
        at.saturating_sub(1)
    };
    CHOICES[to]
}

/// A length drawn at `percent` of `full`: rounded up, so a share never leaves a pixel row or
/// column of the screen without a source, and never zero.
pub fn scaled(full: usize, percent: u32) -> usize {
    (full * percent as usize).div_ceil(100).max(1)
}

/// The place in the scaled picture (of `scaled(full, percent)` pixels) that the screen pixel
/// `at` shows.
pub fn source(at: usize, full: usize, small: usize) -> usize {
    (at * small / full.max(1)).min(small.saturating_sub(1))
}

/// Whether `percent` is the default (left out of the options file).
pub fn is_default(percent: &u32) -> bool {
    *percent == DEFAULT
}

/// The default, for the options file's missing field.
pub fn default_percent() -> u32 {
    DEFAULT
}

/// The options file's way: a number of percent among the choices.
pub mod as_option {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(percent: &u32, to: S) -> Result<S::Ok, S::Error> {
        to.serialize_u32(*percent)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(from: D) -> Result<u32, D::Error> {
        let percent = u32::deserialize(from)?;
        if !(1..=100).contains(&percent) {
            return Err(serde::de::Error::custom(
                "render_scale is 50, 75 or 100 (percent of the screen's size)",
            ));
        }
        Ok(super::nearest(percent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stay_among_the_choices() {
        assert_eq!(step(100, true), 100);
        assert_eq!(step(100, false), 75);
        assert_eq!(step(75, false), 50);
        assert_eq!(step(50, false), 50);
        assert_eq!(step(60, true), 75);
    }

    #[test]
    fn nearest_goes_down() {
        assert_eq!(nearest(99), 75);
        assert_eq!(nearest(10), 50);
        assert_eq!(nearest(100), 100);
    }

    #[test]
    fn the_file_value_round_trips() {
        use crate::options::model::Display;
        for percent in CHOICES {
            let display = Display {
                render_scale: percent,
                ..Display::default()
            };
            let text = toml::to_string(&display).unwrap();
            assert_eq!(toml::from_str::<Display>(&text).unwrap(), display);
        }
        // Left out for the default, which an old file without it gets; a value between the
        // choices goes down to one; nonsense is refused.
        assert!(
            !toml::to_string(&Display::default())
                .unwrap()
                .contains("render_scale")
        );
        let old = "menu_mode=0\nrace_mode=0\nmipmapping=0";
        assert_eq!(toml::from_str::<Display>(old).unwrap().render_scale, 100);
        assert_eq!(
            toml::from_str::<Display>(&format!("{old}\nrender_scale = 80"))
                .unwrap()
                .render_scale,
            75
        );
        assert!(toml::from_str::<Display>(&format!("{old}\nrender_scale = 0")).is_err());
        assert!(toml::from_str::<Display>(&format!("{old}\nrender_scale = 150")).is_err());
    }

    #[test]
    fn sizes_cover_the_screen() {
        assert_eq!(scaled(640, 50), 320);
        assert_eq!(scaled(640, 75), 480);
        assert_eq!(scaled(1366, 75), 1025);
        assert_eq!(scaled(1, 50), 1);
        for full in [480usize, 641, 1366] {
            for p in CHOICES {
                let small = scaled(full, p);
                assert_eq!(source(full - 1, full, small), small - 1);
                assert_eq!(source(0, full, small), 0);
            }
        }
    }
}
