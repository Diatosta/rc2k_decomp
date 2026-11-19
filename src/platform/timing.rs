//! Where the time goes: the frames' counts and timings in the log, and on Windows the main
//! thread and the rasteriser's sampled (with `RC2K_PROFILE` set).

use crate::platform::SHOWN;
use crate::program_log::log;
use crate::system::owner::System;
use std::sync::atomic::Ordering;

// ---- where the time goes: the main thread and the rasteriser's sampled every millisecond ----

/// Where the sampled threads were, by address, since the last report.
#[cfg(windows)]
static SAMPLES: std::sync::Mutex<Option<std::collections::HashMap<usize, u32>>> =
    std::sync::Mutex::new(None);

/// The time putting frames on the screen (`show_frame`), in microseconds.
pub static SHOW_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Since the last report: the longest time between two frames shown and the frames that
/// took over 50 ms (stutters), in microseconds and a count; the longest time between two
/// race sound turns (`sound::race_turn`), in microseconds.
static LONGEST_FRAME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SLOW_FRAMES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static LONGEST_TURN_GAP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The GPU path's waits, in microseconds, for the phones' log line (`phone_line`): the most
/// and the total of the time in the window renderer's present (where SDL waits for the
/// swapchain), in the frame's finish and in a readback of the frame (`render::bring_back`),
/// and in each flush of the 3D (recording and submitting a command buffer); and the longest
/// show of one frame. Written by the game's thread only.
#[cfg_attr(
    handheld,
    allow(
        dead_code,
        reason = "noted by the GPU renderer and the present, not on the handheld"
    )
)]
pub static PRESENT_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg_attr(
    handheld,
    allow(
        dead_code,
        reason = "noted by the GPU renderer and the present, not on the handheld"
    )
)]
pub static PRESENT_MAX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg_attr(
    handheld,
    allow(
        dead_code,
        reason = "noted by the GPU renderer and the present, not on the handheld"
    )
)]
pub static FINISH_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg_attr(
    handheld,
    allow(
        dead_code,
        reason = "noted by the GPU renderer and the present, not on the handheld"
    )
)]
pub static FINISH_MAX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static READ_BACK_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static READ_BACK_MAX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg_attr(
    handheld,
    allow(
        dead_code,
        reason = "noted by the GPU renderer and the present, not on the handheld"
    )
)]
pub static FLUSH_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg_attr(
    handheld,
    allow(
        dead_code,
        reason = "noted by the GPU renderer and the present, not on the handheld"
    )
)]
pub static FLUSH_MAX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg_attr(
    handheld,
    allow(
        dead_code,
        reason = "noted by the GPU renderer and the present, not on the handheld"
    )
)]
pub static FLUSH_RUNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static SHOW_MAX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A span of `micros` noted in a total and a most.
pub fn note_span(
    total: &std::sync::atomic::AtomicU64,
    most: &std::sync::atomic::AtomicU64,
    micros: u64,
) {
    total.fetch_add(micros, Ordering::Relaxed);
    most.fetch_max(micros, Ordering::Relaxed);
}

/// The microseconds since `last` was noted, `last` now.
fn since(last: &std::sync::Mutex<Option<std::time::Instant>>) -> Option<u64> {
    let now = std::time::Instant::now();
    let mut last = last.lock().ok()?;
    let gap = last.map(|at| now.duration_since(at).as_micros() as u64);
    *last = Some(now);
    gap
}

/// A frame shown (`platform`): its time since the last one noted.
pub fn note_frame_shown() {
    use std::sync::atomic::Ordering::Relaxed;
    static LAST: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    if let Some(gap) = since(&LAST) {
        LONGEST_FRAME.fetch_max(gap, Relaxed);
        if gap > 50_000 {
            SLOW_FRAMES.fetch_add(1, Relaxed);
        }
    }
}

