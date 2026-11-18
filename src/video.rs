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
//! otherwise) and with no sound device.
//!
//! The sound is what the game's mixer makes, taken in step with the frames: after each frame the
//! audio thread takes one turn (every command the race sent it, the streams fed) and the mixer
//! is asked for the samples of that frame's stretch of time (44100 a second, whatever the frame
//! rate). They are written to a WAV file beside the video; when the frames are done `ffmpeg`
//! joins the two (the picture copied, the sound AAC, or Opus in a `.webm`). With no `ffmpeg` the
//! WAV file stays in the PNG folder.

use rc2k_core::race::render_dialog::Quality;
use rc2k_core::system::program_switches::Switches;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;

/// What the command line asked to be rendered.
struct Job {
    out: PathBuf,
    /// The race screen's size for the render (`--size`).
    size: Option<(u32, u32)>,
    fps: u32,
    quality: Quality,
    /// Whether the video has the game's sound.
    sound: bool,
}

/// The render the command line asked for (it lasts until the program ends).
static JOB: RwLock<Option<Arc<Job>>> = RwLock::new(None);

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
            ("--quality", switches.quality.is_some()),
            ("--no-sound", switches.no_sound),
            ("--from", switches.from_cs.is_some()),
            ("--to", switches.to_cs.is_some()),
        ] {
            if given {
                return Err(format!("{name} goes with --render-replay"));
            }
        }
        return Ok(());
    }
    let out = switches
        .out
        .as_deref()
        .ok_or("--render-replay needs --out <video file>")?;
    set_job(Some(Job {
        out: PathBuf::from(out),
        size: switches.size,
        fps: switches.fps.unwrap_or(DEFAULT_FPS),
        quality: switches.quality.unwrap_or_default(),
        sound: !switches.no_sound,
    }));
    RANGE.store(pack_range(switches.from_cs, switches.to_cs), Relaxed);
    ACTIVE.store(true, Relaxed);
    CLOCK_BASE.store(CLOCK_START, Relaxed);
    // No window on the screen and no sound for a run that makes a file.
    if std::env::var_os("SDL_VIDEODRIVER").is_none() {
        crate::platform::ffi::sdl::use_offscreen_video();
    }
    if std::env::var_os("SDL_AUDIODRIVER").is_none() {
        crate::platform::ffi::sdl::use_dummy_audio();
    }
    Ok(())
}

fn set_job(job: Option<Job>) {
    *JOB.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = job.map(Arc::new);
}

fn job() -> Option<Arc<Job>> {
    JOB.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// A replay is being rendered.
pub fn active() -> bool {
    ACTIVE.load(Relaxed)
}

static ACTIVE: AtomicBool = AtomicBool::new(false);

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
    (frames != 0).then(|| {
        CLOCK_BASE
            .load(Relaxed)
            .wrapping_add(frames_to_ms(frames - 1, job.fps) as u32)
    })
}

/// Where the clock of the render going on starts.
static CLOCK_BASE: AtomicU32 = AtomicU32::new(CLOCK_START);

/// The time of frame `n` of a video of `fps` frames a second, in whole milliseconds.
fn frames_to_ms(n: u64, fps: u32) -> u64 {
    n * 1000 / u64::from(fps)
}

