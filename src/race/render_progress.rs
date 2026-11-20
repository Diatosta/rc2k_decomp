//! The panel the replay viewer shows while a video is rendered by a program of its own: this
//! program started again with `--render-replay` on a copy of the replay as it is viewed, no
//! window and no sound device, drawing the race at the size chosen. The child writes how far it
//! has come into a file (`RC2K_RENDER_PROGRESS`, a line "percent frames"), which the panel reads
//! and draws as a bar. The viewer waits behind the panel (it takes no key but the one that stops
//! the render); "cancel", Escape or a click stops the child and removes everything it wrote.
//! When the child ends well a notice says where the video is, when it fails a notice says why.

use crate::gfx::typefaces::fonts;
use crate::input::osk_draw::fill;
use crate::owners::Owners;
use crate::text::draw_text::{GameCanvas, draw_text_at};
use rc2k_core::race::render_dialog::{Quality, VideoEncoder};
use rc2k_core::strings::own::id;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// What is to be rendered.
#[derive(Clone)]
pub struct Render {
    /// The copy of the replay as it is viewed (taken away when the render ends).
    pub replay: PathBuf,
    /// The video file.
    pub out: PathBuf,
    pub size: (u32, u32),
    pub fps: u32,
    pub quality: Quality,
    pub encoder: VideoEncoder,
    pub sound: bool,
    /// The part, in hundredths of a second from the replay's start.
    pub from: Option<u32>,
    pub to: Option<u32>,
}

impl Render {
    /// The child's command line.
    fn arguments(&self) -> Vec<String> {
        let path = |p: &PathBuf| p.to_string_lossy().into_owned();
        let mut args = vec![
            "--allow-another".to_string(),
            "--render-replay".into(),
            path(&self.replay),
            "--out".into(),
            path(&self.out),
            "--size".into(),
            format!("{}x{}", self.size.0, self.size.1),
            "--fps".into(),
            self.fps.to_string(),
            "--quality".into(),
            self.quality.name().into(),
        ];
        if self.encoder != VideoEncoder::default() {
            args.extend(["--encoder".into(), self.encoder.name().into()]);
        }
        if !self.sound {
            args.push("--no-sound".into());
        }
        for (switch, value) in [("--from", self.from), ("--to", self.to)] {
            if let Some(cs) = value {
                args.extend([switch.into(), format!("{}.{:02}", cs / 100, cs % 100)]);
            }
        }
        args
    }

    /// The file the child lists the trouble it had in.
    fn problems_file(&self) -> PathBuf {
        self.out.with_extension(crate::video::PROBLEMS_EXTENSION)
    }

    /// The file the child reports its progress in.
    fn progress_file(&self) -> PathBuf {
        self.out.with_extension("progress")
    }
}

/// How the child ended.
enum Ended {
    Done,
    Stopped,
    Failed(String),
    /// The hardware encoder made a damaged picture (`video::DAMAGED`).
    EncoderDamaged,
}

/// The render going on until it ends or is stopped; a notice says how it ended.
pub fn run(o: Owners<'static>, render: &Render) {
    let mut ended = render_once(o, render);
    let mut redone = false;
    if matches!(ended, Ended::EncoderDamaged) {
        redone = true;
        // The graphics card's encoder cannot be trusted with this render: the software one makes
        // the video, from the same copy of the replay.
        crate::program_log::log(
            "render: the hardware encoder made a damaged picture; rendering again with the software encoder",
        );
        let again = Render {
            encoder: VideoEncoder::Software,
            ..render.clone()
        };
        ended = render_once(o, &again);
    }
    let problems = Problems::read(render);
    for file in [
        &render.replay,
        &render.progress_file(),
        &render.problems_file(),
    ] {
        let _ = std::fs::remove_file(file);
    }
    let say = |id: u32, with: &str| {
        crate::race::dialogs::notice(id, with);
    };
    match ended {
        Ended::Stopped => {}
        // No video encoder to write the file: the frames are pictures in a folder.
        _ if problems.no_ffmpeg.is_some() => {
            say(
                id::RENDER_NO_FFMPEG,
                problems.no_ffmpeg.as_deref().unwrap_or(""),
            );
        }
        Ended::Done => {
            if let Some(why) = &problems.encoder {
                say(id::RENDER_ENCODER_FAILED, why);
            } else {
                let name = render.out.file_name().unwrap_or_default().to_string_lossy();
                say(id::RENDER_DONE, &name);
            }
            if redone {
                say(id::RENDER_ENCODER_REDONE, "");
            }
            if problems.sound {
                say(id::RENDER_SOUND_FAILED, "");
            }
        }
        Ended::Failed(why) => {
            match &problems.encoder {
                Some(first) => say(id::RENDER_ENCODER_FAILED, first),
                None => say(id::RENDER_FAILED, &why),
            }
            if redone {
                say(id::RENDER_ENCODER_REDONE, "");
            }
        }
        Ended::EncoderDamaged => {
            crate::race::dialogs::notice(id::RENDER_FAILED, "the encoder made a damaged picture");
        }
    }
}