/// A race sound turn (`sound::race_turn`): its time since the last one noted.
pub fn note_sound_turn() {
    static LAST: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    if let Some(gap) = since(&LAST) {
        LONGEST_TURN_GAP.fetch_max(gap, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The threads the sampler looks at: the main one and the rasteriser's.
#[cfg(windows)]
static SAMPLED: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

/// The rasteriser's threads (rayon's pool), each noted for the sampler when it starts.
pub fn start_threads() {
    let _ = rayon::ThreadPoolBuilder::new()
        .start_handler(|_| {
            // The sampler's ids are the system's own (it opens the threads by them).
            #[cfg(windows)]
            SAMPLED
                .lock()
                .unwrap()
                .push(crate::platform::ffi::win32::current_thread_id());
        })
        .build_global();
}

/// No sampler off Windows (it stops threads through Win32).
#[cfg(not(windows))]
pub fn start_sampler() {}

/// No samples off Windows.
#[cfg(not(windows))]
fn hot_places() -> String {
    String::new()
}

/// Starts the sampler, once (from the main thread), if `RC2K_PROFILE` is set: a thread of its
/// own stops each sampled thread every millisecond, notes its instruction pointer and lets it
/// go on. `rc2k_profile.txt` is emptied as it starts.
#[cfg(windows)]
pub fn start_sampler() {
    use crate::platform::ffi::win32::SampledThread;
    use std::sync::Once;
    static STARTED: Once = Once::new();
    STARTED.call_once(|| {
        if std::env::var_os("RC2K_PROFILE").is_none() {
            return;
        }
        if let Err(e) = std::fs::File::create(crate::platform::beside_log("rc2k_profile.txt")) {
            log(&format!("platform: rc2k_profile.txt is not emptied: {e}"));
        }
        SAMPLED
            .lock()
            .unwrap()
            .insert(0, crate::platform::ffi::win32::current_thread_id());
        *SAMPLES.lock().unwrap() = Some(Default::default());
        std::thread::spawn(move || {
            let mut threads: Vec<(u32, SampledThread)> = Vec::new();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1));
                for &id in SAMPLED.lock().unwrap().iter() {
                    if !threads.iter().any(|t| t.0 == id)
                        && let Some(thread) = SampledThread::open(id)
                    {
                        threads.push((id, thread));
                    }
                }
                for (_, thread) in &threads {
                    // The thread is resumed before `SAMPLES` is locked (it may have been
                    // stopped holding that lock).
                    if let Some(ip) = thread.instruction_pointer()
                        && let Ok(mut samples) = SAMPLES.lock()
                        && let Some(samples) = samples.as_mut()
                    {
                        *samples.entry(ip).or_insert(0) += 1;
                    }
                }
            }
        });
    });
}

/// The samples since the last report, as a line of the hottest places: the program's
/// (`rc2k+offset`, for `llvm-symbolizer`) or a DLL's.
#[cfg(windows)]
fn hot_places() -> String {
    let Some(samples) = SAMPLES
        .lock()
        .ok()
        .and_then(|mut s| s.as_mut().map(std::mem::take))
    else {
        return String::new();
    };
    let total: u32 = samples.values().sum();
    if total == 0 {
        return String::new();
    }
    // Host addresses throughout (sampled instruction pointers, modules).
    let program = crate::platform::ffi::win32::program_base();
    // by function-sized buckets would need symbols: by address, the top 24
    let mut places: Vec<(usize, u32)> = samples.into_iter().collect();
    places.sort_by_key(|p| std::cmp::Reverse(p.1));
    let name = |at: usize| -> String {
        let Some((path, base)) = crate::platform::ffi::win32::module_of_address_ansi(at) else {
            return format!("{at:#x}");
        };
        if base == program {
            return format!("rc2k+{:#x}", at - base);
        }
        let file = path.rsplit(['\\', '/']).next().unwrap_or("?");
        format!("{file}+{:#x}", at - base)
    };
    // Every place, for adding up by function offline (`llvm-symbolizer` on rc2k.exe).
    let all: String = places
        .iter()
        .map(|&(at, n)| format!("{} {n}\n", name(at)))
        .collect();
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(crate::platform::beside_log("rc2k_profile.txt"))
        .and_then(|mut f| {
            use std::io::Write;
            write!(f, "# {total} samples\n{all}")
        });
    let mut line = format!("platform: {total} samples, the hottest:");
    for &(at, n) in places.iter().take(24) {
        line += &format!(" {} {:.1}%,", name(at), n as f64 * 100.0 / total as f64);
    }
    line
}

