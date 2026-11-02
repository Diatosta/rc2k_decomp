//! The Anbernic RG34XXSP (2026-10-30, a first draft): its stock OS has no display system SDL3
//! knows (no DRM, X11 or Wayland), so the frames go straight to `/dev/fb0`. SDL stays for
//! sound and the pad (`platform_input`, `input::menu_pad`), on its `dummy` video driver.

#![allow(unsafe_code)]

use std::fs::File;
use std::os::fd::AsRawFd;
use std::sync::{Mutex, MutexGuard, OnceLock};

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;
const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;

/// The panel.
const WIDTH: usize = 720;
const HEIGHT: usize = 480;
const STRIDE: usize = WIDTH * 4;

/// `fb_var_screeninfo`: 160 bytes, of which these are read and set.
#[repr(C)]
struct VarInfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    /// Offset and length of red, green, blue, alpha (and each one's `msb_right`).
    colours: [u32; 12],
    rest: [u32; 20],
}

struct Framebuffer {
    /// Open while the mapping lives.
    file: File,
    pixels: *mut u8,
    info: VarInfo,
    /// The page not on the screen.
    back: usize,
    /// The last frame's size.
    size: (usize, usize),
}

// SAFETY: only ever used under `FRAMEBUFFER`'s lock; the mapping is the program's for its life.
unsafe impl Send for Framebuffer {}

impl Framebuffer {
    /// The two pages.
    fn pages(&mut self) -> &mut [u8] {
        // SAFETY: the mapping's whole length, the program's for its life; `&mut self` under
        // `FRAMEBUFFER`'s lock is the only way to it.
        unsafe { std::slice::from_raw_parts_mut(self.pixels, STRIDE * HEIGHT * 2) }
    }
}

static FRAMEBUFFER: Mutex<Option<Framebuffer>> = Mutex::new(None);

/// 720x480 at 32 bits, two pages; black.
fn open_framebuffer() -> Result<Framebuffer, String> {
    let file = File::options()
        .read(true)
        .write(true)
        .open("/dev/fb0")
        .map_err(|e| format!("/dev/fb0: {e}"))?;
    let fd = file.as_raw_fd();
    let len = STRIDE * HEIGHT * 2;
    // SAFETY: the ioctls fill and read a record of the kernel's size; the mapping's length is
    // checked by the kernel against the framebuffer's memory.
    unsafe {
        let mut info: VarInfo = std::mem::zeroed();
        if libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut info) != 0 {
            return Err("the framebuffer's mode cannot be read".into());
        }
        info.xres = WIDTH as u32;
        info.yres = HEIGHT as u32;
        info.xres_virtual = WIDTH as u32;
        info.yres_virtual = HEIGHT as u32 * 2;
        info.xoffset = 0;
        info.yoffset = 0;
        info.bits_per_pixel = 32;
        // XRGB8888 in memory order B, G, R, X: the frames' own.
        info.colours = [16, 8, 0, 8, 8, 0, 0, 8, 0, 24, 8, 0];
        if libc::ioctl(fd, FBIOPUT_VSCREENINFO, &mut info) != 0 {
            return Err("the framebuffer takes no 720x480 at 32 bits".into());
        }
        let pixels = libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if pixels == libc::MAP_FAILED {
            return Err("the framebuffer cannot be mapped".into());
        }
        std::ptr::write_bytes(pixels.cast::<u8>(), 0, len);
        Ok(Framebuffer {
            file,
            pixels: pixels.cast(),
            info,
            back: 1,
            size: (0, 0),
        })
    }
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
        fb.pages().fill(0);
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
    let pages = fb.pages();
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
    fb.info.yoffset = (fb.back * HEIGHT) as u32;
    // SAFETY: the record the kernel gave, with the page's offset.
    if unsafe { libc::ioctl(fb.file.as_raw_fd(), FBIOPAN_DISPLAY, &mut fb.info) } != 0 {
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
        && crate::race::numbers_store::NUMBERS.in_race.bits() != 0
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
    pub fn frame_parts(&mut self) -> (*mut u8, usize, usize, *mut f32) {
        let b = &mut *self.0;
        let height = b.pixels.len() / b.width;
        (
            b.pixels.as_mut_ptr().cast(),
            b.width,
            height,
            b.depth.as_mut_ptr(),
        )
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

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering::Relaxed};

const SAMPLES: usize = 1 << 17;
/// (Where the processor was, which thread) of each sample.
static PLACES: [AtomicUsize; SAMPLES] = [const { AtomicUsize::new(0) }; SAMPLES];
static THREADS: [AtomicU32; SAMPLES] = [const { AtomicU32::new(0) }; SAMPLES];
static TAKEN: AtomicUsize = AtomicUsize::new(0);

/// The timer's signal: only atomics and a system call, as a handler must.
unsafe extern "C" fn on_profile(
    _signal: libc::c_int,
    _info: *mut libc::siginfo_t,
    context: *mut libc::c_void,
) {
    // SAFETY: the record the system hands the handler.
    let pc = unsafe { (*context.cast::<libc::ucontext_t>()).uc_mcontext.pc } as usize;
    let n = TAKEN.fetch_add(1, Relaxed);
    if n < SAMPLES {
        PLACES[n].store(pc, Relaxed);
        // SAFETY: `gettid` takes nothing and cannot fail.
        THREADS[n].store(unsafe { libc::syscall(libc::SYS_gettid) } as u32, Relaxed);
    }
}

/// The timer started if `RC2K_PROFILE` is set: 500 samples of processor time a second.
pub fn start_profile() {
    if std::env::var_os("RC2K_PROFILE").is_none() {
        return;
    }
    // SAFETY: a handler of the system's signature, for the program's life; the records are
    // plain data the calls only read.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = on_profile as *const () as usize;
        action.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
        libc::sigaction(libc::SIGPROF, &raw const action, std::ptr::null_mut());
        let time = libc::timeval {
            tv_sec: 0,
            tv_usec: 2000,
        };
        let timer = libc::itimerval {
            it_interval: time,
            it_value: time,
        };
        libc::setitimer(libc::ITIMER_PROF, &raw const timer, std::ptr::null_mut());
    }
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
        text += &format!(
            "{:x} {}\n",
            PLACES[i].load(Relaxed),
            THREADS[i].load(Relaxed)
        );
    }
    let _ = std::fs::write("profile.txt", text);
}
