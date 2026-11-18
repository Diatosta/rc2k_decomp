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
use rc2k_core::race::render_dialog::Quality;
use rc2k_core::strings::own::id;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// What is to be rendered.
pub struct Render {
    /// The copy of the replay as it is viewed (taken away when the render ends).
    pub replay: PathBuf,
    /// The video file.
    pub out: PathBuf,
    pub size: (u32, u32),
    pub fps: u32,
    pub quality: Quality,
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
}

/// The render going on until it ends or is stopped; a notice says how it ended.
pub fn run(o: Owners<'static>, render: &Render) {
    let ended = match start(render) {
        Ok(child) => watch(o, render, child),
        Err(why) => Ended::Failed(why),
    };
    for file in [&render.replay, &render.progress_file()] {
        let _ = std::fs::remove_file(file);
    }
    match ended {
        Ended::Done => {
            let name = render.out.file_name().unwrap_or_default().to_string_lossy();
            crate::error_dialog::notice(&crate::error_dialog::text_with(id::RENDER_DONE, &name));
        }
        Ended::Stopped => {}
        Ended::Failed(why) => {
            crate::error_dialog::notice(&crate::error_dialog::text_with(id::RENDER_FAILED, &why));
        }
    }
}

/// The child started: this program again, with no window or sound device of its own and no
/// dialog that would wait for a key nobody can press.
fn start(render: &Render) -> Result<Child, String> {
    let exe = std::env::current_exe().map_err(|e| format!("the program's file: {e}"))?;
    let _ = std::fs::remove_file(render.progress_file());
    let args = render.arguments();
    crate::logfile::program_log::log(&format!("render: {} {}", exe.display(), args.join(" ")));
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
fn read_progress(render: &Render) -> Option<(u32, u64)> {
    let text = std::fs::read_to_string(render.progress_file()).ok()?;
    parse_progress(&text)
}

fn parse_progress(text: &str) -> Option<(u32, u64)> {
    let mut words = text.split_whitespace();
    Some((words.next()?.parse().ok()?, words.next()?.parse().ok()?))
}

/// The panel up until the child ends or the render is stopped.
fn watch(o: Owners<'static>, render: &Render, mut child: Child) -> Ended {
    // A key from before is not an answer.
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    let mut progress = None;
    let mut was_down = true;
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
        crate::platform::pump_messages_now(o);
        if o.system.session.flags.quitting.is_set() {
            return stop(&mut child);
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() && render.out.is_file() => return Ended::Done,
            Ok(Some(status)) => {
                return Ended::Failed(format!("the render ended with {status}"));
            }
            Ok(None) => {}
            Err(e) => return Ended::Failed(e.to_string()),
        }
        progress = read_progress(render).or(progress);
        if o.gfx.locked.surface.get() == 1 {
            crate::platform::unlock_surface(o.gfx);
        }
        if crate::platform::lock_surface(o.gfx, o.system) < 1 {
            return stop(&mut child);
        }
        let button = draw(o, render, progress);
        crate::platform::unlock_surface(o.gfx);
        if !crate::platform::show_frame_now(o) {
            return stop(&mut child);
        }
        crate::input::menu_pad::take_answer(o.input);
        let key = crate::input::take_key::take_key(o.input);
        let (buttons, x, y) = crate::input::keys::unpack_mouse(&o.input.devices);
        let down = buttons & 1 != 0;
        let [l, t, w, h] = button;
        let clicked = down && !was_down && x >= l && x < l + w && y >= t && y < t + h;
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
fn draw(o: Owners<'static>, render: &Render, progress: Option<(u32, u64)>) -> [i32; 4] {
    use crate::gfx::button_hints::width_of;
    let mut canvas = GameCanvas::new(o.gfx);
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
    fill(o, [0, 0, w, h], SCREEN);
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
    let panel_h = MARGIN * 2 + line_h * 2 + line_h * 2 + row_h + line_h + row_h;
    let (left, top) = ((w - panel_w) / 2, (h - panel_h) / 2);
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
    let percent = progress.map_or(0, |(p, _)| p.min(100));
    fill(o, [left + MARGIN, y, inner, row_h - 8], BAR_TRACK);
    fill(
        o,
        [left + MARGIN, y, inner * percent as i32 / 100, row_h - 8],
        BAR_DONE,
    );
    y += row_h;
    let status = match progress {
        Some((p, frames)) => format!(
            "{}   {}",
            own(id::RENDER_PERCENT).replace("{}", &p.to_string()),
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
    y += line_h + row_h / 4;
    let button = [left + MARGIN, y, inner, row_h - 6];
    fill(o, button, BUTTON);
    let cancel = own(id::RENDER_CANCEL);
    put(
        &cancel,
        left + (panel_w - measure(&cancel)) / 2,
        y + (row_h - 6) / 2,
        &mut canvas,
    );
    button
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render() -> Render {
        Render {
            replay: "savedata/videos/r.replay.tmp".into(),
            out: "savedata/videos/r.mp4".into(),
            size: (2560, 1440),
            fps: 60,
            quality: Quality::High,
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
    fn progress_is_two_numbers() {
        assert_eq!(parse_progress("42 1234\n"), Some((42, 1234)));
        assert_eq!(parse_progress("42"), None);
        assert_eq!(parse_progress(""), None);
    }
}
