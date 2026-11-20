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

mod encoder;

use encoder::Encoder;
use rc2k_core::gfx::nv12::{even_size, frame_bytes, from_bgra, to_rgb};
use rc2k_core::race::render_dialog::{Quality, VideoEncoder};
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
    /// Whether the machine's own encoder may be used.
    encoder: VideoEncoder,
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
            ("--encoder", switches.encoder.is_some()),
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
        encoder: switches.encoder.unwrap_or_default(),
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
    crate::clock::tick_count::use_video_clock(clock);
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

/// The race's loop is about to run: the clock starts, and the device's audio callback gives
/// silence, since the sound is taken frame by frame instead (`sound_turn`).
pub(super) fn race_loop_begins() {
    if active() {
        FRAMES.store(1, Relaxed);
        crate::platform::sound::set_silent_for_video(true);
    }
}

/// Whether the frame being shown is one of the race's (set when it is drawn, taken when it is
/// shown), so a menu's frame is not in the video.
static RACE_FRAME: AtomicBool = AtomicBool::new(false);

/// A race frame has been drawn and is about to be shown: it goes into the video and the clock
/// moves on one frame.
pub(super) fn frame_drawn() {
    if active() && FRAMES.load(Relaxed) != 0 {
        FRAMES.fetch_add(1, Relaxed);
        RACE_FRAME.store(true, Relaxed);
    }
}

