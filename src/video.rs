//! A replay rendered to a video (`--render-replay <file> --out <video> --size WxH --fps N`).
//!
//! The replay plays as it does in the viewer, tick for tick, but on a clock of its own: each
//! frame drawn moves the game's clock on by one frame of the video (`1000 / fps` milliseconds),
//! so the run goes as fast as frames can be drawn, whatever the video's rate. The simulation
//! ticks a hundred times a second; a frame that falls between two ticks shows the blend of the
//! two (`race::blend`), so any frame rate gives a smooth picture, and the ticks themselves are
//! the same as when the replay is watched.
//!
//! The frames are written as raw pixels to an `ffmpeg` found on the path (H.264 when it has
//! `libx264`, else its own choice for the file's extension). With none, the frames are written
//! as numbered PNG files in a folder named after the video, and the log says so. The program runs
//! without a window of its own on screen (SDL's offscreen driver, unless `SDL_VIDEODRIVER` says
//! otherwise) and with no sound.

use rc2k_core::system::program_switches::Switches;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Mutex, OnceLock};
use std::thread::JoinHandle;

/// What the command line asked to be rendered.
struct Job {
    out: PathBuf,
    size: Option<(u32, u32)>,
    fps: u32,
}

static JOB: OnceLock<Option<Job>> = OnceLock::new();

/// The frame rate when `--fps` is not given.
pub const DEFAULT_FPS: u32 = 60;

/// The program's switches read: whether a replay is to be rendered. `Err`: why it cannot be
/// (a render with nowhere to put the video).
pub fn begin(switches: &Switches) -> Result<(), String> {
    if switches.render_replay.is_none() {
        for (name, given) in [
            ("--out", switches.out.is_some()),
            ("--size", switches.size.is_some()),
            ("--fps", switches.fps.is_some()),
        ] {
            if given {
                return Err(format!("{name} goes with --render-replay"));
            }
        }
        let _ = JOB.set(None);
        return Ok(());
    }
    let out = switches
        .out
        .as_deref()
        .ok_or("--render-replay needs --out <video file>")?;
    let _ = JOB.set(Some(Job {
        out: PathBuf::from(out),
        size: switches.size,
        fps: switches.fps.unwrap_or(DEFAULT_FPS),
    }));
    // No window on the screen and no sound for a run that makes a file.
    if std::env::var_os("SDL_VIDEODRIVER").is_none() {
        crate::platform::ffi::sdl::use_offscreen_video();
    }
    if std::env::var_os("SDL_AUDIODRIVER").is_none() {
        crate::platform::ffi::sdl::use_dummy_audio();
    }
    Ok(())
}

fn job() -> Option<&'static Job> {
    JOB.get().and_then(Option::as_ref)
}

/// A replay is being rendered.
pub fn active() -> bool {
    job().is_some()
}

/// The size the video's frames are (the race screen's size), if `--size` gave one.
pub fn size() -> Option<(u32, u32)> {
    job().and_then(|job| job.size)
}

// ---- the clock ----

/// The video's own clock starts here: later than any the menus before it read (the game's clock
/// starts at 0x10000), so the clock never runs backwards.
const CLOCK_START: u32 = 0x0100_0000;

/// Frames drawn since the race's loop began, plus one while the clock runs (0: it does not).
static FRAMES: AtomicU64 = AtomicU64::new(0);

/// The game's clock in milliseconds while a replay is rendered and its race has begun. After
/// the loop ends it stays where the last frame left it.
pub fn clock() -> Option<u32> {
    let frames = FRAMES.load(Relaxed);
    let job = job()?;
    (frames != 0).then(|| CLOCK_START + (frames_to_ms(frames - 1, job.fps)) as u32)
}

/// The time of frame `n` of a video of `fps` frames a second, in whole milliseconds.
fn frames_to_ms(n: u64, fps: u32) -> u64 {
    n * 1000 / u64::from(fps)
}

/// The race's loop is about to run: the clock starts.
pub fn race_loop_begins() {
    if active() {
        FRAMES.store(1, Relaxed);
    }
}

