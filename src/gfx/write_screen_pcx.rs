//! `gfx::write_screen_pcx` (0x411E0C): the locked surface (`g_surface`) written as a
//! 24-bit PCX file: the header (`g_pcxHeader`, 0x80 bytes zeroed: 0x0A, version 5, run
//! coded, 8 bits; 300 dpi both ways; right and bottom the size less one; the 16-colour
//! palette greys 0 .. 15; three planes of `width` bytes), then each row taken apart
//! (`gfx::pixel_to_components`, 16- or 32-bit pixels) into red, green and blue planes in
//! `byte_94F6F8` and run coded as one line of `3 * width` bytes (runs to 63, a lone byte
//! below 0xC0 as itself), through the stream `g_pictureStream` (`files::open_stream` mode
//! 1, `files::write_stream_block`, `files::write_stream_byte`, `files::close_stream`).
//! The file is `dword_4F49C8` when set, else the game's screenshot name `000.pcx`
//! (0x4F52BC), its three digits one more first (999 goes round to 000).
//!
//! In:  nothing (the surface locked)
//! Out: carry set when it failed (the open, a write, the close); every register kept
//!      (`pusha` / `popa`)
//!
//! A 16-bit pixel is read with `mov ax`: the conversion gets the upper half of eax as it
//! was (reproduced, eax followed through). Pixels of other sizes are not read: the last
//! value of eax is used (reproduced). The runs go over the planes' ends (the format wants
//! each plane's line coded alone; the game's own reader does not mind).
//!
//! Callers: `gfx::take_screenshot`, `race::viewer_snapshot`, `race::replay_screenshot`.
//! Refuses what `open_stream` refuses. Shadow mode runs the original alone (it writes a file).
//! Self-test: a surface of the test's (16 and 32 bits, 64 x 48), the file a name of the
//! test's in `var` (removed after), against the original: the files' bytes, the header,
//! the stream and the planes compared.

use crate::errors::Refused;
use crate::files::stream::{BUFFER_LEN, Stream};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::pixel_format::PixelFormat;
use crate::physics::flow::{a, g, p};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::write_screen_pcx", rva(0x411E0C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const HEADER: usize = 0x51B5F4; // g_pcxHeader
const STREAM: usize = 0x5195E4; // g_pictureStream
const PLANES: usize = 0x94F6F8; // byte_94F6F8
pub const PICTURE_NAME: usize = 0x4F49C8; // dword_4F49C8: the file, or 0
const SCREENSHOT_NAME: usize = 0x4F52BC; // "000.pcx"
const WIDTH: usize = 0x4F49A0; // g_surface.width
const HEIGHT: usize = 0x4F49A4; // g_surface.height
const BITS: usize = 0x4F49A8; // g_surface.bits_per_pixel
const FORMAT: usize = 0x4F49B8; // g_surface.format
const PITCH: usize = 0x4F49BC; // g_surface.pitch
const PIXELS: usize = 0x4F49C0; // g_surface.pixels

/// `files::write_stream_byte` with the byte.
unsafe fn write_stream_byte(stream: *mut Stream, byte: u8) -> bool {
    unsafe { crate::files::write_stream_byte::write_stream_byte(stream, byte as u32) }
}

/// `files::write_stream_block`.
unsafe fn write_stream_block(stream: *mut Stream, bytes: &[u8]) -> bool {
    unsafe { crate::files::write_stream_block::write_stream_all(stream, bytes) }
}

/// The header's first 0x80 bytes for a surface `width` x `height` (converted:
/// `rc2k_core::gfx::pcx::screen_header`).
pub use rc2k_core::gfx::pcx::screen_header as header;

/// The Rust version: whether it worked (carry clear).
pub unsafe fn write_screen_pcx() -> Result<bool, Refused> {
    unsafe {
        let header_bytes = header(g(WIDTH), g(HEIGHT));
        std::ptr::copy_nonoverlapping(header_bytes.as_ptr(), a(HEADER) as *mut u8, 0x80);
        let mut path = g(PICTURE_NAME);
        if path == 0 {
            path = a(SCREENSHOT_NAME);
            let digits = path as *mut u8;
            for n in (0..3).rev() {
                let d = digits.add(n);
                d.write(d.read().wrapping_add(1));
                if d.read() <= b'9' {
                    break;
                }
                d.write(b'0');
            }
        }
        let stream = a(STREAM) as *mut Stream;
        if !crate::files::open_stream::open_stream(stream, path as *const u8, 1)? {
            return Ok(false);
        }
        let mut ok = write_stream_block(stream, &header_bytes) && rows(stream);
        let closed = crate::files::close_stream::close_stream(stream);
        ok = ok && closed;
        Ok(ok)
    }
}