/// The video's clock runs (the race's loop has begun): the device's audio callback then gives
/// silence, since the sound is taken frame by frame instead (`sound_turn`).
pub fn sound_taken_by_frames() -> bool {
    active() && FRAMES.load(Relaxed) != 0
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

// ---- the part rendered, and how far it has come ----

/// The part of the replay to render, in hundredths of a second from its start, packed:
/// `from` in the high half and `to` (0: to the end) in the low half, each capped at 65535.
static RANGE: AtomicU32 = AtomicU32::new(0);

fn pack_range(from: Option<u32>, to: Option<u32>) -> u32 {
    (from.unwrap_or(0).min(0xFFFF) << 16) | to.unwrap_or(0).min(0xFFFF)
}

/// The part `--from` and `--to` asked for: (from, to) in hundredths of a second, `to` none for
/// the replay's end; none when the whole replay is to be rendered.
pub fn range() -> Option<(u32, Option<u32>)> {
    let packed = RANGE.load(Relaxed);
    let (from, to) = (packed >> 16, packed & 0xFFFF);
    (packed != 0).then_some((from, (to != 0).then_some(to)))
}

/// When the progress was last written.
static PROGRESS_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// A render that a parent program watches (`RC2K_RENDER_PROGRESS` names a file) writes how far it
/// has come, `fraction` of the part, about every half second: a line "percent frames".
pub fn note_progress(fraction: f64) {
    let Some(path) = std::env::var_os("RC2K_RENDER_PROGRESS") else {
        return;
    };
    let mut at = PROGRESS_AT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if at.is_some_and(|t| t.elapsed() < std::time::Duration::from_millis(500)) {
        return;
    }
    *at = Some(std::time::Instant::now());
    let frames = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .map_or(0, |w| w.video_frames);
    let line = format!(
        "{} {frames}\n",
        (fraction.clamp(0.0, 1.0) * 100.0).round() as u32
    );
    // Written to a name of its own and renamed, so that a reader never sees half a line.
    let mut temp = path.clone();
    temp.push(".new");
    if std::fs::write(&temp, line).is_ok() {
        let _ = std::fs::rename(&temp, &path);
    }
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
    /// The sound's WAV file.
    sound: Option<Wav>,
    /// Frames of the video taken so far (the sound follows them).
    video_frames: u64,
    /// Frames of sound drawn from the mixer and thrown away (a video without sound).
    sound_skipped: u64,
    /// Where `ffmpeg` writes the picture alone, to be joined with the sound at the end.
    silent: Option<PathBuf>,
    /// The tools `ffmpeg` has, when there is one.
    tools: Option<Tools>,
}

/// The sound's output rate (the mixer's).
const SOUND_RATE: u32 = 44100;

/// A 16-bit stereo WAV file being written; its header is made right at the end.
struct Wav {
    file: std::fs::File,
    path: PathBuf,
    frames: u64,
}

impl Wav {
    fn create(path: PathBuf) -> std::io::Result<Self> {
        let mut file = std::fs::File::create(&path)?;
        file.write_all(&[0u8; 44])?;
        Ok(Self {
            file,
            path,
            frames: 0,
        })
    }

    fn add(&mut self, pcm: &[i16]) -> std::io::Result<()> {
        let bytes: Vec<u8> = pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
        self.file.write_all(&bytes)?;
        self.frames += (pcm.len() / 2) as u64;
        Ok(())
    }

    /// The header written (the sizes known now) and the file closed.
    fn close(mut self) -> std::io::Result<PathBuf> {
        let data = (self.frames * 4) as u32;
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&wav_header(data))?;
        Ok(self.path)
    }
}

/// A WAV header for `data` bytes of 16-bit stereo at `SOUND_RATE`.
fn wav_header(data: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(36 + data).to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&2u16.to_le_bytes());
    header.extend_from_slice(&SOUND_RATE.to_le_bytes());
    header.extend_from_slice(&(SOUND_RATE * 4).to_le_bytes());
    header.extend_from_slice(&4u16.to_le_bytes());
    header.extend_from_slice(&16u16.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data.to_le_bytes());
    header
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
    let writer = writer.get_or_insert_with(|| start_writer(&job, width, height));
    if writer.frames.send(pixels.to_vec()).is_err() {
        crate::logfile::program_log::log("video: the writer stopped; the frame is dropped");
    }
    writer.video_frames += 1;
}