/// Whether the frame being shown is one of the race's (set when it is drawn, taken when it is
/// shown), so a menu's frame is not in the video.
static RACE_FRAME: AtomicBool = AtomicBool::new(false);

/// A race frame has been drawn and is about to be shown: it goes into the video and the clock
/// moves on one frame.
pub fn frame_drawn() {
    if active() && FRAMES.load(Relaxed) != 0 {
        FRAMES.fetch_add(1, Relaxed);
        RACE_FRAME.store(true, Relaxed);
    }
}

/// The frame about to be shown is wanted for the video.
pub fn frame_wanted() -> bool {
    active() && RACE_FRAME.load(Relaxed)
}

// ---- the file ----

/// How the frames are written.
enum Sink {
    /// Raw frames into an `ffmpeg`'s input.
    Ffmpeg(Child),
    /// Numbered PNG files in a folder.
    Pngs(PathBuf),
}

struct Writer {
    /// The frames wait here for the thread that writes them.
    frames: SyncSender<Vec<u8>>,
    thread: JoinHandle<u32>,
}

static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

/// The frame's pixels (32 bits a pixel, blue first) into the video. The first one starts the
/// file.
pub fn frame(pixels: &[u8], width: u32, height: u32) {
    RACE_FRAME.store(false, Relaxed);
    let Some(job) = job() else { return };
    let mut writer = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let writer = writer.get_or_insert_with(|| start_writer(job, width, height));
    if writer.frames.send(pixels.to_vec()).is_err() {
        crate::logfile::program_log::log("video: the writer stopped; the frame is dropped");
    }
}

/// The video's file closed (the program ends): the frames still waiting written first.
pub fn finish() {
    let writer = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let Some(Writer { frames, thread }) = writer else {
        if active() {
            crate::logfile::program_log::log("video: no frame was drawn; nothing written");
        }
        return;
    };
    drop(frames);
    let count = thread.join().unwrap_or(0);
    crate::logfile::program_log::log(&format!("video: {count} frames written"));
}

fn log(text: &str) {
    crate::logfile::program_log::log(&format!("video: {text}"));
}

/// An `ffmpeg` on the path, and whether it has the H.264 encoder.
fn find_ffmpeg() -> Option<bool> {
    let listed = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !listed.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&listed.stdout).contains("libx264"))
}

/// The arguments that turn raw frames into the file: the size and rate of the input, H.264
/// in the usual 4:2:0 (an odd side cut by one pixel) when it is there, else the file's default.
fn ffmpeg_args(out: &Path, width: u32, height: u32, fps: u32, h264: bool) -> Vec<String> {
    let mut args: Vec<String> = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "bgr0",
        "-s",
    ]
    .map(String::from)
    .to_vec();
    args.push(format!("{width}x{height}"));
    args.extend(["-r".into(), fps.to_string(), "-i".into(), "-".into()]);
    if h264 {
        args.extend(
            [
                "-vf",
                "crop=trunc(iw/2)*2:trunc(ih/2)*2",
                "-c:v",
                "libx264",
                "-preset",
                "medium",
                "-crf",
                "18",
                "-pix_fmt",
                "yuv420p",
            ]
            .map(String::from),
        );
    }
    args.push(out.to_string_lossy().into_owned());
    args
}

/// The folder the PNG frames go in: the video's name without its extension, and `_frames`.
fn frames_folder(out: &Path) -> PathBuf {
    let stem = out
        .file_stem()
        .map_or_else(|| "video".into(), |s| s.to_string_lossy().into_owned());
    out.with_file_name(format!("{stem}_frames"))
}