/// The rows; false when a write failed. Converted: `rc2k_core::gfx::pcx::screen_planes` and
/// `encode_line`; the planes are left in the game's buffer, as the original leaves them.
unsafe fn rows(stream: *mut Stream) -> bool {
    unsafe {
        let mut row = g(PIXELS);
        // eax as the original has it: the block's byte count after the header (0x80).
        let mut eax: u32 = 0x80;
        let width = g(WIDTH);
        let mut y = 0u32;
        loop {
            let format = (g(FORMAT) as *const PixelFormat).read_unaligned();
            let planes =
                rc2k_core::gfx::pcx::screen_planes(width, &mut eax, &mut |x, eax| match g(BITS) {
                    0x10 => crate::gfx::pixel_to_components::pixel_to_components(
                        &format,
                        eax & 0xFFFF_0000 | ((row + 2 * x) as *const u16).read_unaligned() as u32,
                    ),
                    0x20 => crate::gfx::pixel_to_components::pixel_to_components(
                        &format,
                        ((row + 4 * x) as *const u32).read_unaligned(),
                    ),
                    _ => eax,
                });
            std::ptr::copy_nonoverlapping(planes.as_ptr(), a(PLANES) as *mut u8, planes.len());
            for byte in rc2k_core::gfx::pcx::encode_line(&planes, &mut eax) {
                if !write_stream_byte(stream, byte) {
                    return false;
                }
            }
            row = row.wrapping_add(g(PITCH));
            y += 1;
            if y == g(HEIGHT) {
                break;
            }
        }
        true
    }
}

/// For Rust callers in the game.
pub unsafe fn write_screen_pcx_in_game(r: &mut Regs) -> Result<(), Refused> {
    let ok = unsafe { write_screen_pcx() }?;
    r.set_carry(!ok);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::framework::shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut left = *r;
    match unsafe { write_screen_pcx_in_game(&mut left) } {
        Ok(()) => r.eflags = left.eflags,
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const FILE_A: &str = "var\\rc2k_hook_pcx_a.pcx";
const FILE_B: &str = "var\\rc2k_hook_pcx_b.pcx";

fn self_test() -> (usize, usize) {
    use crate::race::mud::{TestSurface, has_format};
    use crate::service::harness::{capture, diff_summary};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    let regions = [
        (a(HEADER), 0x80),
        (a(STREAM), 0x10 + BUFFER_LEN),
        (a(PLANES), 3 * 64),
        (a(PICTURE_NAME), 4),
        (a(SCREENSHOT_NAME), 8),
    ];
    let saved = crate::race::viewer::Saved::new(&regions);
    let last = crate::files::stream::LastPath::save();
    for bits in [16u32, 32] {
        let surface = TestSurface::new(bits);
        let (a_path, b_path) = (b"var\\rc2k_hook_pcx_a.pcx\0", b"var\\rc2k_hook_pcx_b.pcx\0");
        p(PICTURE_NAME, a_path.as_ptr() as u32);
        let mut ro = Regs::SENTINEL;
        unsafe { call_original(&mut ro) };
        let theirs = (
            ro.carry(),
            std::fs::read(FILE_A).ok(),
            capture(&regions[..3]),
        );
        p(PICTURE_NAME, b_path.as_ptr() as u32);
        let mine = unsafe { write_screen_pcx() };
        let state = (
            !mine.as_ref().is_ok_and(|&ok| ok),
            std::fs::read(FILE_B).ok(),
            capture(&regions[..3]),
        );
        let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
        t.check(regs_ok && theirs == state, || {
            format!(
                "{bits} bits: regs_ok={regs_ok} carry {} {}; files the same {} ({:?} {:?} bytes); \
                 memory {}",
                theirs.0,
                state.0,
                theirs.1 == state.1,
                theirs.1.as_ref().map(|f| f.len()),
                state.1.as_ref().map(|f| f.len()),
                diff_summary(&theirs.2, &state.2)
            )
        });
        drop(surface);
    }
    let _ = std::fs::remove_file(FILE_A);
    let _ = std::fs::remove_file(FILE_B);
    last.restore();
    drop(saved);
    t.finish()
}