/// The sound of the frame just taken: the audio thread takes a turn (so every sound the race
/// asked for is in the mixer), then the mixer's samples for the frame's stretch of time go to
/// the sound file.
pub fn sound_turn(audio: &crate::audio::owner::Audio) {
    let Some(job) = job() else { return };
    // The audio thread takes the commands queued and ends a turn; the wait is bounded so a
    // stuck thread cannot hold the render.
    crate::sound::queue::send_command_in(audio, crate::sound::queue::END_OF_TURN);
    crate::sound::queue::wait_a_while(audio, std::time::Duration::from_millis(250));
    let mut writer = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(writer) = writer.as_mut() else {
        return;
    };
    let due = sound_frames_due(writer.video_frames, job.fps);
    let Some(wav) = writer.sound.as_mut() else {
        if !job.sound {
            // No sound in the video, but the mixer is still drawn from so that it keeps up with
            // the frames.
            let wanted = due.saturating_sub(writer.sound_skipped) as usize;
            if wanted > 0 {
                drop(crate::platform::sound::mixed_frames(wanted));
                writer.sound_skipped = due;
            }
        }
        return;
    };
    let wanted = due.saturating_sub(wav.frames) as usize;
    if wanted == 0 {
        return;
    }
    let pcm = crate::platform::sound::mixed_frames(wanted);
    if let Err(e) = wav.add(&pcm) {
        log(&format!(
            "{}: {e}; the sound stops here",
            wav.path.display()
        ));
        writer.sound = None;
    }
}

/// The frames of sound that belong to the first `video_frames` frames of a video of `fps`.
fn sound_frames_due(video_frames: u64, fps: u32) -> u64 {
    video_frames * u64::from(SOUND_RATE) / u64::from(fps)
}

/// The video's file closed (the program ends): the frames still waiting written first.
pub fn finish() {
    let writer = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let Some(Writer {
        frames,
        thread,
        sound,
        silent,
        tools,
        ..
    }) = writer
    else {
        if active() {
            crate::logfile::program_log::log("video: no frame was drawn; nothing written");
        }
        return;
    };
    drop(frames);
    let count = thread.join().unwrap_or(0);
    crate::logfile::program_log::log(&format!("video: {count} frames written"));
    let wav = sound.and_then(|wav| {
        let seconds = wav.frames as f64 / f64::from(SOUND_RATE);
        wav.close()
            .inspect(|path| log(&format!("{}: {seconds:.2} s of sound", path.display())))
            .ok()
    });
    if let (Some(silent), Some(wav), Some(tools), Some(job)) = (silent, wav, tools, job()) {
        join_sound(&silent, &wav, &job.out, tools);
    }
}

/// The picture and the sound joined into the video file; the two parts removed when it is made.
/// If the join fails the picture alone becomes the video.
fn join_sound(silent: &Path, wav: &Path, out: &Path, tools: Tools) {
    let args = join_args(silent, wav, out, tools);
    let made = Command::new("ffmpeg")
        .args(&args)
        .stdin(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if made {
        let _ = std::fs::remove_file(silent);
        let _ = std::fs::remove_file(wav);
        log(&format!("{}: picture and sound joined", out.display()));
    } else {
        log("the sound could not be joined; the video has no sound");
        let _ = std::fs::rename(silent, out);
        let _ = std::fs::remove_file(wav);
    }
}

/// The arguments that join a picture file and a WAV file: the picture copied, the sound AAC (Opus
/// in a `.webm`) if `ffmpeg` has the encoder, else its default for the file's extension.
fn join_args(silent: &Path, wav: &Path, out: &Path, tools: Tools) -> Vec<String> {
    let webm = out
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("webm"));
    let mut args: Vec<String> = ["-hide_banner", "-loglevel", "error", "-y", "-i"]
        .map(String::from)
        .to_vec();
    args.push(silent.to_string_lossy().into_owned());
    args.extend(["-i".into(), wav.to_string_lossy().into_owned()]);
    args.extend(["-map", "0:v", "-map", "1:a", "-c:v", "copy"].map(String::from));
    match (webm, tools) {
        (true, Tools { opus: true, .. }) => {
            args.extend(["-c:a", "libopus", "-b:a", "160k"].map(String::from));
        }
        (false, Tools { aac: true, .. }) => {
            args.extend(["-c:a", "aac", "-b:a", "192k"].map(String::from));
        }
        _ => {}
    }
    args.push(out.to_string_lossy().into_owned());
    args
}

fn log(text: &str) {
    crate::logfile::program_log::log(&format!("video: {text}"));
}

/// The encoders an `ffmpeg` has that the render uses.
#[derive(Clone, Copy)]
struct Tools {
    h264: bool,
    aac: bool,
    opus: bool,
}

