//! The switches that say how the program runs on this machine, not what the game does: which
//! renderer draws (`--renderer`) and whether another copy may run beside this one
//! (`--allow-another`), or runs as a numbered instance with its own saved state (`--instance`). The words after the program's own name, as `menu::launch` takes them.

/// The switches that take a value (the next word, or after `=`).
pub(crate) const VALUED: [&str; 12] = [
    "renderer",
    "instance",
    "game-data",
    "render-replay",
    "out",
    "size",
    "fps",
    "quality",
    "encoder",
    "from",
    "to",
    "hud",
];

/// The switches that take none.
pub const PLAIN: [&str; 2] = ["--allow-another", "--no-sound"];

/// What the program's words ask for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Switches {
    /// `--renderer`'s value as given (the last one), for the renderer's choice to check.
    pub renderer: Option<String>,
    /// `--allow-another`: this copy starts although another copy is running in the folder.
    pub allow_another: bool,
    /// `--instance`'s number, 1 ..= `MOST_INSTANCES`: this copy is that instance of the folder.
    pub instance: Option<u32>,
    /// `--game-data`: the folder or zip the game's data is taken from when the program's folder
    /// has none (the first-run page's choice, given on the command line).
    pub game_data: Option<String>,
    /// `--render-replay`: the replay file (or slot) to render to a video instead of playing it.
    pub render_replay: Option<String>,
    /// `--out`: the video file (or, with no encoder at hand, the name its frames' folder is made
    /// from).
    pub out: Option<String>,
    /// `--size`: the video's width and height.
    pub size: Option<(u32, u32)>,
    /// `--fps`: the video's frames a second.
    pub fps: Option<u32>,
    /// `--quality`: how well the encoder keeps the picture.
    pub quality: Option<crate::race::render_dialog::Quality>,
    /// `--encoder`: whether a hardware encoder may be used (`automatic`) or only the software
    /// one (`software`).
    pub encoder: Option<crate::race::render_dialog::VideoEncoder>,
    /// `--no-sound`: the video has no sound.
    pub no_sound: bool,
    /// `--from` and `--to`: the part of the replay to render, in hundredths of a second from its
    /// start (the end of the replay when `--to` is not given).
    pub from_cs: Option<u32>,
    pub to_cs: Option<u32>,
    /// `--hud`: the first two cars' HUD pieces in the render (the viewer's own, which the
    /// render's replay has no other way to know: the players' settings are taken in by a race
    /// started from the menus).
    pub hud_bits: Option<[u32; 2]>,
}

/// The video sizes taken: each side from this many pixels up to that many (even sides are
/// needed by the common video formats; the game draws any).
pub(crate) const SIZE_RANGE: std::ops::RangeInclusive<u32> = 160..=16384;

/// The video frame rates taken.
pub(crate) const FPS_RANGE: std::ops::RangeInclusive<u32> = 1..=480;

/// A size as `--size` gives it: `1280x720`.
pub(crate) fn parse_size(text: &str) -> Option<(u32, u32)> {
    let (w, h) = text.split_once(['x', 'X'])?;
    let (w, h) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    (SIZE_RANGE.contains(&w) && SIZE_RANGE.contains(&h)).then_some((w, h))
}

/// A time as `--from` and `--to` give it, in seconds with up to two decimals, as hundredths of a
/// second; a replay holds at most 90 minutes, so a time is under 600 seconds.
pub(crate) fn parse_seconds(text: &str) -> Option<u32> {
    let seconds: f64 = text.trim().parse().ok()?;
    (seconds.is_finite() && (0.0..600.0).contains(&seconds))
        .then(|| (seconds * 100.0).round() as u32)
}

/// The most instances `--instance` names.
pub(crate) const MOST_INSTANCES: u32 = 8;

impl Switches {
    /// The switches among `args` (the words after the program's own name). Refused: a
    /// `--renderer` with no value after it.
    pub fn from_args<S: AsRef<str>>(args: &[S]) -> Result<Self, String> {
        let mut found = Self::default();
        let mut i = 0;
        while i < args.len() {
            let word = args[i].as_ref();
            i += 1;
            if PLAIN.contains(&word) {
                if word == "--no-sound" {
                    found.no_sound = true;
                } else {
                    found.allow_another = true;
                }
                continue;
            }
            let Some(long) = word.strip_prefix("--") else {
                continue;
            };
            let (key, inline) = match long.split_once('=') {
                Some((k, v)) => (k, Some(v.to_string())),
                None => (long, None),
            };
            // Another switch's value is no switch of ours, whatever it looks like.
            if !VALUED.contains(&key) {
                if inline.is_none() && crate::cli::takes_value(word) {
                    i += 1;
                }
                continue;
            }
            let value = match inline {
                Some(v) => v,
                None => {
                    let v = args
                        .get(i)
                        .ok_or_else(|| format!("--{key} needs a value"))?;
                    i += 1;
                    v.as_ref().to_string()
                }
            };
            if key == "instance" {
                let number = value
                    .parse::<u32>()
                    .ok()
                    .filter(|n| (1..=MOST_INSTANCES).contains(n))
                    .ok_or_else(|| {
                        format!("--instance {value}: a number from 1 to {MOST_INSTANCES} is wanted")
                    })?;
                found.instance = Some(number);
            } else if key == "game-data" {
                found.game_data = Some(value);
            } else if key == "render-replay" {
                found.render_replay = Some(value);
            } else if key == "out" {
                found.out = Some(value);
            } else if key == "size" {
                found.size = Some(parse_size(&value).ok_or_else(|| {
                    format!(
                        "--size {value}: a size like 1280x720 is wanted (sides of 160 to 16384)"
                    )
                })?);
            } else if key == "fps" {
                found.fps = Some(
                    value
                        .parse::<u32>()
                        .ok()
                        .filter(|n| FPS_RANGE.contains(n))
                        .ok_or_else(|| {
                            format!("--fps {value}: a number from 1 to 480 is wanted")
                        })?,
                );
            } else if key == "from" || key == "to" {
                let cs = parse_seconds(&value).ok_or_else(|| {
                    format!("--{key} {value}: a time in seconds, like 90 or 12.5, is wanted")
                })?;
                if key == "from" {
                    found.from_cs = Some(cs);
                } else {
                    found.to_cs = Some(cs);
                }
            } else if key == "hud" {
                found.hud_bits =
                    Some(parse_hud(&value).ok_or_else(|| {
                        format!("--hud {value}: two numbers, like 31,0, are wanted")
                    })?);
            } else if key == "encoder" {
                found.encoder = Some(
                    crate::race::render_dialog::VideoEncoder::of(&value).ok_or_else(|| {
                        format!("--encoder {value}: automatic or software is wanted")
                    })?,
                );
            } else if key == "quality" {
                found.quality = Some(crate::race::render_dialog::Quality::of(&value).ok_or_else(
                    || format!("--quality {value}: low, medium, high or best is wanted"),
                )?);
            } else {
                found.renderer = Some(value);
            }
        }
        Ok(found)
    }
}

