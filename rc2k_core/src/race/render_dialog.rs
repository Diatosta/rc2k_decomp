//! The choices of the dialog the replay viewer opens before it renders a video: the video's
//! size, its frames a second, the part of the replay, the encoder's quality and whether the
//! sound goes in. The dialog's own logic lives here (what each key does, which values each row
//! steps through), and the game's side draws it.
//!
//! The size, rate, quality and sound are remembered between runs (`options::model::VideoChoices`
//! in `var/rc2k.toml`); the part is not, since "from here" and "between the marks" depend on the
//! viewer.

use crate::gfx::frame_limit::FrameLimit;
use crate::options::model::VideoChoices;

/// The frame rates the dialog offers: the numbers of the graphics page's frame limits (the
/// display's own pace as its rate; "off" has none), and 30 and 60 whatever the page lists,
/// since a video needs a fixed rate. Slowest first.
pub(crate) fn frame_rates(limits: &[FrameLimit], display_rate: f64) -> Vec<u32> {
    let mut rates: Vec<u32> = limits
        .iter()
        .filter_map(|l| match l {
            FrameLimit::Off => None,
            FrameLimit::Display => Some(display_rate.round() as u32),
            FrameLimit::Rate(n) => Some(*n),
        })
        .chain([30, 60])
        .filter(|&n| n > 0)
        .collect();
    rates.sort_unstable();
    rates.dedup();
    rates
}

/// The frame rates a video offers on a display of `refresh` Hz: the page's frame limits as with the
/// smooth frames on (a video always blends the frames between two ticks, so the rates go up to the
/// display's own whatever the page's smooth setting), and 30 and 60.
pub fn video_rates(refresh: Option<f64>) -> Vec<u32> {
    let limits = FrameLimit::choices(refresh, true);
    frame_rates(
        &limits,
        crate::gfx::frame_limit::display_rate(refresh, true),
    )
}

/// The rate when none was chosen.
pub const DEFAULT_FPS: u32 = 60;

/// The part of the replay that is rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// From the replay's first frame to its end.
    Whole,
    /// From the block and frame the viewer shows to the end.
    FromHere,
    /// From the first mark to the second.
    Marked,
}

/// Which encoder writes the picture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VideoEncoder {
    /// The machine's own encoder when one works (a graphics card's), else the software one.
    #[default]
    Automatic,
    /// Always the software encoder.
    Software,
}

impl VideoEncoder {
    pub const ALL: [VideoEncoder; 2] = [Self::Automatic, Self::Software];

    /// The number of the choice in `ALL`.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&e| e == self).unwrap_or(0)
    }

    /// The word `--encoder` uses.
    pub fn name(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Software => "software",
        }
    }

    /// The choice a word names.
    pub fn of(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|e| e.name().eq_ignore_ascii_case(name.trim()))
    }
}

/// How good the picture is kept by the encoder (its constant rate factor: lower is better).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Quality {
    Low,
    Medium,
    #[default]
    High,
    Best,
}

impl Quality {
    pub const ALL: [Quality; 4] = [Self::Low, Self::Medium, Self::High, Self::Best];

    /// The encoder's constant rate factor.
    pub fn crf(self) -> u32 {
        match self {
            Self::Low => 28,
            Self::Medium => 23,
            Self::High => 18,
            Self::Best => 12,
        }
    }

    /// The word the options file and `--quality` use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Best => "best",
        }
    }

    /// The quality a word names.
    pub fn of(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|q| q.name().eq_ignore_ascii_case(name.trim()))
    }

    /// The number of the quality in `ALL`.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&q| q == self).unwrap_or(2)
    }
}

/// What is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The video's size; none: the screen's own.
    pub size: Option<(u32, u32)>,
    pub fps: u32,
    pub part: Part,
    pub quality: Quality,
    pub encoder: VideoEncoder,
    pub sound: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            size: None,
            fps: DEFAULT_FPS,
            part: Part::Whole,
            quality: Quality::default(),
            encoder: VideoEncoder::default(),
            sound: true,
        }
    }
}

impl Settings {
    /// The settings the options file kept (the part is always the whole replay).
    pub fn from_kept(kept: &VideoChoices) -> Self {
        let base = Self::default();
        Self {
            size: kept.size.map(|[w, h]| (w, h)),
            fps: kept
                .fps
                .filter(|f| (1..=480).contains(f))
                .unwrap_or(base.fps),
            part: Part::Whole,
            quality: kept
                .quality
                .as_deref()
                .and_then(Quality::of)
                .unwrap_or(base.quality),
            encoder: kept
                .encoder
                .as_deref()
                .and_then(VideoEncoder::of)
                .unwrap_or(base.encoder),
            sound: kept.sound.unwrap_or(base.sound),
        }
    }

