//! What `gfx::palette_pixels_to_dwords` and `gfx::rgb_pixels_to_dwords` share: both spread
//! the picture buffer's freshly decoded pixels, one or three bytes each and one after the
//! other, out to a dword a pixel, in place, from the last pixel down.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write};
use crate::gfx::harness::{check_pixels, dispatch_pixels};
use crate::gfx::picture::{HEIGHT_RVA, WIDTH_RVA, picture};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, restore};

/// The number of pixels, `g_pictureWidth` * `g_pictureHeight`.
/// Refuses none (the original then writes before the buffer) and more than a signed number
/// holds (it then does one pixel and stops), and a null buffer.
pub unsafe fn pixel_count() -> Result<usize, BadInput> {
    let count = unsafe { game_read::<u32>(WIDTH_RVA).wrapping_mul(game_read::<u32>(HEIGHT_RVA)) };
    if picture().is_null() {
        return Err(BadInput("no picture buffer"));
    }
    if count == 0 || count > i32::MAX as u32 {
        return Err(BadInput("a picture of no pixels, or of too many"));
    }
    Ok(count as usize)
}

pub type Spread = unsafe fn() -> Result<(), BadInput>;

pub fn dispatch_spread(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, rust: Spread) {
    let regions: Vec<Region> = match unsafe { pixel_count() } {
        Ok(count) => vec![(picture() as u32, count * 4)],
        Err(_) => Vec::new(),
    };
    dispatch_pixels(hook, r, call_original, &regions, &|_| {
        unsafe { rust() }.map_err(Refused::from)?;
        Ok(Left::nothing())
    });
}

/// `dispatch_spread` without its hook (a refusal is the caller's).
pub fn spread_entry(_: &mut Regs, rust: Spread) -> Result<(), Refused> {
    unsafe { rust() }?;
    Ok(())
}

/// The self-test of either: small pictures in the game's own buffer, which is put back.
pub fn check_spread(t: &mut SelfTest, original: usize, rust: Spread) {
    const BYTES: usize = 0x1000;
    let saved_sizes = capture(&[at(WIDTH_RVA), at(HEIGHT_RVA)]);
    let buffer = [(picture() as u32, BYTES)];
    let saved = capture(&buffer);
    for (width, height, seed) in [
        (1u32, 1u32, 3u32),
        (7, 5, 1),
        (16, 16, 0x9E37),
        (300, 3, 77),
    ] {
        let prepare = || unsafe {
            game_write(WIDTH_RVA, width);
            game_write(HEIGHT_RVA, height);
            for n in 0..BYTES {
                picture()
                    .add(n)
                    .write((n as u32).wrapping_mul(seed).wrapping_add(n as u32 >> 3) as u8);
            }
        };
        check_pixels(
            t,
            &format!("{width}x{height} seed {seed:#x}"),
            original,
            Regs::SENTINEL,
            &prepare,
            &buffer,
            &|_| {
                unsafe { rust() }.map_err(Refused::from)?;
                Ok(Left::nothing())
            },
        );
    }
    restore(&buffer, &saved);
    restore(&[at(WIDTH_RVA), at(HEIGHT_RVA)], &saved_sizes);
}