/// The frame about to be shown is wanted for the video.
pub(super) fn frame_wanted() -> bool {
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
/// has come, `fraction` of the part, about every half second: a line "percent frames" (the
/// percent with a decimal).
pub(super) fn note_progress(fraction: f64) {
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
    let line = format!("{:.1} {frames}\n", fraction.clamp(0.0, 1.0) * 100.0);
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
    /// When the first frame came.
    started: std::time::Instant,
    /// The picture file to look over at the end: a hardware encoder's.
    checked: Option<PathBuf>,
    /// The size of the screen's frames.
    size: (u32, u32),
    /// Whether the frames go to the encoder as NV12 (at the even size) and not as 32-bit pixels.
    nv12: bool,
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

/// Frame buffers the writer is done with, kept for the next frames: a new buffer of a whole
/// picture every frame costs more in fresh memory than the copy into it does.
static SPARE: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

/// How many spare buffers are kept (the queue's length and the ones in the writer's hands).
const SPARE_MOST: usize = 8;

/// A buffer for a frame's pixels: one from an earlier frame when there is one.
pub(super) fn spare_buffer() -> Vec<u8> {
    SPARE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .pop()
        .unwrap_or_default()
}

/// A copy of the frame's pixels, in a buffer from an earlier frame when there is one.
pub(super) fn copy_frame(pixels: &[u8]) -> Vec<u8> {
    let mut copy = spare_buffer();
    copy.resize(pixels.len(), 0);
    copy.copy_from_slice(pixels);
    copy
}

/// A frame the writer is done with, kept for `copy_frame`.
fn keep_buffer(buffer: Vec<u8>) {
    let mut spare = SPARE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if spare.len() < SPARE_MOST {
        spare.push(buffer);
    }
}

/// The parts of a frame's work that are timed.
#[derive(Clone, Copy)]
pub enum Stage {
    /// The picture brought back from the graphics card (with the Vulkan backend: the last
    /// frame's, and the new one's copy started).
    ReadBack,
    /// The picture copied out of the screen's memory (on the paths that bring it back whole).
    Copy,
    /// Waiting for room in the queue to the encoder (the encoder is the slower part).
    Queue,
    /// The sound's turn.
    Sound,
}

const STAGES: usize = 4;

/// The microseconds each stage has taken in all, for the log at the end.
static STAGE_MICROS: [AtomicU64; STAGES] = [const { AtomicU64::new(0) }; STAGES];

/// A stage of a frame took `time`.
pub(super) fn note_stage(stage: Stage, time: std::time::Duration) {
    STAGE_MICROS[stage as usize].fetch_add(time.as_micros() as u64, Relaxed);
}

/// A frame of the video has been taken (its pixels come with `send`, which may be later): the
/// frames and the sound that follows them are counted from here. The first one starts the file.
pub(super) fn frame_taken(width: u32, height: u32) {
    RACE_FRAME.store(false, Relaxed);
    let Some(job) = job() else { return };
    let mut writer = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let writer = writer.get_or_insert_with(|| start_writer(&job, width, height));
    writer.video_frames += 1;
}

/// A thing that went wrong that the player should be told when the render is over: a line of
/// `kind`, a tab and `detail` appended to the file beside the progress file, which the program
/// that started this one reads (`race::render_progress`). Kinds: `no_ffmpeg` (detail: the
/// folder the numbered pictures go to), `encoder` (its first error line), `sound`.
fn note_problem(kind: &str, detail: &str) {
    log(&format!("problem for the player: {kind}: {detail}"));
    let Some(path) = std::env::var_os("RC2K_RENDER_PROGRESS") else {
        return;
    };
    let line = format!("{kind}\t{}\n", detail.replace(['\n', '\r'], " "));
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::path::PathBuf::from(path).with_extension(PROBLEMS_EXTENSION))
    {
        let _ = file.write_all(line.as_bytes());
    }
}

/// The ending of the file the problems go in, in place of the progress file's.
pub(super) const PROBLEMS_EXTENSION: &str = "problems";

/// Whether the frames are made as NV12 (`render::video_read_next`): an `ffmpeg` is there to take
/// them and the card converts them. The writer's own answer once it has started.
pub fn nv12() -> bool {
    if let Some(writer) = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
    {
        return writer.nv12;
    }
    nv12_now()
}

/// `nv12` for a writer not yet started.
fn nv12_now() -> bool {
    static FFMPEG_FOUND: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    job().is_some()
        && *FFMPEG_FOUND.get_or_init(|| find_ffmpeg().is_some())
        && crate::render::video_nv12_possible()
}

/// The pixels of the next frame not yet sent to the video, in the order the frames were taken:
/// NV12 when the card made them, else 32 bits a pixel, blue first (the video's NV12 writer
/// converts those itself).
pub fn send(pixels: Vec<u8>) {
    let writer = WRITER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(writer) = writer.as_ref() else {
        return;
    };
    let began = std::time::Instant::now();
    let pixels =
        if writer.nv12 && pixels.len() != frame_bytes(even_size(writer.size.0, writer.size.1)) {
            let mut converted = spare_buffer();
            from_bgra(&pixels, writer.size.0, writer.size.1, &mut converted);
            keep_buffer(pixels);
            converted
        } else {
            pixels
        };
    if writer.frames.send(pixels).is_err() {
        crate::program_log::log("video: the writer stopped; the frame is dropped");
    }
    note_stage(Stage::Queue, began.elapsed());
}

/// The sound of the frame just taken: the audio thread takes a turn (so every sound the race
/// asked for is in the mixer), then the mixer's samples for the frame's stretch of time go to
/// the sound file.
pub(super) fn sound_turn(audio: &crate::audio::owner::Audio) {
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

/// The exit code of a render whose hardware encoder made a damaged picture (the video is
/// removed): the program that started it renders again with the software encoder.
pub(super) const DAMAGED: u32 = 4;

/// The video's file closed (the program ends): the frames still waiting written first. `Some`: the
/// exit code the render ends with when it did not make a good video.
pub fn finish() -> Option<u32> {
    // The last frame, still on its way back from the card.
    if let Some(pixels) = crate::render::video_read_last(spare_buffer()) {
        send(pixels);
    }
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
        started,
        checked,
        ..
    }) = writer
    else {
        if active() {
            crate::program_log::log("video: no frame was drawn; nothing written");
        }
        return None;
    };
    let looped = started.elapsed();
    drop(frames);
    let count = thread.join().unwrap_or(0);
    crate::program_log::log(&format!("video: {count} frames written"));
    log_times(count, looped, started.elapsed());
    if let Some(picture) = checked
        && let Err(why) = picture_is_whole(&picture)
    {
        log(&format!(
            "the hardware encoder's picture is damaged ({why}); the video is removed"
        ));
        if let Some(job) = job() {
            for file in leftovers(&job.out) {
                if file.is_dir() {
                    let _ = std::fs::remove_dir_all(&file);
                } else {
                    let _ = std::fs::remove_file(&file);
                }
            }
        }
        return Some(DAMAGED);
    }
    let wav = sound.and_then(|wav| {
        let seconds = wav.frames as f64 / f64::from(SOUND_RATE);
        wav.close()
            .inspect(|path| log(&format!("{}: {seconds:.2} s of sound", path.display())))
            .ok()
    });
    if let (Some(silent), Some(wav), Some(tools), Some(job)) = (silent, wav, tools, job()) {
        join_sound(&silent, &wav, &job.out, tools);
    }
    None
}