    /// What goes into the options file: only what differs from the defaults.
    pub fn to_kept(&self) -> VideoChoices {
        let base = Self::default();
        VideoChoices {
            size: self.size.map(|(w, h)| [w, h]),
            fps: (self.fps != base.fps).then_some(self.fps),
            quality: (self.quality != base.quality).then(|| self.quality.name().to_owned()),
            encoder: (self.encoder != base.encoder).then(|| self.encoder.name().to_owned()),
            sound: (self.sound != base.sound).then_some(self.sound),
        }
    }
}

/// The rows, top to bottom; the last two are buttons side by side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Size,
    FrameRate,
    Part,
    Quality,
    Encoder,
    Sound,
    Render,
    Cancel,
}

pub const ROWS: [Row; 8] = [
    Row::Size,
    Row::FrameRate,
    Row::Part,
    Row::Quality,
    Row::Encoder,
    Row::Sound,
    Row::Render,
    Row::Cancel,
];

/// A key the dialog answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
}

/// What a key leads to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Render,
    Cancel,
}

/// The dialog's state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialog {
    pub settings: Settings,
    /// The row with the highlight, a number in `ROWS`.
    pub row: usize,
    /// Whether the viewer's marks are set (the part "between the marks" is offered then).
    pub marks: bool,
    /// The sizes offered after the screen's own: the graphics page's list of resolutions.
    pub sizes: Vec<(u32, u32)>,
    /// The frame rates offered (`frame_rates`).
    pub rates: Vec<u32>,
}

fn sorted(mut sizes: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    crate::gfx::resolutions::sort(&mut sizes);
    sizes
}

/// The value after (`forward`) or before `current` in `list`, wrapping; the first value when
/// `current` is not in it.
fn step<T: Copy + PartialEq>(list: &[T], current: T, forward: bool) -> T {
    let Some(at) = list.iter().position(|&v| v == current) else {
        return list[0];
    };
    let n = list.len();
    list[if forward {
        (at + 1) % n
    } else {
        (at + n - 1) % n
    }]
}

impl Dialog {
    pub fn new(
        mut settings: Settings,
        marks: bool,
        sizes: Vec<(u32, u32)>,
        rates: Vec<u32>,
    ) -> Self {
        // A choice kept for another display is still offered.
        if let Some(size) = settings.size
            && !sizes.contains(&size)
        {
            let mut sizes = sizes.clone();
            sizes.push(size);
            return Self::new(settings, marks, sorted(sizes), rates);
        }
        if !rates.contains(&settings.fps) {
            let mut rates = rates;
            rates.push(settings.fps);
            rates.sort_unstable();
            return Self::new(settings, marks, sizes, rates);
        }
        settings.part = Part::Whole;
        Self {
            sizes,
            rates,
            settings,
            row: ROWS.iter().position(|&r| r == Row::Render).unwrap_or(0),
            marks,
        }
    }