/// What the child told of what went wrong (`video::note_problem`).
#[derive(Default, Debug, PartialEq)]
struct Problems {
    /// No `ffmpeg`: the folder the numbered pictures went to.
    no_ffmpeg: Option<String>,
    /// The encoder stopped: its first line.
    encoder: Option<String>,
    /// The sound could not be joined.
    sound: bool,
}

impl Problems {
    fn read(render: &Render) -> Self {
        Self::parse(&std::fs::read_to_string(render.problems_file()).unwrap_or_default())
    }

    fn parse(text: &str) -> Self {
        let mut problems = Self::default();
        for line in text.lines() {
            let (kind, detail) = line.split_once('\t').unwrap_or((line, ""));
            match kind {
                "no_ffmpeg" => problems.no_ffmpeg = Some(detail.to_owned()),
                "encoder" => problems.encoder = Some(detail.to_owned()),
                "sound" => problems.sound = true,
                _ => {}
            }
        }
        problems
    }
}

/// One run of the child, with the panel up until it ends.
fn render_once(o: Owners<'static>, render: &Render) -> Ended {
    match start(render) {
        Ok(child) => watch(o, render, child),
        Err(why) => Ended::Failed(why),
    }
}

/// The child started: this program again, with no window or sound device of its own and no
/// dialog that would wait for a key nobody can press.
fn start(render: &Render) -> Result<Child, String> {
    let exe = std::env::current_exe().map_err(|e| format!("the program's file: {e}"))?;
    let _ = std::fs::remove_file(render.progress_file());
    let _ = std::fs::remove_file(render.problems_file());
    let args = render.arguments();
    crate::program_log::log(&format!("render: {} {}", exe.display(), args.join(" ")));
    Command::new(exe)
        .args(args)
        .env("SDL_VIDEODRIVER", "offscreen")
        .env("SDL_AUDIODRIVER", "dummy")
        .env("RC2K_STEAM_OFF", "1")
        .env("RC2K_NO_ERROR_DIALOG", "1")
        // A test driver's folder belongs to the program that was started for it.
        .env_remove("RC2K_REMOTE")
        .env("RC2K_RENDER_PROGRESS", render.progress_file())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())
}

/// What the child's progress file says: the percent done and the frames written.
fn read_progress(render: &Render) -> Option<(f64, u64)> {
    let text = std::fs::read_to_string(render.progress_file()).ok()?;
    parse_progress(&text)
}

fn parse_progress(text: &str) -> Option<(f64, u64)> {
    let mut words = text.split_whitespace();
    let percent: f64 = words.next()?.parse().ok()?;
    Some((percent, words.next()?.parse().ok()?))
}

/// How far back the speed is measured, in seconds.
const PACE_WINDOW: f64 = 10.0;

/// The frames written at the moments the progress was read, for the speed the render has now (the
/// last few seconds', so a slow start or a quiet stretch of the replay does not hold on).
#[derive(Default)]
struct Pace {
    /// (seconds since the render began, frames written), oldest first.
    samples: std::collections::VecDeque<(f64, u64)>,
}

