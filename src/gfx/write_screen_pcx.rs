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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::files::stream::Stream;
use crate::gfx::decoder::Slot as DecSlot;
use crate::gfx::pixel_format::PixelFormat;
use crate::physics::flow::g;

const HEADER: DecSlot = DecSlot::at(0x51B5F4); // g_pcxHeader
const STREAM: DecSlot = DecSlot::at(0x5195E4); // g_pictureStream
pub const PICTURE_NAME: crate::gfx::picture_store::Slot =
    crate::gfx::picture_store::Slot::at(0x4F49C8); // dword_4F49C8: the file, or 0
const WIDTH: usize = 0x4F49A0; // g_surface.width
const HEIGHT: usize = 0x4F49A4; // g_surface.height
const BITS: usize = 0x4F49A8; // g_surface.bits_per_pixel
const FORMAT: usize = 0x4F49B8; // g_surface.format
const PITCH: usize = 0x4F49BC; // g_surface.pitch
const PIXELS: usize = 0x4F49C0; // g_surface.pixels

/// `files::write_stream_byte` with the byte.
unsafe fn write_stream_byte(stream: *mut Stream, byte: u8) -> Result<bool, Refused> {
    unsafe {
        Ok(crate::files::write_stream_byte::write_stream_byte(
            stream,
            byte as u32,
        )?)
    }
}

/// `files::write_stream_block`.
unsafe fn write_stream_block(stream: *mut Stream, bytes: &[u8]) -> Result<bool, Refused> {
    unsafe {
        Ok(crate::files::write_stream_block::write_stream_all(
            stream, bytes,
        )?)
    }
}

/// The header's first 0x80 bytes for a surface `width` x `height` (converted:
/// `rc2k_core::gfx::pcx::screen_header`).
pub use rc2k_core::gfx::pcx::screen_header as header;

/// The Rust version: whether it worked (carry clear).
pub unsafe fn write_screen_pcx() -> Result<bool, Refused> {
    unsafe {
        let header_bytes = header(g(WIDTH), g(HEIGHT));
        std::ptr::copy_nonoverlapping(header_bytes.as_ptr(), HEADER.address() as *mut u8, 0x80);
        let mut path: u32 = crate::gfx::block::game_read(PICTURE_NAME);
        if path == 0 {
            path = crate::gfx::shots::NAME.address() as u32;
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
        let stream = STREAM.address() as *mut Stream;
        if !crate::files::open_stream::open_stream(stream, path as *const u8, 1)? {
            return Ok(false);
        }
        let mut ok = write_stream_block(stream, &header_bytes)? && rows(stream)?;
        let closed = crate::files::close_stream::close_stream(stream)?;
        ok = ok && closed;
        Ok(ok)
    }
}

/// The rows; false when a write failed. Converted: `rc2k_core::gfx::pcx::screen_planes` and
/// `encode_line`; the planes are left in the game's buffer, as the original leaves them.
unsafe fn rows(stream: *mut Stream) -> Result<bool, Refused> {
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
            std::ptr::copy_nonoverlapping(
                planes.as_ptr(),
                crate::gfx::pcx_planes::PLANES.address() as *mut u8,
                planes.len(),
            );
            for byte in rc2k_core::gfx::pcx::encode_line(&planes, &mut eax) {
                if !write_stream_byte(stream, byte)? {
                    return Ok(false);
                }
            }
            row = row.wrapping_add(g(PITCH));
            y += 1;
            if y == g(HEIGHT) {
                break;
            }
        }
        Ok(true)
    }
}