    /// The parts the dialog offers.
    pub fn parts(&self) -> &'static [Part] {
        if self.marks {
            &[Part::Whole, Part::FromHere, Part::Marked]
        } else {
            &[Part::Whole, Part::FromHere]
        }
    }

    /// The row with the highlight.
    pub fn current(&self) -> Row {
        ROWS[self.row]
    }

    /// The row's value changed one step.
    fn change(&mut self, row: Row, forward: bool) {
        let parts = self.parts();
        let s = &mut self.settings;
        match row {
            Row::Size => {
                let list: Vec<Option<(u32, u32)>> = std::iter::once(None)
                    .chain(self.sizes.iter().map(|&size| Some(size)))
                    .collect();
                s.size = step(&list, s.size, forward);
            }
            Row::FrameRate => s.fps = step(&self.rates, s.fps, forward),
            Row::Part => s.part = step(parts, s.part, forward),
            Row::Quality => s.quality = step(&Quality::ALL, s.quality, forward),
            Row::Encoder => s.encoder = step(&VideoEncoder::ALL, s.encoder, forward),
            Row::Sound => s.sound = !s.sound,
            Row::Render | Row::Cancel => {}
        }
    }

    /// A key pressed.
    pub fn key(&mut self, key: Key) -> Outcome {
        let row = self.current();
        let button = matches!(row, Row::Render | Row::Cancel);
        match key {
            Key::Up => self.row = self.up_from(),
            Key::Down => {
                // The buttons share a line: down from a setting reaches the first.
                if !button {
                    self.row = (self.row + 1).min(ROWS.len() - 1);
                }
            }
            Key::Left if button => self.row = (self.row - 1).max(self.first_button()),
            Key::Right if button => self.row = (self.row + 1).min(ROWS.len() - 1),
            Key::Left => self.change(row, false),
            Key::Right => self.change(row, true),
            Key::Enter => match row {
                Row::Render => return Outcome::Render,
                Row::Cancel => return Outcome::Cancel,
                other => self.change(other, true),
            },
            Key::Escape => return Outcome::Cancel,
        }
        Outcome::Stay
    }

    fn first_button(&self) -> usize {
        ROWS.iter().position(|&r| r == Row::Render).unwrap_or(0)
    }

    /// The row up from the highlight (both buttons go up to the last setting).
    fn up_from(&self) -> usize {
        if self.row >= self.first_button() {
            self.first_button() - 1
        } else {
            self.row.saturating_sub(1)
        }
    }

    /// The row at the point (x, y) of a frame, from the places the rows were drawn at.
    pub fn row_at(places: &[(Row, [i32; 4])], x: i32, y: i32) -> Option<Row> {
        places
            .iter()
            .find(|(_, [l, t, w, h])| x >= *l && x < l + w && y >= *t && y < t + h)
            .map(|&(row, _)| row)
    }

    /// The highlight put on `row`.
    pub fn select(&mut self, row: Row) {
        if let Some(at) = ROWS.iter().position(|&r| r == row) {
            self.row = at;
        }
    }
}

/// How far a render has come, 0 ..= 1, when the replay's blocks `from ..= end` are played and
/// block `block` is being played.
pub fn progress(block: u32, from: u32, end: u32) -> f64 {
    let length = end.saturating_sub(from).max(1);
    (f64::from(block.saturating_sub(from)) / f64::from(length)).clamp(0.0, 1.0)
}

