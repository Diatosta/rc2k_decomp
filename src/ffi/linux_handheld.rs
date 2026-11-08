//! Linux system calls for the handheld (`libc`), for `handheld`: the framebuffer device
//! (`/dev/fb0`, its mode, its mapped pages and the pan between them) and the sampling profiler
//! (a `SIGPROF` timer, the samples' registers, and reading a few words of this process).
//!
//! `Framebuffer` owns the open device and its mapping, which lives as long as the program
//! (never unmapped); its pages are reached only through `pages`, under `&mut self`.

#![allow(unsafe_code)]

use std::fs::File;
use std::os::fd::AsRawFd;
use std::sync::OnceLock;

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;

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

/// `fb_fix_screeninfo`, of which the memory's length and a row's are read.
#[repr(C)]
struct FixedInfo {
    id: [u8; 16],
    memory_start: libc::c_ulong,
    memory_length: u32,
    kind: u32,
    kind_aux: u32,
    visual: u32,
    pan_steps: [u16; 3],
    row_bytes: u32,
    io_start: libc::c_ulong,
    io_length: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

/// The framebuffer device in a mode, its two pages mapped.
pub struct Framebuffer {
    /// Open while the mapping lives.
    file: File,
    pixels: *mut u8,
    /// The pages' whole length in bytes.
    len: usize,
    info: VarInfo,
}

// SAFETY: the owner keeps it under one lock; the mapping is the program's for its life.
unsafe impl Send for Framebuffer {}

impl Framebuffer {
    /// `/dev/fb0` set to `width` x `height` at 32 bits (XRGB8888, B, G, R, X in memory) on two
    /// pages of `stride` bytes a row, mapped and black; or why it cannot.
    pub fn open(width: usize, height: usize, stride: usize) -> Result<Self, String> {
        let file = File::options()
            .read(true)
            .write(true)
            .open("/dev/fb0")
            .map_err(|e| format!("/dev/fb0: {e}"))?;
        let fd = file.as_raw_fd();
        let len = stride * height * 2;
        // SAFETY: the ioctls fill and read a record of the kernel's size; the mapping's length
        // is checked by the kernel against the framebuffer's memory.
        unsafe {
            let mut info: VarInfo = std::mem::zeroed();
            if libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut info) != 0 {
                return Err("the framebuffer's mode cannot be read".into());
            }
            info.xres = width as u32;
            info.yres = height as u32;
            info.xres_virtual = width as u32;
            info.yres_virtual = height as u32 * 2;
            info.xoffset = 0;
            info.yoffset = 0;
            info.bits_per_pixel = 32;
            // XRGB8888 in memory order B, G, R, X: the frames' own.
            info.colours = [16, 8, 0, 8, 8, 0, 0, 8, 0, 24, 8, 0];
            if libc::ioctl(fd, FBIOPUT_VSCREENINFO, &mut info) != 0 {
                return Err("the framebuffer takes no 720x480 at 32 bits".into());
            }
            // The mode the driver has set, which may not be the one asked for.
            let set = (info.xres, info.yres, info.bits_per_pixel);
            if set != (width as u32, height as u32, 32) || info.yres_virtual < height as u32 * 2 {
                return Err(format!(
                    "the framebuffer set {} x {} at {} bits, {} rows in all, not 720x480 at 32 \
                     bits on two pages",
                    set.0, set.1, set.2, info.yres_virtual
                ));
            }
            let mut fixed: FixedInfo = std::mem::zeroed();
            if libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fixed) != 0 {
                return Err("the framebuffer's layout cannot be read".into());
            }
            if fixed.row_bytes as usize != stride || (fixed.memory_length as usize) < len {
                return Err(format!(
                    "the framebuffer's rows are {} bytes and its memory {} bytes, not {stride} \
                     and at least {len}",
                    fixed.row_bytes, fixed.memory_length
                ));
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
                len,
                info,
            })
        }
    }

    /// The two pages.
    pub fn pages(&mut self) -> &mut [u8] {
        // SAFETY: the mapping's whole length, the program's for its life; `&mut self` is the
        // only way to it.
        unsafe { std::slice::from_raw_parts_mut(self.pixels, self.len) }
    }

    /// The screen panned to the page starting at row `y_offset`.
    pub fn pan_to(&mut self, y_offset: u32) -> bool {
        self.info.yoffset = y_offset;
        // SAFETY: the record the kernel gave, with the page's offset.
        unsafe { libc::ioctl(self.file.as_raw_fd(), FBIOPAN_DISPLAY, &mut self.info) == 0 }
    }
}

/// What the profiler's signal gives: where the processor was, the stack pointer, the frame
/// pointer, the link register.
pub struct Sample {
    pub pc: usize,
    pub sp: usize,
    pub frame: usize,
    pub link: usize,
}

/// The handler `start_profile_timer` was given; set once, before the timer runs.
static ON_SAMPLE: OnceLock<fn(Sample)> = OnceLock::new();

/// The timer's signal: only atomics and a system call, as a handler must.
unsafe extern "C" fn on_profile(
    _signal: libc::c_int,
    _info: *mut libc::siginfo_t,
    context: *mut libc::c_void,
) {
    // SAFETY: the record the system hands the handler.
    let sample = unsafe {
        let m = &(*context.cast::<libc::ucontext_t>()).uc_mcontext;
        Sample {
            pc: m.pc as usize,
            sp: m.sp as usize,
            frame: m.regs[29] as usize,
            link: m.regs[30] as usize,
        }
    };
    if let Some(handler) = ON_SAMPLE.get() {
        handler(sample);
    }
}

/// `handler` called from a `SIGPROF` signal handler (so it may only use atomics and system
/// calls) 500 times a second of processor time.
pub fn start_profile_timer(handler: fn(Sample)) {
    let _ = ON_SAMPLE.set(handler);
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
}

/// This thread's system id (a system call alone, as a signal handler must).
pub fn current_thread_id() -> u32 {
    // SAFETY: `gettid` takes nothing and cannot fail.
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

/// The two words at `at` in this process's memory, or None where it has none; a system call
/// alone, as a signal handler must.
pub fn read_own_words(at: usize) -> Option<[usize; 2]> {
    let mut words = [0usize; 2];
    let local = libc::iovec {
        iov_base: words.as_mut_ptr().cast(),
        iov_len: size_of_val(&words),
    };
    let remote = libc::iovec {
        iov_base: std::ptr::with_exposed_provenance_mut(at),
        iov_len: size_of_val(&words),
    };
    // SAFETY: `local` is `words`, of its length; the kernel only reads at `remote`, and fails
    // the call for memory that is not there.
    let read = unsafe {
        libc::process_vm_readv(libc::getpid(), &raw const local, 1, &raw const remote, 1, 0)
    };
    (read == size_of_val(&words) as isize).then_some(words)
}