/// Every five seconds a line on where the time went: the frames shown, the fans drawn and
/// the time in the rasteriser, the calls of functions that are not ported (each a fault).
pub fn report_timing(system: &System) {
    use std::sync::Mutex;
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Instant;
    /// When, and the counts then.
    type Mark = (Instant, u64, u64, u64, u64, u64);
    static LAST: Mutex<Option<Mark>> = Mutex::new(None);
    let now = Instant::now();
    let (frames, fans, micros, flushes, pixels) = (
        SHOWN.load(Ordering::SeqCst) as u64,
        crate::render::FANS.load(Relaxed),
        crate::render::RASTER_MICROS.load(Relaxed),
        crate::render::FLUSHES.load(Relaxed),
        crate::render::PIXELS.load(Relaxed),
    );
    // The screen's places, for the pixels a frame in screens.
    let screen = (system.session.screen.get().width.get().max(1)
        * system.session.screen.get().height.get().max(1)) as f64;
    let Ok(mut last) = LAST.lock() else {
        return;
    };
    match *last {
        Some((at, f0, n0, m0, l0, p0)) if now.duration_since(at).as_secs() >= 5 => {
            let seconds = now.duration_since(at).as_secs_f64();
            log(&format!(
                "platform: the last {seconds:.1} s: {} frames ({:.1} a second), {} fans in {} flushes, {:.1} screens of pixels a frame ({:.0} ms in the rasteriser, {:.0}%); the longest frame {:.1} ms, {} over 50 ms; the longest gap between sound turns {:.1} ms",
                frames - f0,
                (frames - f0) as f64 / seconds,
                fans - n0,
                flushes - l0,
                (pixels - p0) as f64 / (frames - f0).max(1) as f64 / screen,
                (micros - m0) as f64 / 1000.0,
                (micros - m0) as f64 / 10000.0 / seconds,
                LONGEST_FRAME.swap(0, Relaxed) as f64 / 1000.0,
                SLOW_FRAMES.swap(0, Relaxed),
                LONGEST_TURN_GAP.swap(0, Relaxed) as f64 / 1000.0,
            ));
            *last = Some((now, frames, fans, micros, flushes, pixels));
            log(&format!(
                "platform: so far {} frames read back from the GPU ({} ms), {} textures sent up ({} KB), the canvas {} KB and the retained background {} KB; {} GPU draws ({} with no alpha test, {} program switches); the GL thread busy {} ms (batching {} ms, showing {} ms), the game waiting for it {} ms",
                crate::render::READ_BACKS.load(Relaxed),
                crate::render::READ_BACK_MICROS.load(Relaxed) / 1000,
                crate::render::UPLOADS.load(Relaxed),
                crate::render::UPLOAD_BYTES.load(Relaxed) / 1024,
                crate::render::CANVAS_UPLOAD_BYTES.load(Relaxed) / 1024,
                crate::render::BACKGROUND_UPLOAD_BYTES.load(Relaxed) / 1024,
                crate::render::GPU_DRAWS.load(Relaxed),
                crate::render::GPU_FREE_DRAWS.load(Relaxed),
                crate::render::GPU_PROGRAM_SWITCHES.load(Relaxed),
                crate::render::GL_BUSY_MICROS.load(Relaxed) / 1000,
                crate::render::GL_BATCHING_MICROS.load(Relaxed) / 1000,
                crate::render::GL_SHOW_MICROS.load(Relaxed) / 1000,
                crate::render::GL_WAIT_MICROS.load(Relaxed) / 1000,
            ));
            #[cfg(handheld)]
            {
                use crate::render::gpu_time as g;
                let frames = g::FRAMES.swap(0, Relaxed);
                let (issue, wait) = (
                    g::ISSUE_MICROS.swap(0, Relaxed),
                    g::WAIT_MICROS.swap(0, Relaxed),
                );
                let longest = g::LONGEST_WAIT_MICROS.swap(0, Relaxed);
                if frames > 0 {
                    log(&format!(
                        "platform: GPU time (RC2K_GPU_TIME): {frames} frames waited for in the last 5 s: {:.2} ms to issue a frame's draws, then {:.2} ms (longest {:.2} ms) for the GPU to finish them",
                        issue as f64 / 1000.0 / frames as f64,
                        wait as f64 / 1000.0 / frames as f64,
                        longest as f64 / 1000.0,
                    ));
                }
            }
            {
                use crate::gfx::draw2d::quads;
                log(&format!(
                    "platform: so far {} solid quads, {} of {} GPU flushes sent canvas rows; {} glyph quads drawn, {} drawn on the canvas after being noted, {} left to the canvas (not in the atlas {}, no palette {}, atlas full {}), {} KB sent to the atlas, the atlas cleared {} times",
                    quads::SOLIDS.load(Relaxed),
                    crate::render::CANVAS_UPLOADS.load(Relaxed),
                    crate::render::GPU_FLUSHES.load(Relaxed),
                    quads::QUADS.load(Relaxed),
                    quads::DEMOTED.load(Relaxed),
                    quads::REFUSED.load(Relaxed),
                    quads::REFUSED_BY[0].load(Relaxed),
                    quads::REFUSED_BY[1].load(Relaxed),
                    quads::REFUSED_BY[2].load(Relaxed),
                    quads::ATLAS_BYTES.load(Relaxed) / 1024,
                    quads::ATLAS_CLEARS.load(Relaxed),
                ));
            }
            #[cfg(phone)]
            phone_line(system, seconds, frames - f0);
            // The handheld's: where the time goes, and the samples so far.
            #[cfg(handheld)]
            {
                use crate::debug::timing;
                static WAS: std::sync::Mutex<([u64; timing::STAGES], [u64; timing::STAGES], u32)> =
                    std::sync::Mutex::new(([0; timing::STAGES], [0; timing::STAGES], 0));
                let (nanos, runs) = timing::counts();
                let ticks = timing::ticks();
                if let Ok(mut was) = WAS.lock() {
                    let ms = |n: usize| (nanos[n] - was.0[n]) / 1_000_000;
                    let times = |n: usize| runs[n] - was.1[n];
                    log(&format!(
                        "handheld: input {} ms / {}, physics {} ms / {} (ticks {}), scene {} ms / {} (course faces {} ms, depth entries {} ms, mirror {} ms, effects {} ms), sound tick {} ms / {}, audio {} ms / {}",
                        ms(0),
                        times(0),
                        ms(1),
                        times(1),
                        ticks - was.2,
                        ms(2),
                        times(2),
                        ms(5),
                        ms(6),
                        ms(7),
                        ms(8),
                        ms(3),
                        times(3),
                        ms(4),
                        times(4)
                    ));
                    *was = (nanos, runs, ticks);
                }
                crate::platform::handheld::write_profile();
            }
            {
                use crate::render::{PARALLEL_MICROS, TILE_MICROS};
                static LAST_P: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                static LAST_T: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                static LAST_S: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let (p, t) = (PARALLEL_MICROS.load(Relaxed), TILE_MICROS.load(Relaxed));
                let (dp, dt) = (p - LAST_P.swap(p, Relaxed), t - LAST_T.swap(t, Relaxed));
                static LAST_N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let sh = SHOW_MICROS.load(Relaxed);
                let ds = sh - LAST_S.swap(sh, Relaxed);
                let no = crate::platform::direct3d::NOTE_MICROS.load(Relaxed);
                let dn = no - LAST_N.swap(no, Relaxed);
                log(&format!(
                    "platform: the rasteriser's parallel part {:.0} ms, the threads in its tiles {:.0} ms ({:.1} threads busy on average); showing the frames {:.0} ms; noting the fans {:.0} ms",
                    dp as f64 / 1000.0,
                    dt as f64 / 1000.0,
                    dt as f64 / dp.max(1) as f64,
                    ds as f64 / 1000.0,
                    dn as f64 / 1000.0
                ));
            }
            let hot = hot_places();
            if !hot.is_empty() {
                log(&hot);
            }
        }
        Some(_) => {}
        None => *last = Some((now, frames, fans, micros, flushes, pixels)),
    }
}