/// Whether `ffmpeg` decodes the whole picture file without a word of complaint. A decoder that
/// cannot be run is no proof of damage: the file is let through.
fn picture_is_whole(picture: &Path) -> Result<(), String> {
    let began = std::time::Instant::now();
    let checked = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(picture)
        .args(["-f", "null", "-"])
        .stdin(Stdio::null())
        .output();
    let Ok(checked) = checked else {
        log("the picture could not be checked");
        return Ok(());
    };
    log(&format!(
        "the picture checked in {} ms",
        began.elapsed().as_millis()
    ));
    let said = String::from_utf8_lossy(&checked.stderr);
    match said.lines().next() {
        None if checked.status.success() => Ok(()),
        None => Err(format!("the check ended with {}", checked.status)),
        Some(first) => Err(first.trim().to_owned()),
    }
}

/// The time the frames took, by stage, in the log: `looped` is the time from the first frame to
/// the last one handed to the encoder, `total` the time until the encoder had finished.
fn log_times(frames: u32, looped: std::time::Duration, total: std::time::Duration) {
    let ms = |stage: Stage| STAGE_MICROS[stage as usize].load(Relaxed) / 1000;
    let per_second = |time: std::time::Duration| f64::from(frames) / time.as_secs_f64().max(1e-3);
    let (read_back, copy, queue, sound) = (
        ms(Stage::ReadBack),
        ms(Stage::Copy),
        ms(Stage::Queue),
        ms(Stage::Sound),
    );
    let looped_ms = looped.as_millis() as u64;
    let rest = looped_ms.saturating_sub(read_back + copy + queue + sound);
    log(&format!(
        "{frames} frames: {:.1} a second while drawing ({looped_ms} ms), {:.1} a second to the \
         end of the encoding ({} ms); of the drawing time {read_back} ms read back from the \
         card, {copy} ms copied, {queue} ms waiting for the encoder, {sound} ms sound, {rest} ms \
         simulation and drawing",
        per_second(looped),
        per_second(total),
        total.as_millis()
    ));
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
        note_problem("sound", "");
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
    crate::program_log::log(&format!("video: {text}"));
}

/// The encoders an `ffmpeg` has that the render uses.
#[derive(Clone, Copy)]
struct Tools {
    h264: bool,
    aac: bool,
    opus: bool,
    /// The hardware encoders it lists that this system can have (`encoder::found_in`).
    hardware: u16,
}

/// Whether an `ffmpeg` is on the path (the render's dialog says if it is not).
pub(super) fn encoder_found() -> bool {
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
        hardware: encoder::found_in(&text),
    })
}