/// Whether an `ffmpeg` is on the path (the render's dialog says if it is not).
pub fn encoder_found() -> bool {
    find_ffmpeg().is_some()
}

/// An `ffmpeg` on the path, and the encoders it has.
fn find_ffmpeg() -> Option<Tools> {
    let listed = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !listed.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&listed.stdout);
    let has = |name: &str| text.split_whitespace().any(|word| word == name);
    Some(Tools {
        h264: has("libx264"),
        aac: has("aac"),
        opus: has("libopus"),
    })
}

/// The arguments that turn raw frames into the file: the size and rate of the input, H.264 at the
/// job's quality in the usual 4:2:0 (an odd side cut by one pixel) when it is there, else the
/// file's default.
fn ffmpeg_args(out: &Path, width: u32, height: u32, job: &Job, h264: bool) -> Vec<String> {
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
    args.extend(["-r".into(), job.fps.to_string(), "-i".into(), "-".into()]);
    if h264 {
        args.extend(["-vf", "crop=trunc(iw/2)*2:trunc(ih/2)*2"].map(String::from));
        args.extend(["-c:v", "libx264", "-preset", "medium", "-crf"].map(String::from));
        args.push(job.quality.crf().to_string());
        args.extend(["-pix_fmt", "yuv420p"].map(String::from));
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

/// Everything a render of `out` leaves in the folder while it runs or after it stopped halfway:
/// the video, the picture and the sound it is made from, and the folder of numbered pictures.
pub fn leftovers(out: &Path) -> Vec<PathBuf> {
    vec![
        out.to_path_buf(),
        silent_path(out),
        sound_path(out),
        frames_folder(out),
    ]
}

/// Where the picture goes alone while the sound is made, beside the video.
fn silent_path(out: &Path) -> PathBuf {
    let stem = out
        .file_stem()
        .map_or_else(|| "video".into(), |s| s.to_string_lossy().into_owned());
    let ext = out
        .extension()
        .map_or_else(String::new, |e| format!(".{}", e.to_string_lossy()));
    out.with_file_name(format!("{stem}.picture{ext}"))
}

/// The sound's file beside the video (joined into it at the end).
fn sound_path(out: &Path) -> PathBuf {
    out.with_extension("sound.wav")
}

/// The sink, the picture's own file when sound is to be joined to it, and the encoders found.
fn open_sink(job: &Job, width: u32, height: u32) -> (Sink, Option<PathBuf>, Option<Tools>) {
    match find_ffmpeg() {
        Some(tools) => {
            // With an encoder for the sound the picture goes to a file of its own first.
            let silent = (job.sound && (tools.aac || tools.opus)).then(|| silent_path(&job.out));
            let target = silent.as_deref().unwrap_or(&job.out);
            let args = ffmpeg_args(target, width, height, job, tools.h264);
            match Command::new("ffmpeg")
                .args(&args)
                .stdin(Stdio::piped())
                .spawn()
            {
                Ok(child) => {
                    log(&format!(
                        "ffmpeg ({}) writes {}: {width} x {height}, {} frames a second",
                        if tools.h264 {
                            "H.264"
                        } else {
                            "its default codec"
                        },
                        job.out.display(),
                        job.fps
                    ));
                    return (Sink::Ffmpeg(child), silent, Some(tools));
                }
                Err(e) => log(&format!("ffmpeg did not start ({e})")),
            }
        }
        None => log("no ffmpeg on the path"),
    }
    let folder = frames_folder(&job.out);
    log(&format!(
        "writing numbered PNG files in {} instead (make the video from them with an encoder: {} frames a second; the sound is audio.wav there)",
        folder.display(),
        job.fps
    ));
    (Sink::Pngs(folder), None, None)
}

fn start_writer(job: &Job, width: u32, height: u32) -> Writer {
    let (sink, silent, tools) = open_sink(job, width, height);
    let sound_file = match (&sink, &silent) {
        (_, _) if !job.sound => None,
        (Sink::Pngs(folder), _) => {
            let _ = std::fs::create_dir_all(folder);
            Some(folder.join("audio.wav"))
        }
        (_, Some(_)) => Some(sound_path(&job.out)),
        _ => None,
    };
    let sound = sound_file.and_then(|path| match Wav::create(path.clone()) {
        Ok(wav) => Some(wav),
        Err(e) => {
            log(&format!("{}: {e}; no sound", path.display()));
            None
        }
    });
    // A few frames wait, so drawing goes on while the last ones are being encoded.
    let (frames, queue) = sync_channel::<Vec<u8>>(4);
    let thread = std::thread::spawn(move || write_frames(sink, &queue, width, height));
    Writer {
        frames,
        thread,
        sound,
        video_frames: 0,
        sound_skipped: 0,
        silent,
        tools,
    }
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
        let args = ffmpeg_args(Path::new("v.mp4"), 1281, 720, &job(60), true);
        let at = |flag: &str| args.iter().position(|a| a == flag).unwrap();
        assert_eq!(args[at("-s") + 1], "1281x720");
        assert_eq!(args[at("-r") + 1], "60");
        assert_eq!(args.last().unwrap(), "v.mp4");
        assert!(args.iter().any(|a| a == "libx264"));
        let plain = ffmpeg_args(Path::new("v.mkv"), 640, 480, &job(30), false);
        assert!(!plain.iter().any(|a| a == "libx264"));
        assert!(!plain.iter().any(|a| a == "-vf"));
    }

    fn job(fps: u32) -> Job {
        Job {
            out: "v.mp4".into(),
            size: None,
            fps,
            quality: Quality::Medium,
            sound: true,
        }
    }

    #[test]
    fn the_chosen_quality_reaches_the_encoder() {
        let args = ffmpeg_args(Path::new("v.mp4"), 800, 600, &job(60), true);
        let at = |flag: &str| args.iter().position(|a| a == flag).unwrap();
        assert_eq!(args[at("-crf") + 1], "23");
    }

    #[test]
    fn the_part_is_packed_and_read_back() {
        assert_eq!(pack_range(None, None), 0);
        let packed = pack_range(Some(1250), Some(3000));
        assert_eq!((packed >> 16, packed & 0xFFFF), (1250, 3000));
        assert_eq!(pack_range(Some(70_000), None) >> 16, 0xFFFF);
    }

    #[test]
    fn sound_follows_the_frames() {
        // 60 frames a second: 735 samples a frame; 24 frames a second: 1837.5 (whole in sum).
        assert_eq!(sound_frames_due(1, 60), 735);
        assert_eq!(sound_frames_due(60, 60), 44100);
        assert_eq!(sound_frames_due(24, 24), 44100);
        assert_eq!(sound_frames_due(1, 24), 1837);
    }

    #[test]
    fn the_wav_header_says_its_size() {
        let header = wav_header(400);
        assert_eq!(&header[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(header[4..8].try_into().unwrap()), 436);
        assert_eq!(u32::from_le_bytes(header[40..44].try_into().unwrap()), 400);
        assert_eq!(header.len(), 44);
    }

    #[test]
    fn the_join_copies_the_picture_and_encodes_the_sound() {
        let all = Tools {
            h264: true,
            aac: true,
            opus: true,
        };
        let args = join_args(
            Path::new("a.picture.mp4"),
            Path::new("a.wav"),
            Path::new("a.mp4"),
            all,
        );
        assert!(args.windows(2).any(|w| w == ["-c:a", "aac"]));
        let args = join_args(
            Path::new("a.picture.webm"),
            Path::new("a.wav"),
            Path::new("a.webm"),
            all,
        );
        assert!(args.windows(2).any(|w| w == ["-c:a", "libopus"]));
        assert_eq!(args.last().unwrap(), "a.webm");
        assert_eq!(
            silent_path(Path::new("out/run.mp4")),
            Path::new("out/run.picture.mp4")
        );
    }

    #[test]
    fn the_frames_folder_is_named_after_the_video() {
        assert_eq!(
            frames_folder(Path::new("out/run.mp4")),
            Path::new("out/run_frames")
        );
    }
}
