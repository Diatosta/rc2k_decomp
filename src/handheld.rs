//! The Anbernic RG34XXSP (2026-10-30, a first draft): its stock OS has no display system SDL3
//! knows (no DRM, X11 or Wayland), so the frames go straight to `/dev/fb0`. SDL stays for
//! sound and the pad (`platform_input`, `input::menu_pad`), on its `dummy` video driver.

use crate::ffi::linux_handheld::{self, Framebuffer, Sample};
use std::sync::{Mutex, MutexGuard, OnceLock};

/// The panel.
const WIDTH: usize = 720;
const HEIGHT: usize = 480;
const STRIDE: usize = WIDTH * 4;

/// The framebuffer and what is shown on it.
struct Screen {
    framebuffer: Framebuffer,
    /// The page not on the screen.
    back: usize,
    /// The last frame's size.
    size: (usize, usize),
}

static FRAMEBUFFER: Mutex<Option<Screen>> = Mutex::new(None);

/// 720x480 at 32 bits, two pages; black.
fn open_framebuffer() -> Result<Screen, String> {
    Ok(Screen {
        framebuffer: Framebuffer::open(WIDTH, HEIGHT, STRIDE)?,
        back: 1,
        size: (0, 0),
    })
}

/// The framebuffer in its mode, the first time.
pub fn open_screen() -> Result<(), &'static str> {
    let mut held = FRAMEBUFFER.lock().map_err(|_| "the framebuffer's lock")?;
    if held.is_none() {
        match open_framebuffer() {
            Ok(fb) => {
                *held = Some(fb);
                start_profile();
            }
            Err(e) => {
                crate::framework::log(&format!("handheld: {e}"));
                return Err("no framebuffer");
            }
        }
    }
    Ok(())
}

/// Shows a frame of `width` x `height` XRGB8888 rows, centred; one too big for the panel (the
/// menus' 800 x 600) is made smaller to fit, keeping its shape, by the nearest pixels.
pub fn present(frame: &[u8], width: usize, height: usize) -> Result<(), &'static str> {
    open_screen()?;
    let mut held = FRAMEBUFFER.lock().map_err(|_| "the framebuffer's lock")?;
    let Some(fb) = held.as_mut() else {
        return Err("no framebuffer");
    };
    // A new size leaves old pixels where the frame no longer reaches: both pages cleared.
    if fb.size != (width, height) {
        fb.size = (width, height);
        fb.framebuffer.pages().fill(0);
    }
    let (shown_w, shown_h) = if width > WIDTH || height > HEIGHT {
        let by_height = (width * HEIGHT / height, HEIGHT);
        if by_height.0 <= WIDTH {
            by_height
        } else {
            (WIDTH, height * WIDTH / width)
        }
    } else {
        (width, height)
    };
    let (left, top) = ((WIDTH - shown_w) / 2, (HEIGHT - shown_h) / 2);
    let page = fb.back * HEIGHT * STRIDE;
    let pages = fb.framebuffer.pages();
    for row in 0..shown_h {
        let from = &frame[row * height / shown_h * width * 4..][..width * 4];
        let at = page + (top + row) * STRIDE + left * 4;
        let to = &mut pages[at..at + shown_w * 4];
        if shown_w == width {
            to.copy_from_slice(from);
        } else {
            for (x, pixel) in to.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let f = x * width / shown_w * 4;
                pixel.copy_from_slice(&from[f..f + 4]);
            }
        }
    }
    if !fb.framebuffer.pan_to((fb.back * HEIGHT) as u32) {
        return Err("the framebuffer does not pan");
    }
    fb.back ^= 1;
    Ok(())
}

/// The race's mode the handheld takes: the game's smallest size, 640 x 480 (a saved 1920 x 1280
/// is far past what the processor can draw, and the panel's size).
pub fn race_mode(mode: u32) -> u32 {
    mode & !0xFF | 1
}

/// The built-in pad as a gamepad, for SDL's GUID of it (hex). It is read through its joystick
/// node (`/dev/input/js0`): the kernel's button order (A 304 .. R3 316), the sticks' four axes,
/// and the cross as a hat (SDL makes one of the node's two hat axes). The buttons by their printed names, so A enters as labelled.
pub fn pad_mapping(guid: &str) -> String {
    format!(
        "{guid},ANBERNIC-keys,a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,back:b6,\
         start:b7,guide:b8,leftstick:b9,lefttrigger:b10,righttrigger:b11,rightstick:b12,\
         leftx:a0,lefty:a1,rightx:a2,righty:a3,dpup:h0.1,dpright:h0.2,dpdown:h0.4,dpleft:h0.8,\
         platform:Linux,"
    )
}

// ---- a race's 3D drawn smaller: the frame's dearest part, with the fewest pixels to give ----

/// How many times smaller, each way.
pub const SMALLER: usize = 2;

struct SmallBuffers {
    pixels: Vec<u32>,
    depth: Vec<f32>,
    width: usize,
}

static SMALL: Mutex<SmallBuffers> = Mutex::new(SmallBuffers {
    pixels: Vec::new(),
    depth: Vec::new(),
    width: 0,
});