#[cfg(test)]
/// The size the video is made at: the chosen one, or the screen's.
pub(crate) fn video_size(chosen: Option<(u32, u32)>, screen: (u32, u32)) -> (u32, u32) {
    chosen.unwrap_or(screen)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: [(u32, u32); 3] = [(800, 600), (1280, 720), (3840, 2160)];

    fn dialog() -> Dialog {
        Dialog::new(
            Settings::default(),
            false,
            SIZES.to_vec(),
            vec![30, 60, 120],
        )
    }

    #[test]
    fn the_highlight_starts_on_render_and_the_buttons_share_a_line() {
        let mut d = dialog();
        assert_eq!(d.current(), Row::Render);
        d.key(Key::Right);
        assert_eq!(d.current(), Row::Cancel);
        d.key(Key::Right);
        assert_eq!(d.current(), Row::Cancel);
        d.key(Key::Left);
        assert_eq!(d.current(), Row::Render);
        d.key(Key::Left);
        assert_eq!(d.current(), Row::Render);
        d.key(Key::Right);
        d.key(Key::Up);
        assert_eq!(d.current(), Row::Sound);
        d.key(Key::Down);
        assert_eq!(d.current(), Row::Render);
        d.key(Key::Down);
        assert_eq!(d.current(), Row::Render);
        for _ in 0..8 {
            d.key(Key::Up);
        }
        assert_eq!(d.current(), Row::Size);
    }

    #[test]
    fn left_and_right_step_through_the_values_and_wrap() {
        let mut d = dialog();
        d.select(Row::FrameRate);
        d.key(Key::Right);
        assert_eq!(d.settings.fps, 120);
        d.key(Key::Right);
        assert_eq!(d.settings.fps, 30);
        d.key(Key::Left);
        assert_eq!(d.settings.fps, 120);
        d.select(Row::Size);
        d.key(Key::Left);
        assert_eq!(d.settings.size, Some((3840, 2160)));
        d.key(Key::Right);
        assert_eq!(d.settings.size, None);
        d.select(Row::Sound);
        d.key(Key::Enter);
        assert!(!d.settings.sound);
        d.key(Key::Left);
        assert!(d.settings.sound);
    }

    #[test]
    fn rates_are_the_pages_with_30_and_60() {
        let limits = FrameLimit::choices(Some(144.0), true);
        let rates = frame_rates(&limits, 144.0);
        assert!(rates.contains(&30) && rates.contains(&60) && rates.contains(&144));
        assert!(rates.windows(2).all(|w| w[0] < w[1]));
        let limits = [FrameLimit::Off, FrameLimit::Rate(75)];
        assert_eq!(frame_rates(&limits, 60.0), [30, 60, 75]);
    }

    #[test]
    fn a_video_offers_rates_up_to_the_display() {
        assert_eq!(video_rates(Some(240.0)), [30, 60, 80, 120, 240]);
        assert_eq!(video_rates(Some(144.0)), [30, 36, 48, 60, 72, 144]);
        assert_eq!(video_rates(Some(60.0)), [30, 60]);
        assert_eq!(video_rates(None), [30, 60]);
    }

    #[test]
    fn a_kept_choice_the_lists_lack_is_added() {
        let s = Settings {
            size: Some((1000, 700)),
            fps: 144,
            ..Settings::default()
        };
        let d = Dialog::new(s, false, SIZES.to_vec(), vec![60]);
        assert!(d.sizes.contains(&(1000, 700)));
        assert_eq!(d.rates, [60, 144]);
        assert_eq!(d.settings, s);
    }

    #[test]
    fn the_marked_part_is_offered_only_with_marks() {
        let mut d = dialog();
        d.select(Row::Part);
        for _ in 0..4 {
            d.key(Key::Right);
            assert_ne!(d.settings.part, Part::Marked);
        }
        let mut d = Dialog::new(Settings::default(), true, SIZES.to_vec(), vec![60]);
        d.select(Row::Part);
        d.key(Key::Left);
        assert_eq!(d.settings.part, Part::Marked);
    }

    #[test]
    fn enter_and_escape_end_the_dialog() {
        let mut d = dialog();
        assert_eq!(d.key(Key::Enter), Outcome::Render);
        assert_eq!(d.key(Key::Escape), Outcome::Cancel);
        d.key(Key::Right);
        assert_eq!(d.key(Key::Enter), Outcome::Cancel);
        d.select(Row::Quality);
        assert_eq!(d.key(Key::Enter), Outcome::Stay);
        assert_eq!(d.settings.quality, Quality::Best);
    }

    #[test]
    fn a_value_that_is_not_in_the_list_steps_to_the_first() {
        let mut d = dialog();
        d.settings.fps = 144;
        d.select(Row::FrameRate);
        d.key(Key::Right);
        assert_eq!(d.settings.fps, 30);
    }

    #[test]
    fn only_the_changed_choices_are_kept() {
        assert_eq!(Settings::default().to_kept(), VideoChoices::default());
        let s = Settings {
            size: Some((1920, 1080)),
            fps: 30,
            part: Part::FromHere,
            quality: Quality::Low,
            encoder: VideoEncoder::Software,
            sound: false,
        };
        let kept = s.to_kept();
        assert_eq!(kept.size, Some([1920, 1080]));
        assert_eq!(kept.fps, Some(30));
        assert_eq!(kept.quality.as_deref(), Some("low"));
        assert_eq!(kept.sound, Some(false));
        assert_eq!(kept.encoder.as_deref(), Some("software"));
        // The part is not kept.
        assert_eq!(
            Settings::from_kept(&kept),
            Settings {
                part: Part::Whole,
                ..s
            }
        );
        let odd = VideoChoices {
            size: None,
            fps: Some(0),
            quality: Some("loud".into()),
            encoder: Some("quantum".into()),
            sound: None,
        };
        assert_eq!(Settings::from_kept(&odd), Settings::default());
    }

    #[test]
    fn progress_is_along_the_part() {
        assert_eq!(progress(0, 0, 10), 0.0);
        assert_eq!(progress(5, 0, 10), 0.5);
        assert_eq!(progress(15, 10, 20), 0.5);
        assert_eq!(progress(3, 10, 20), 0.0);
        assert_eq!(progress(30, 10, 20), 1.0);
        assert_eq!(progress(4, 4, 4), 0.0);
    }

    #[test]
    fn a_row_is_found_by_its_place() {
        let places = [
            (Row::Size, [10, 10, 100, 20]),
            (Row::Sound, [10, 40, 100, 20]),
        ];
        assert_eq!(Dialog::row_at(&places, 20, 45), Some(Row::Sound));
        assert_eq!(Dialog::row_at(&places, 200, 45), None);
        assert_eq!(video_size(None, (800, 600)), (800, 600));
        assert_eq!(video_size(Some((1280, 720)), (800, 600)), (1280, 720));
    }
}