impl Pace {
    /// `frames` were written by `at` seconds.
    fn note(&mut self, at: f64, frames: u64) {
        if self.samples.back().is_some_and(|&(_, f)| f == frames) {
            return;
        }
        self.samples.push_back((at, frames));
        while self.samples.len() > 2 && at - self.samples[1].0 >= PACE_WINDOW {
            self.samples.pop_front();
        }
    }

    /// Frames a second over the samples kept; none before two readings a moment apart.
    fn per_second(&self) -> Option<f64> {
        let (&(t0, f0), &(t1, f1)) = (self.samples.front()?, self.samples.back()?);
        (t1 - t0 >= 1.0 && f1 > f0).then(|| (f1 - f0) as f64 / (t1 - t0))
    }
}

/// The seconds the render has left, when `frames` are `percent` of it and `per_second` frames are
/// written a second; none before there is a percent to go by.
fn seconds_left(percent: f64, frames: u64, per_second: f64) -> Option<u64> {
    if percent < 0.5 || per_second <= 0.0 {
        return None;
    }
    let total = frames as f64 * 100.0 / percent;
    Some(((total - frames as f64).max(0.0) / per_second).round() as u64)
}

/// A time as minutes and seconds ("2:05"), with hours before them when there are any.
fn clock_text(seconds: u64) -> String {
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// The panel up until the child ends or the render is stopped.
fn watch(o: Owners<'static>, render: &Render, mut child: Child) -> Ended {
    // A key from before is not an answer.
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    let mut progress = None;
    let mut pace = Pace::default();
    let began = std::time::Instant::now();
    let mut was_down = true;
    // A tap on the cancel button stops the render (`shell::touch_menu`).
    #[cfg(touch_screen)]
    let _taps = crate::shell::touch_menu::InstallerPage::begin();
    let stop = |child: &mut Child| {
        let _ = child.kill();
        let _ = child.wait();
        for file in crate::video::leftovers(&render.out) {
            if file.is_dir() {
                let _ = std::fs::remove_dir_all(&file);
            } else {
                let _ = std::fs::remove_file(&file);
            }
        }
        Ended::Stopped
    };
    loop {
        crate::shell::frame::pump_messages_now(o);
        if o.system.session.flags.quitting.is_set() {
            return stop(&mut child);
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() && render.out.is_file() => return Ended::Done,
            Ok(Some(status))
                if render.encoder == VideoEncoder::Automatic
                    && status.code() == i32::try_from(crate::video::DAMAGED).ok() =>
            {
                return Ended::EncoderDamaged;
            }
            Ok(Some(status)) => {
                return Ended::Failed(format!("the render ended with {status}"));
            }
            Ok(None) => {}
            Err(e) => return Ended::Failed(e.to_string()),
        }
        progress = read_progress(render).or(progress);
        if let Some((_, frames)) = progress {
            pace.note(began.elapsed().as_secs_f64(), frames);
        }
        if o.gfx.locked.surface.get() == 1 {
            crate::platform::unlock_surface(o.gfx);
        }
        let Some(locked) = crate::gfx::with_locked_surface::lock(o.gfx, o.system) else {
            return stop(&mut child);
        };
        let (button, panel) = draw(o, render, progress, &pace);
        drop(locked);
        if !crate::shell::frame::show_frame_now(o) {
            return stop(&mut child);
        }
        crate::input::menu_pad::take_answer(o.input);
        let key = crate::input::take_key::take_key(o.input);
        let (buttons, x, y) = crate::input::keys::unpack_mouse(&o.input.devices);
        // The button is where the panel was laid out.
        let (x, y) = panel.point(x, y);
        let down = buttons & 1 != 0;
        let [l, t, w, h] = button;
        #[cfg_attr(
            not(touch_screen),
            allow(unused_mut, reason = "a tap adds to it on touch builds")
        )]
        let mut clicked = down && !was_down && x >= l && x < l + w && y >= t && y < t + h;
        #[cfg(touch_screen)]
        if let Some((tx, ty)) = crate::shell::touch_menu::take_installer_tap() {
            // The tap, like the mouse, where the panel was laid out.
            let (tx, ty) = panel.point(tx as i32, ty as i32);
            let hit = tx >= l && tx < l + w && ty >= t && ty < t + h;
            if hit {
                crate::shell::touch_menu::tap_used();
            }
            clicked |= hit;
        }
        was_down = down;
        if matches!(key, 0x01 | 0x1C | 0x60) || clicked {
            return stop(&mut child);
        }
    }
}