/// The phones' line on where a race's frame goes (the log is what a phone's owner can send):
/// frames a second, the game's stages (ticks of the logic, the scene's drawing, the sound), the
/// time showing a frame and, inside it, the finish of the GPU's frame and the present, the 3D's
/// flushes, and readbacks of the frame; with the race size, the 3D's share, the renderer and the
/// display's refresh. Times are average and (worst) in milliseconds a frame, except the flushes,
/// a flush.
#[cfg(phone)]
fn phone_line(system: &System, seconds: f64, frames: u64) {
    use crate::debug::timing;
    use std::sync::atomic::Ordering::Relaxed;
    /// The counts at the last line: the stages' nanoseconds and runs, the ticks, the waits'
    /// totals, the flushes and the read-backs.
    type Counts = (
        [u64; timing::STAGES],
        [u64; timing::STAGES],
        u32,
        [u64; 5],
        u64,
        u64,
    );
    static WAS: std::sync::Mutex<Counts> =
        std::sync::Mutex::new(([0; timing::STAGES], [0; timing::STAGES], 0, [0; 5], 0, 0));
    let (nanos, runs) = timing::counts();
    let ticks = timing::ticks();
    let totals = [
        SHOW_MICROS.load(Relaxed),
        PRESENT_MICROS.load(Relaxed),
        FINISH_MICROS.load(Relaxed),
        READ_BACK_MICROS.load(Relaxed),
        FLUSH_MICROS.load(Relaxed),
    ];
    let flushes = FLUSH_RUNS.load(Relaxed);
    let read_backs = crate::render::READ_BACKS.load(Relaxed);
    let Ok(mut was) = WAS.lock() else {
        return;
    };
    let frames = frames.max(1) as f64;
    let ms = |n: usize| (nanos[n] - was.0[n]) as f64 / 1e6;
    let per_frame = |total: u64, before: u64| (total - before) as f64 / 1000.0 / frames;
    let screen = system.session.screen.get();
    let flush_count = (flushes - was.4).max(1) as f64;
    log(&format!(
        "phone: {:.1} frames a second at {} x {}, 3D {}%, renderer {}, display {} Hz; a frame: logic {:.1} ms ({} ticks, {:.1} a frame), scene {:.1} (course {:.1}, depth entries {:.1}, mirror {:.1}, effects {:.1}), sound {:.1}; show {:.1} (worst {:.1}) of which finish {:.1} (worst {:.1}), present {:.1} (worst {:.1}); {} flushes, {:.2} ms each (worst {:.1}); {} readbacks of the frame, {:.1} ms each (worst {:.1}); {} drawing threads",
        frames / seconds,
        screen.width.get(),
        screen.height.get(),
        crate::render::render_scale(),
        if crate::render::gpu_running() {
            "GPU"
        } else {
            "software"
        },
        crate::platform::pacing::display_refresh().map_or("?".to_string(), |r| format!("{r:.0}")),
        (ms(1) + ms(0)) / frames,
        ticks - was.2,
        f64::from(ticks - was.2) / frames,
        ms(2) / frames,
        ms(5) / frames,
        ms(6) / frames,
        ms(7) / frames,
        ms(8) / frames,
        ms(3) / frames,
        per_frame(totals[0], was.3[0]),
        SHOW_MAX.swap(0, Relaxed) as f64 / 1000.0,
        per_frame(totals[2], was.3[2]),
        FINISH_MAX.swap(0, Relaxed) as f64 / 1000.0,
        per_frame(totals[1], was.3[1]),
        PRESENT_MAX.swap(0, Relaxed) as f64 / 1000.0,
        flushes - was.4,
        (totals[4] - was.3[4]) as f64 / 1000.0 / flush_count,
        FLUSH_MAX.swap(0, Relaxed) as f64 / 1000.0,
        read_backs - was.5,
        (totals[3] - was.3[3]) as f64 / 1000.0 / (read_backs - was.5).max(1) as f64,
        READ_BACK_MAX.swap(0, Relaxed) as f64 / 1000.0,
        crate::scene::workers::threads(),
    ));
    *was = (nanos, runs, ticks, totals, flushes, read_backs);
}