/// Whether this flush goes through a small frame: in a race, unless `RC2K_FULL_RESOLUTION`.
pub fn draws_small() -> bool {
    static FULL: OnceLock<bool> = OnceLock::new();
    !*FULL.get_or_init(|| std::env::var_os("RC2K_FULL_RESOLUTION").is_some())
        && race().course.numbers.in_race.bits() != 0
}

/// A small frame while it is drawn: its buffers held until it is enlarged.
pub struct Small(MutexGuard<'static, SmallBuffers>);

impl Small {
    /// The small frame, holding the screen's picture made smaller (what was drawn before this
    /// flush, which the 3D may blend over).
    pub fn from_screen(screen: &[u8], pitch: usize, width: usize, height: usize) -> Self {
        let (w, h) = (width / SMALLER, height / SMALLER);
        let mut b = SMALL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if b.pixels.len() != w * h {
            b.pixels = vec![0; w * h];
            b.depth = vec![1.0; w * h];
        }
        b.width = w;
        for (y, to) in b.pixels.chunks_exact_mut(w).enumerate() {
            let from = screen[y * SMALLER * pitch..].as_chunks::<4>().0;
            for (x, pixel) in to.iter_mut().enumerate() {
                *pixel = u32::from_ne_bytes(from[x * SMALLER]);
            }
        }
        Self(b)
    }

    /// For the rasteriser's `Frame`, which holds them no longer than this `Small` lives:
    /// (pixels, width, height, depths).
    pub fn frame_parts(&mut self) -> (&mut [u32], usize, usize, &mut [f32]) {
        let b = &mut *self.0;
        let height = b.pixels.len() / b.width;
        (&mut b.pixels, b.width, height, &mut b.depth)
    }

    /// Back on the screen, each pixel made `SMALLER` wide and high.
    pub fn enlarge_to(self, screen: &mut [u8], pitch: usize, width: usize, height: usize) {
        let b = &*self.0;
        for (y, row) in screen.chunks_exact_mut(pitch).take(height).enumerate() {
            let from = &b.pixels[(y / SMALLER) * b.width..][..b.width];
            let to = row[..width * 4].as_chunks_mut::<{ 4 * SMALLER }>().0;
            for (wide, &p) in to.iter_mut().zip(from) {
                for pixel in wide.as_chunks_mut::<4>().0 {
                    *pixel = p.to_ne_bytes();
                }
            }
        }
    }
}

// ---- a sampling profiler, while the handheld's speed is being worked out (the OS has no perf) ----

use crate::race::owner::race;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering::Relaxed};

const SAMPLES: usize = 1 << 16;
/// (Where the processor was, which thread) of each sample.
static PLACES: [AtomicUsize; SAMPLES] = [const { AtomicUsize::new(0) }; SAMPLES];
static THREADS: [AtomicU32; SAMPLES] = [const { AtomicU32::new(0) }; SAMPLES];
/// How many callers each sample keeps.
const CALLERS: usize = 24;
/// Each sample's callers, nearest first (0: the chain ended).
static CALLED_FROM: [[AtomicUsize; CALLERS]; SAMPLES] =
    [const { [const { AtomicUsize::new(0) }; CALLERS] }; SAMPLES];
static TAKEN: AtomicUsize = AtomicUsize::new(0);

/// The timer's signal, as `linux_handheld` hands it: only atomics and a system call.
fn on_profile(sample: Sample) {
    let Sample {
        pc,
        sp,
        mut frame,
        link: lr,
    } = sample;
    let n = TAKEN.fetch_add(1, Relaxed);
    if n < SAMPLES {
        PLACES[n].store(pc, Relaxed);
        CALLED_FROM[n][0].store(lr, Relaxed);
        // The frame records, each above the last and near the stack pointer: a function that
        // keeps none leaves a value that fails these tests and the chain stops.
        let mut low = sp;
        for caller in &CALLED_FROM[n][1..] {
            if frame < low || frame > sp + (1 << 16) || !frame.is_multiple_of(16) {
                break;
            }
            // Read by the kernel, which answers a failure for memory past the stack's top
            // rather than a fault in the handler.
            let Some([next, from]) = linux_handheld::read_own_words(frame) else {
                break;
            };
            caller.store(from, Relaxed);
            low = frame + 16;
            frame = next;
        }
        THREADS[n].store(linux_handheld::current_thread_id(), Relaxed);
    }
}

/// The timer started if `RC2K_PROFILE` is set: 500 samples of processor time a second.
pub fn start_profile() {
    if std::env::var_os("RC2K_PROFILE").is_none() {
        return;
    }
    linux_handheld::start_profile_timer(on_profile);
    crate::framework::log(&format!(
        "handheld: profiling; the code's base {:#x}",
        start_profile as *const () as usize
    ));
}

/// The samples so far as `profile.txt`: the marker's place, then (place, thread) a line.
pub fn write_profile() {
    let n = TAKEN.load(Relaxed).min(SAMPLES);
    if n == 0 {
        return;
    }
    let mut text = format!("{:x}\n", start_profile as *const () as usize);
    for i in 0..n {
        text += &format!("{:x} {}", PLACES[i].load(Relaxed), THREADS[i].load(Relaxed));
        for caller in &CALLED_FROM[i] {
            text += &format!(" {:x}", caller.load(Relaxed));
        }
        text.push('\n');
    }
    let _ = std::fs::write("profile.txt", text);
}