/// `--hud`'s value: two numbers parted by a comma.
fn parse_hud(value: &str) -> Option<[u32; 2]> {
    let (first, second) = value.split_once(',')?;
    Some([first.trim().parse().ok()?, second.trim().parse().ok()?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_switches_are_read_with_their_values() {
        assert_eq!(Switches::from_args::<&str>(&[]), Ok(Switches::default()));
        let s = Switches::from_args(&["--renderer", "software", "--allow-another"]).unwrap();
        assert_eq!(s.renderer.as_deref(), Some("software"));
        assert!(s.allow_another);
        let s = Switches::from_args(&["--renderer=gpu", "--renderer", "software"]).unwrap();
        assert_eq!(s.renderer.as_deref(), Some("software"));
        assert!(!s.allow_another);
    }

    #[test]
    fn an_instance_is_a_small_number() {
        let s = Switches::from_args(&["--instance", "3"]).unwrap();
        assert_eq!(s.instance, Some(3));
        assert_eq!(
            Switches::from_args(&["--instance=1"]).unwrap().instance,
            Some(1)
        );
        assert!(Switches::from_args(&["--instance", "0"]).is_err());
        assert!(Switches::from_args(&["--instance", "9"]).is_err());
        assert!(Switches::from_args(&["--instance=two"]).is_err());
        assert!(Switches::from_args(&["--instance"]).is_err());
    }

    #[test]
    fn the_video_switches_are_read_and_checked() {
        let s = Switches::from_args(&[
            "--render-replay",
            "r.rpl",
            "--out=v.mp4",
            "--size",
            "1280x720",
            "--fps",
            "60",
        ])
        .unwrap();
        assert_eq!(s.render_replay.as_deref(), Some("r.rpl"));
        assert_eq!(s.out.as_deref(), Some("v.mp4"));
        assert_eq!((s.size, s.fps), (Some((1280, 720)), Some(60)));
        assert_eq!(parse_size("1920X1080"), Some((1920, 1080)));
        for bad in ["1280", "x720", "10x10", "1280x", "99999x720"] {
            assert!(Switches::from_args(&["--size", bad]).is_err(), "{bad}");
        }
        assert!(Switches::from_args(&["--fps", "0"]).is_err());
        assert!(Switches::from_args(&["--fps", "fast"]).is_err());
        let s = Switches::from_args(&["--quality=Best", "--no-sound"]).unwrap();
        assert_eq!(s.quality, Some(crate::race::render_dialog::Quality::Best));
        assert!(s.no_sound && !s.allow_another);
        assert!(Switches::from_args(&["--quality", "loud"]).is_err());
        let s = Switches::from_args(&["--encoder=Software"]).unwrap();
        assert_eq!(
            s.encoder,
            Some(crate::race::render_dialog::VideoEncoder::Software)
        );
        assert!(Switches::from_args(&["--encoder", "gpu"]).is_err());
        let s = Switches::from_args(&["--from", "12.5", "--to=90"]).unwrap();
        assert_eq!((s.from_cs, s.to_cs), (Some(1250), Some(9000)));
        assert!(Switches::from_args(&["--from", "-1"]).is_err());
        assert!(Switches::from_args(&["--to", "soon"]).is_err());
        let s = Switches::from_args(&["--hud", "31,0"]).unwrap();
        assert_eq!(s.hud_bits, Some([31, 0]));
        assert!(Switches::from_args(&["--hud", "31"]).is_err());
    }

    #[test]
    fn a_value_is_not_a_switch_and_may_not_be_missing() {
        // The word after a valued switch is its value, whatever it looks like.
        let s = Switches::from_args(&["--renderer", "--allow-another"]).unwrap();
        assert_eq!(s.renderer.as_deref(), Some("--allow-another"));
        assert!(!s.allow_another);
        assert!(Switches::from_args(&["--renderer"]).is_err());
        // Nor is it ours when another switch takes it.
        let s = Switches::from_args(&["--stage", "--allow-another", "--car=x"]).unwrap();
        assert_eq!(s, Switches::default());
    }
}
