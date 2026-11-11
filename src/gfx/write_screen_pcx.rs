//! `gfx::write_screen_pcx` (0x411E0C): the locked surface (`g_surface`) written as a
//! 24-bit PCX file: the header (`g_pcxHeader`, 0x80 bytes zeroed: 0x0A, version 5, run
//! coded, 8 bits; 300 dpi both ways; right and bottom the size less one; the 16-colour
//! palette greys 0 .. 15; three planes of `width` bytes), then each row taken apart
//! (`gfx::pixel_to_components`, 16- or 32-bit pixels) into red, green and blue planes in
//! `planes` and run coded as one line of `3 * width` bytes (runs to 63, a lone byte
//! below 0xC0 as itself), through the stream `g_pictureStream` (`files::open_stream` mode
//! 1, `files::write_stream_block`, `files::write_stream_byte`, `files::close_stream`).
//! The file is `name` when set, else the game's screenshot name `000.pcx`
//! (0x4F52BC), its three digits one more first (999 goes round to 000). Here the callers
//! that set `name` (and put it back after) pass the name instead.
//!
//! In:  nothing (the surface locked)
//!
//! A 16-bit pixel is read into the low half of a register: the conversion gets the upper
//! half as it was (reproduced; the register is followed through as `eax`). Pixels of other
//! sizes are not read: the last value of `eax` is used (reproduced). The runs go over the
//! planes' ends (the format wants each plane's line coded alone; the game's own reader
//! does not mind).
//!
//! Callers: `gfx::take_screenshot`, `race::viewer_snapshot`, `race::replay_screenshot`. Refuses
//! what `open_stream` refuses.

use crate::errors::Refused;
use crate::files::stream::{Stream, with_stream};
use crate::files::write_stream_block::write_stream_all;
use crate::files::write_stream_byte::write_stream_byte;
use crate::gfx::owner::gfx;
use crate::gfx::shots::Shots;

/// The header's first 0x80 bytes for a surface `width` x `height` (converted:
/// `rc2k_core::gfx::pcx::screen_header`).
pub use rc2k_core::gfx::pcx::screen_header as header;

/// The screenshot name's three digits one more (999 goes round to 000).
fn next_shot_name(shots: &Shots) {
    for n in (0..3).rev() {
        let d = shots.name.get(n).wrapping_add(1);
        if d <= b'9' {
            shots.name.set(n, d);
            break;
        }
        shots.name.set(n, b'0');
    }
}

/// Whether it worked (carry clear). `name` the file's path (without its
/// NUL), or None for the next numbered screenshot.
pub fn write_screen_pcx(
    shots: &Shots,
    name: Option<&[u8]>,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    let header_bytes = header(
        gfx.drawing.draw_surface.width.get(),
        gfx.drawing.draw_surface.height.get(),
    );
    gfx.pictures.pcx.header.write(&header_bytes);
    let path = match name {
        Some(name) => name.to_vec(),
        None => {
            next_shot_name(shots);
            shots.name.text()
        }
    };
    with_stream(|stream| {
        if !crate::files::open_stream::open_stream(stream, &path, 1)? {
            return Ok(false);
        }
        let mut ok = write_stream_all(stream, &header_bytes)? && rows(stream)?;
        let closed = crate::files::close_stream::close_stream(stream)?;
        ok = ok && closed;
        Ok(ok)
    })
}

/// The rows; false when a write failed. Converted: `rc2k_core::gfx::pcx::screen_planes` and
/// `encode_line`; the planes are left in the game's buffer, as the original leaves them.
fn rows(stream: &mut Stream) -> Result<bool, Refused> {
    let gfx = gfx();
    let rows = crate::gfx::rows::Rows::surface()?;
    // The block's byte count after the header (0x80), as the original's register has it.
    let mut eax: u32 = 0x80;
    let width = gfx.drawing.draw_surface.width.get();
    let mut y = 0u32;
    loop {
        let format = crate::gfx::pixel_format::layout(gfx.drawing.draw_surface.format.get())?;
        // The row's pixels, as the original reads them (none for other sizes).
        let bits = gfx.drawing.draw_surface.bits_per_pixel.get();
        let pixels = match bits {
            0x10 => rows.read(y, 0, 2 * width as usize, "the surface")?,
            0x20 => rows.read(y, 0, 4 * width as usize, "the surface")?,
            _ => Vec::new(),
        };
        let planes = rc2k_core::gfx::pcx::screen_planes(width, &mut eax, &mut |x, eax| {
            let x = x as usize;
            match bits {
                0x10 => crate::gfx::pixel_to_components::pixel_to_components(
                    &format,
                    eax & 0xFFFF_0000
                        | u16::from_le_bytes([pixels[2 * x], pixels[2 * x + 1]]) as u32,
                ),
                0x20 => crate::gfx::pixel_to_components::pixel_to_components(
                    &format,
                    u32::from_le_bytes([
                        pixels[4 * x],
                        pixels[4 * x + 1],
                        pixels[4 * x + 2],
                        pixels[4 * x + 3],
                    ]),
                ),
                _ => eax,
            }
        });
        gfx.pictures
            .planes
            .write_at(0, &planes, "a screenshot row's planes")?;
        for byte in rc2k_core::gfx::pcx::encode_line(&planes, &mut eax) {
            if !write_stream_byte(stream, byte as u32)? {
                return Ok(false);
            }
        }
        y += 1;
        if y == rows.count() {
            break;
        }
    }
    Ok(true)
}