fn open_sink(job: &Job, width: u32, height: u32) -> Sink {
    match find_ffmpeg() {
        Some(h264) => {
            let args = ffmpeg_args(&job.out, width, height, job.fps, h264);
            match Command::new("ffmpeg")
                .args(&args)
                .stdin(Stdio::piped())
                .spawn()
            {
                Ok(child) => {
                    log(&format!(
                        "ffmpeg ({}) writes {}: {width} x {height}, {} frames a second",
                        if h264 { "H.264" } else { "its default codec" },
                        job.out.display(),
                        job.fps
                    ));
                    return Sink::Ffmpeg(child);
                }
                Err(e) => log(&format!("ffmpeg did not start ({e})")),
            }
        }
        None => log("no ffmpeg on the path"),
    }
    let folder = frames_folder(&job.out);
    log(&format!(
        "writing numbered PNG files in {} instead (make the video from them with an encoder: {} frames a second)",
        folder.display(),
        job.fps
    ));
    Sink::Pngs(folder)
}

fn start_writer(job: &Job, width: u32, height: u32) -> Writer {
    let sink = open_sink(job, width, height);
    // A few frames wait, so drawing goes on while the last ones are being encoded.
    let (frames, queue) = sync_channel::<Vec<u8>>(4);
    let thread = std::thread::spawn(move || write_frames(sink, &queue, width, height));
    Writer { frames, thread }
}

/// The frames from `queue` into the sink until the queue's end; how many were written.
fn write_frames(
    sink: Sink,
    queue: &std::sync::mpsc::Receiver<Vec<u8>>,
    width: u32,
    height: u32,
) -> u32 {
    let mut count = 0;
    match sink {
        Sink::Ffmpeg(mut child) => {
            let Some(mut input) = child.stdin.take() else {
                return 0;
            };
            for pixels in queue {
                // The frame is 32 bits a pixel whatever the video's size.
                if let Err(e) = input.write_all(&pixels) {
                    log(&format!("ffmpeg stopped taking frames ({e})"));
                    break;
                }
                count += 1;
            }
            drop(input);
            match child.wait() {
                Ok(status) if status.success() => {}
                Ok(status) => log(&format!("ffmpeg ended with {status}")),
                Err(e) => log(&format!("ffmpeg could not be waited for ({e})")),
            }
        }
        Sink::Pngs(folder) => {
            if let Err(e) = std::fs::create_dir_all(&folder) {
                log(&format!("{}: {e}", folder.display()));
                return 0;
            }
            for pixels in queue {
                let rgb: Vec<u8> = pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|p| [p[2], p[1], p[0]])
                    .collect();
                let png = rc2k_core::gfx::png_write::encode_rgb(width, height, &rgb);
                count += 1;
                if let Err(e) = std::fs::write(folder.join(format!("frame_{count:06}.png")), png) {
                    log(&format!("{}: {e}", folder.display()));
                    break;
                }
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_fall_where_the_rate_puts_them() {
        // 60 frames a second: 0, 16, 33, 50 ms, and a whole second after 60 frames.
        let ms: Vec<u64> = (0..4).map(|n| frames_to_ms(n, 60)).collect();
        assert_eq!(ms, [0, 16, 33, 50]);
        assert_eq!(frames_to_ms(60, 60), 1000);
        assert_eq!(frames_to_ms(144, 144), 1000);
        // Over a hundred a second some frames share a tick: the same millisecond is not
        // counted twice but the clock never goes back.
        let ms: Vec<u64> = (0..300).map(|n| frames_to_ms(n, 240)).collect();
        assert!(ms.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn the_encoder_gets_the_frames_size_and_rate() {
        let args = ffmpeg_args(Path::new("v.mp4"), 1281, 720, 60, true);
        let at = |flag: &str| args.iter().position(|a| a == flag).unwrap();
        assert_eq!(args[at("-s") + 1], "1281x720");
        assert_eq!(args[at("-r") + 1], "60");
        assert_eq!(args.last().unwrap(), "v.mp4");
        assert!(args.iter().any(|a| a == "libx264"));
        let plain = ffmpeg_args(Path::new("v.mkv"), 640, 480, 30, false);
        assert!(!plain.iter().any(|a| a == "libx264"));
    }

    #[test]
    fn the_frames_folder_is_named_after_the_video() {
        assert_eq!(
            frames_folder(Path::new("out/run.mp4")),
            Path::new("out/run_frames")
        );
    }
}