/// The arguments that turn raw frames into the file: the size and rate of the input, then what
/// the encoder needs (`encoder::encode_args`: H.264 at the job's quality in the usual
/// 4:2:0, an odd side cut by one pixel, or `ffmpeg`'s own choice for the file's extension).
fn ffmpeg_args(
    out: &Path,
    (width, height): (u32, u32),
    nv12: bool,
    job: &Job,
    encoder: Encoder,
) -> Vec<String> {
    let mut args: Vec<String> = ["-hide_banner", "-loglevel", "error", "-y"]
        .map(String::from)
        .to_vec();
    args.extend(encoder::device_args(encoder));
    let (format, (width, height)) = if nv12 {
        ("nv12", even_size(width, height))
    } else {
        ("bgr0", (width, height))
    };
    args.extend(["-f", "rawvideo", "-pix_fmt", format, "-s"].map(String::from));
    args.push(format!("{width}x{height}"));
    args.extend(["-r".into(), job.fps.to_string(), "-i".into(), "-".into()]);
    args.extend(encoder::encode_args(encoder, job.quality));
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
pub(super) fn leftovers(out: &Path) -> Vec<PathBuf> {
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

/// The sink and what goes with it.
fn open_sink(job: &Job, width: u32, height: u32, nv12: bool) -> Opened {
    match find_ffmpeg() {
        Some(tools) => {
            // With an encoder for the sound the picture goes to a file of its own first.
            let silent = (job.sound && (tools.aac || tools.opus)).then(|| silent_path(&job.out));
            let target = silent.as_deref().unwrap_or(&job.out);
            let encoder = encoder::choose(
                job.encoder == VideoEncoder::Automatic,
                tools.h264,
                tools.hardware,
                (width, height),
                job.quality,
                &log,
            );
            let args = ffmpeg_args(target, (width, height), nv12, job, encoder);
            match Command::new("ffmpeg")
                .args(&args)
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => {
                    log(&format!(
                        "ffmpeg ({}) writes {}: {width} x {height}, {} frames a second",
                        encoder.name(),
                        job.out.display(),
                        job.fps
                    ));
                    let checked =
                        matches!(encoder, Encoder::Hardware(_)).then(|| target.to_owned());
                    return Opened {
                        sink: Sink::Ffmpeg(child),
                        silent,
                        tools: Some(tools),
                        checked,
                    };
                }
                Err(e) => {
                    log(&format!("ffmpeg did not start ({e})"));
                    note_problem("encoder", &format!("ffmpeg did not start ({e})"));
                }
            }
        }
        None => {
            log("no ffmpeg on the path");
            note_problem(
                "no_ffmpeg",
                &frames_folder(&job.out)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
            );
        }
    }
    let folder = frames_folder(&job.out);
    log(&format!(
        "writing numbered PNG files in {} instead (make the video from them with an encoder: {} frames a second; the sound is audio.wav there)",
        folder.display(),
        job.fps
    ));
    Opened {
        sink: Sink::Pngs(folder),
        silent: None,
        tools: None,
        checked: None,
    }
}

/// What `open_sink` made.
struct Opened {
    sink: Sink,
    /// The picture's own file, when sound is to be joined to it.
    silent: Option<PathBuf>,
    /// The tools `ffmpeg` has.
    tools: Option<Tools>,
    /// The picture file to look over at the end (a hardware encoder's).
    checked: Option<PathBuf>,
}

fn start_writer(job: &Job, width: u32, height: u32) -> Writer {
    let nv12 = nv12_now();
    let Opened {
        sink,
        silent,
        tools,
        checked,
    } = open_sink(job, width, height, nv12);
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
        started: std::time::Instant::now(),
        checked,
        size: (width, height),
        nv12,
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
                keep_buffer(pixels);
                count += 1;
            }
            drop(input);
            let said = child.stderr.take().map(|mut pipe| {
                std::thread::spawn(move || {
                    let mut text = String::new();
                    let _ = std::io::Read::read_to_string(&mut pipe, &mut text);
                    text
                })
            });
            let status = child.wait();
            let said = said
                .and_then(|reader| reader.join().ok())
                .unwrap_or_default();
            let first = said.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            if !first.is_empty() {
                log(&format!("ffmpeg said: {first}"));
            }
            match status {
                Ok(status) if status.success() => {}
                Ok(status) => {
                    log(&format!("ffmpeg ended with {status}"));
                    let ended = status.to_string();
                    note_problem("encoder", if first.is_empty() { &ended } else { first });
                }
                Err(e) => log(&format!("ffmpeg could not be waited for ({e})")),
            }
        }
        Sink::Pngs(folder) => {
            if let Err(e) = std::fs::create_dir_all(&folder) {
                log(&format!("{}: {e}", folder.display()));
                return 0;
            }
            for pixels in queue {
                // A frame the card converted (an encoder was expected) is NV12 at the even size.
                let (rgb, (width, height)): (Vec<u8>, _) =
                    if pixels.len() == width as usize * height as usize * 4 {
                        let rgb = pixels
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .flat_map(|p| [p[2], p[1], p[0]])
                            .collect();
                        (rgb, (width, height))
                    } else {
                        let size = even_size(width, height);
                        (to_rgb(&pixels, size), size)
                    };
                let png = rc2k_core::gfx::png_write::encode_rgb(width, height, &rgb);
                count += 1;
                if let Err(e) = std::fs::write(folder.join(format!("frame_{count:06}.png")), png) {
                    log(&format!("{}: {e}", folder.display()));
                    break;
                }
                keep_buffer(pixels);
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
        let args = ffmpeg_args(
            Path::new("v.mp4"),
            (1281, 720),
            false,
            &job(60),
            Encoder::Software,
        );
        let at = |flag: &str| args.iter().position(|a| a == flag).unwrap();
        assert_eq!(args[at("-s") + 1], "1281x720");
        assert_eq!(args[at("-r") + 1], "60");
        assert_eq!(args.last().unwrap(), "v.mp4");
        assert!(args.iter().any(|a| a == "libx264"));
        let plain = ffmpeg_args(
            Path::new("v.mkv"),
            (640, 480),
            false,
            &job(30),
            Encoder::Default,
        );
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
            encoder: VideoEncoder::Automatic,
        }
    }

    #[test]
    fn the_chosen_quality_reaches_the_encoder() {
        let args = ffmpeg_args(
            Path::new("v.mp4"),
            (800, 600),
            false,
            &job(60),
            Encoder::Software,
        );
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
            hardware: 0,
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
