//! Where the time goes: the frames' counts and timings in the log, and on Windows the main
//! thread and the rasteriser's sampled (with `RC2K_PROFILE` set).

use crate::framework::log;
use crate::platform::SHOWN;
use crate::system::owner::system;
use std::sync::atomic::Ordering;

// ---- where the time goes: the main thread and the rasteriser's sampled every millisecond ----

/// Where the sampled threads were, by address, since the last report.
#[cfg(windows)]
static SAMPLES: std::sync::Mutex<Option<std::collections::HashMap<usize, u32>>> =
    std::sync::Mutex::new(None);

/// The time putting frames on the screen (`show_frame`), in microseconds.
pub static SHOW_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

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
                .push(crate::ffi::win32::current_thread_id());
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
    use crate::ffi::win32::SampledThread;
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
            .insert(0, crate::ffi::win32::current_thread_id());
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
    let program = crate::ffi::win32::program_base();
    // by function-sized buckets would need symbols: by address, the top 24
    let mut places: Vec<(usize, u32)> = samples.into_iter().collect();
    places.sort_by_key(|p| std::cmp::Reverse(p.1));
    let name = |at: usize| -> String {
        let Some((path, base)) = crate::ffi::win32::module_of_address_ansi(at) else {
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
pub fn report_timing() {
    let system = system();
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
    let screen = (system.session.screen.width.get().max(1)
        * system.session.screen.height.get().max(1)) as f64;
    let Ok(mut last) = LAST.lock() else {
        return;
    };
    match *last {
        Some((at, f0, n0, m0, l0, p0)) if now.duration_since(at).as_secs() >= 5 => {
            let seconds = now.duration_since(at).as_secs_f64();
            log(&format!(
                "platform: the last {seconds:.1} s: {} frames ({:.1} a second), {} fans in {} flushes, {:.1} screens of pixels a frame ({:.0} ms in the rasteriser, {:.0}%)",
                frames - f0,
                (frames - f0) as f64 / seconds,
                fans - n0,
                flushes - l0,
                (pixels - p0) as f64 / (frames - f0).max(1) as f64 / screen,
                (micros - m0) as f64 / 1000.0,
                (micros - m0) as f64 / 10000.0 / seconds
            ));
            *last = Some((now, frames, fans, micros, flushes, pixels));
            log(&format!(
                "platform: so far {} frames read back from the GPU, {} textures sent up ({} KB); {} GPU draws; the GL thread busy {} ms, the game waiting for it {} ms",
                crate::render::READ_BACKS.load(Relaxed),
                crate::render::UPLOADS.load(Relaxed),
                crate::render::UPLOAD_BYTES.load(Relaxed) / 1024,
                crate::render::GPU_DRAWS.load(Relaxed),
                crate::render::GL_BUSY_MICROS.load(Relaxed) / 1000,
                crate::render::GL_WAIT_MICROS.load(Relaxed) / 1000,
            ));
            // The handheld's: where the time goes, and the samples so far.
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
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
                        "handheld: input {} ms / {}, physics {} ms / {} (ticks {}), scene {} ms / {} (course faces {} ms, depth entries {} ms, mirror {} ms), sound tick {} ms / {}, audio {} ms / {}",
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
                        ms(3),
                        times(3),
                        ms(4),
                        times(4)
                    ));
                    *was = (nanos, runs, ticks);
                }
                crate::handheld::write_profile();
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
                let no = crate::platform_3d::NOTE_MICROS.load(Relaxed);
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