const SCREEN: u32 = 0x05081A;
const PANEL: u32 = 0x0C1238;
const BAR_TRACK: u32 = 0x27347A;
const BAR_DONE: u32 = 0x30D030;
const BUTTON: u32 = 0xC8202A;
const MARGIN: i32 = 20;
const TEXT_UP: i32 = 12;

/// One frame of the panel; the cancel button's place (for the mouse).
fn draw(
    o: Owners<'static>,
    render: &Render,
    progress: Option<(f64, u64)>,
    pace: &Pace,
) -> ([i32; 4], crate::gfx::ui_scale::Panel) {
    use crate::gfx::button_hints::width_of;
    let mut canvas = GameCanvas::new(o.gfx);
    // Laid out on the virtual screen, enlarged to the real one at the end.
    let view = crate::gfx::ui_scale::View::now(o);
    let (w, h) = (view.width, view.height);
    // The background at the real size, the panel on the virtual screen.
    fill(o, [0, 0, view.screen.0, view.screen.1], SCREEN);
    let own = |n: u32| {
        rc2k_core::files::ansi::ansi_text(&crate::strings::append_string_by_id::own_text(
            o.strings, n,
        ))
    };
    let measure = |s: &str| {
        width_of(
            fonts::LIST,
            &rc2k_core::files::ansi::ansi_bytes(s),
            &mut GameCanvas::new(o.gfx),
        )
    };
    let (line_h, row_h) = if h < 560 { (26, 32) } else { (30, 38) };
    let panel_w = (w - 40).min(720);
    let inner = panel_w - 2 * MARGIN;
    // The cancel button: taller on a touch screen, for a finger.
    let cancel_h = if cfg!(touch_screen) { 50 } else { row_h - 6 };
    let panel_h = MARGIN * 2 + line_h * 2 + line_h * 2 + row_h + line_h * 2 + cancel_h + 6;
    let panel = view.panel(panel_w, panel_h);
    let (left, top) = panel.origin();
    fill(o, [left, top, panel_w, panel_h], PANEL);
    let put = |text: &str, x: i32, y: i32, canvas: &mut GameCanvas| {
        let mut bytes = rc2k_core::files::ansi::ansi_bytes(text);
        bytes.push(0);
        let _ = draw_text_at(fonts::LIST, &bytes[..], x, y - TEXT_UP, canvas);
    };
    let mut y = top + MARGIN;
    put(
        &own(id::RENDER_PROGRESS_TITLE),
        left + MARGIN,
        y + line_h / 2,
        &mut canvas,
    );
    y += line_h * 2;
    let name = render.out.file_name().unwrap_or_default().to_string_lossy();
    let size = format!(
        "{name}  ({} x {}, {} fps)",
        render.size.0, render.size.1, render.fps
    );
    put(&size, left + MARGIN, y + line_h / 2, &mut canvas);
    y += line_h * 2;
    let percent = progress.map_or(0.0, |(p, _)| p.clamp(0.0, 100.0));
    fill(o, [left + MARGIN, y, inner, row_h - 8], BAR_TRACK);
    fill(
        o,
        [
            left + MARGIN,
            y,
            (f64::from(inner) * percent / 100.0) as i32,
            row_h - 8,
        ],
        BAR_DONE,
    );
    y += row_h;
    let status = match progress {
        Some((p, frames)) => format!(
            "{}   {}",
            own(id::RENDER_PERCENT).replace("{}", &(p.clamp(0.0, 100.0) as u32).to_string()),
            own(id::RENDER_FRAMES).replace("{}", &frames.to_string())
        ),
        None => own(id::RENDER_STARTING),
    };
    put(
        &status,
        left + (panel_w - measure(&status)) / 2,
        y + line_h / 2,
        &mut canvas,
    );
    y += line_h;
    // The speed and the time left, once the render has gone on long enough to tell.
    if let (Some(rate), Some((p, frames))) = (pace.per_second(), progress) {
        let mut speed = own(id::RENDER_SPEED).replace("{}", &format!("{rate:.1}"));
        if let Some(left) = seconds_left(p, frames, rate) {
            speed.push_str("   ");
            speed.push_str(&own(id::RENDER_LEFT).replace("{}", &clock_text(left)));
        }
        put(
            &speed,
            left + (panel_w - measure(&speed)) / 2,
            y + line_h / 2,
            &mut canvas,
        );
    }
    y += line_h + row_h / 4;
    let button = [left + MARGIN, y, inner, cancel_h];
    fill(o, button, BUTTON);
    let cancel = own(id::RENDER_CANCEL);
    put(
        &cancel,
        left + (panel_w - measure(&cancel)) / 2,
        y + cancel_h / 2,
        &mut canvas,
    );
    panel.finish(o, SCREEN);
    (button, panel)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_childs_problems_are_read_by_kind() {
        let read =
            Problems::parse("no_ffmpeg\tv_frames\nencoder\tNo such device\nsound\t\nother\tx\n");
        assert_eq!(read.no_ffmpeg.as_deref(), Some("v_frames"));
        assert_eq!(read.encoder.as_deref(), Some("No such device"));
        assert!(read.sound);
        assert_eq!(Problems::parse(""), Problems::default());
    }

    use super::*;

    fn render() -> Render {
        Render {
            replay: "savedata/videos/r.replay.tmp".into(),
            out: "savedata/videos/r.mp4".into(),
            size: (2560, 1440),
            fps: 60,
            quality: Quality::High,
            encoder: VideoEncoder::Software,
            sound: false,
            from: Some(1250),
            to: None,
        }
    }

    #[test]
    fn the_child_is_told_what_to_render() {
        let args = render().arguments();
        let after = |flag: &str| {
            let at = args.iter().position(|a| a == flag).unwrap();
            args[at + 1].clone()
        };
        assert_eq!(after("--render-replay"), "savedata/videos/r.replay.tmp");
        assert_eq!(after("--size"), "2560x1440");
        assert_eq!(after("--quality"), "high");
        assert_eq!(after("--encoder"), "software");
        assert_eq!(after("--from"), "12.50");
        assert!(args.contains(&"--no-sound".to_string()));
        assert!(args.contains(&"--allow-another".to_string()));
        assert!(!args.contains(&"--to".to_string()));
        // `Switches` reads what is given, whatever the game does with it.
        let parsed = rc2k_core::system::program_switches::Switches::from_args(&args).unwrap();
        assert_eq!(parsed.from_cs, Some(1250));
        assert_eq!(parsed.size, Some((2560, 1440)));
    }

    #[test]
    fn the_speed_is_the_recent_one() {
        let mut pace = Pace::default();
        assert_eq!(pace.per_second(), None);
        pace.note(0.5, 0);
        pace.note(1.0, 10);
        assert_eq!(pace.per_second(), None);
        // 20 frames a second from here on; the slow start is forgotten after the window.
        for s in 2..=30 {
            pace.note(f64::from(s), 10 + 20 * (s as u64 - 1));
        }
        let rate = pace.per_second().unwrap();
        assert!((rate - 20.0).abs() < 0.01, "{rate}");
    }

    #[test]
    fn the_time_left_follows_from_the_percent() {
        // 25 % done after 300 frames: 900 left, at 30 a second 30 seconds.
        assert_eq!(seconds_left(25.0, 300, 30.0), Some(30));
        assert_eq!(seconds_left(0.0, 0, 30.0), None);
        assert_eq!(seconds_left(100.0, 600, 30.0), Some(0));
        assert_eq!(clock_text(5), "0:05");
        assert_eq!(clock_text(125), "2:05");
        assert_eq!(clock_text(3725), "1:02:05");
    }

    #[test]
    fn progress_is_two_numbers() {
        assert_eq!(parse_progress("42 1234\n"), Some((42.0, 1234)));
        assert_eq!(parse_progress("42.5 99\n"), Some((42.5, 99)));
        assert_eq!(parse_progress("42"), None);
        assert_eq!(parse_progress(""), None);
    }
}
