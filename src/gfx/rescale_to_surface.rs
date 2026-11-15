//! Draws the decoded picture onto the surface at the surface's size: `gfx::rescale_picture`'s
//! arithmetic with other ends. The rows are those of the decoded picture where it lies (the decoded
//! stride apart, listed with `gfx::fill_row_table`), and every pixel made goes to the surface in
//! the surface's format: its three parts are put in `components` (whose fourth byte is left as it
//! was), `gfx::components_to_pixel` with the surface's format makes the pixel, and it is stored as
//! a word on a 16-bit surface, else as a dword; the surface's gap is added after each row. Refuses
//! a decoded picture or a surface without width or height, more rows than the row table holds, and
//! missing pixels or format.

use crate::errors::{BadInput, Refused};
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, fill_row_table, row_address};
use crate::gfx::rescale_picture::Rescaler;

/// What one rescale made, kept for the next rescale of the same picture to a surface of the same
/// size and format (the loading picture is drawn once for each page of the screen).
pub struct Rescaled {
    /// The rescale's inputs: the picture's and the surface's sizes, the surface's format and
    /// pixel width, and the components' fourth byte.
    key: [u32; 7],
    /// The surface's pixels, row after row without the gaps.
    pixels: Vec<u8>,
    last_parts: Option<[u8; 3]>,
    last_scratch: Option<[u8; 4]>,
    #[cfg(feature = "remote")]
    fold: u64,
    work: rc2k_core::gfx::picture::Work,
}

/// `surface` is the surface drawn on. `kept` is the last rescale's result: used when it was made
/// from the same inputs (the picture's decoded pixels must be the same too: the caller's promise),
/// else replaced.
pub fn rescale_to_surface(
    rescaler: &Rescaler,
    surface: &crate::gfx::screen_state::DrawSurface,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    kept: &mut Option<Rescaled>,
) -> Result<(), Refused> {
    let pictures = &gfx.pictures;
    let (from_width, from_height): (u32, u32) =
        (pictures.decoder.width.get(), pictures.decoder.height.get());
    let (width, height) = (surface.width.get(), surface.height.get());
    let from: u32 = pictures.decoder.pixels.get();
    if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
        return Err(BadInput("a picture or a surface without width or height").into());
    }
    if from_height > ROW_TABLE_ROOM
        || from == 0
        || surface.pixels.get() == 0
        || surface.format.get() == 0
    {
        return Err(BadInput("a decoded picture or a surface the rescaler cannot take").into());
    }
    // The count was checked above. The rows as offsets in the decoded buffer.
    let _ = fill_row_table(0, pictures.decoder.stride.get(), from_height, gfx);
    let format = crate::gfx::pixel_format::layout(system, gfx, surface.format.get())?;
    let size = if surface.bits_per_pixel.get() == 0x10 {
        2
    } else {
        4
    };
    // The components' fourth byte stays as it was.
    let alpha = rescaler.components.get_u32(0) & 0xFF00_0000;
    let key = [
        from_width,
        from_height,
        width,
        height,
        surface.format.get(),
        size,
        alpha,
    ];
    if kept.as_ref().is_none_or(|k| k.key != key) {
        *kept = Some(rescale_pixels(key, &format, gfx)?);
    }
    let Some(made) = kept.as_ref() else {
        return Ok(());
    };
    // Where the next row goes: the surface's rows are a gap apart.
    let out = crate::gfx::locked_pixels::Place::start(surface.pixels.get());
    let row_len = width as usize * size as usize;
    let stride = row_len as u32 + surface.row_gap.get();
    let recording = crate::gfx::draw2d::enabled()
        && surface.pixels.get() == crate::gfx::locked_pixels::SCREEN
        && crate::gfx::surface::ON_PICTURE.load(std::sync::atomic::Ordering::Relaxed) == 0;
    // On the screen the GPU keeps the picture as the frame's background when it can: the canvas
    // is not written.
    let by_background = size == 4
        && surface.row_gap.get() == 0
        && surface.pixels.get() == crate::gfx::locked_pixels::SCREEN
        && crate::gfx::surface::ON_PICTURE.load(std::sync::atomic::Ordering::Relaxed) == 0
        && crate::platform_3d::screen(system.session.screen.get()).is_some_and(|screen| {
            screen.width == width as usize
                && screen.height == height as usize
                && crate::render::background::defer_pixels(&screen, &made.pixels)
        });
    for (n, row) in made.pixels.chunks_exact(row_len).enumerate() {
        if !by_background {
            out.must_write_bytes_at((n as u32).wrapping_mul(stride), row, "the surface");
        }
        if recording {
            crate::gfx::draw2d::mirror((n as u32).wrapping_mul(stride), row);
        }
    }
    if recording {
        crate::gfx::draw2d::push(crate::gfx::draw2d::Command::Image(
            crate::gfx::draw2d::Image {
                row_len,
                pitch: stride,
                pixels: made.pixels.clone(),
            },
        ));
    }
    if let Some(parts) = made.last_parts {
        for (part, &byte) in parts.iter().enumerate() {
            rescaler.components.set(part, byte);
        }
    }
    if let Some(scratch) = made.last_scratch {
        gfx.pixel.formats.scratch.write(&scratch);
    }
    #[cfg(feature = "remote")]
    crate::race::course_hash::note_texture_row(u32::MAX, 0, 0, &made.fold.to_le_bytes());
    let work = &made.work;
    rescaler.step_x.set(work.step_x);
    rescaler.step_x_fraction.set(work.step_x_fraction);
    rescaler.step_y.set(work.step_y);
    rescaler.step_y_fraction.set(work.step_y_fraction);
    rescaler.weight_x.set(work.weight_x);
    rescaler.weight_y.set(work.weight_y);
    rescaler.columns_left.set(0);
    rescaler.rows_left.set(0);
    Ok(())
}

/// The picture rescaled to the surface's size and format: its pixels row after row, and what the
/// work leaves in the rescaler's variables. The parts of the last pixel are left in the
/// components, and the scratch bytes of the last bit-by-bit packing in the pixel format's scratch.
/// `key` is `rescale_to_surface`'s.
fn rescale_pixels(
    key: [u32; 7],
    format: &rc2k_core::gfx::pixels::PixelFormat,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<Rescaled, Refused> {
    let [from_width, from_height, width, height, _, size, alpha] = key;
    let from: u32 = gfx.pictures.decoder.pixels.get();
    #[cfg(feature = "remote")]
    let mut fold = 0u64;
    // The rows' offsets, read once; a row past the table ends the program as `row_address` does.
    let rows: Vec<u32> = (0..from_height).map(|row| row_address(row, gfx)).collect();
    let mut last_parts = None;
    let mut last_scratch = None;
    let mut pixels = Vec::with_capacity(width as usize * height as usize * size as usize);
    let work = crate::gfx::picture::with_buffer(from, |decoded| {
        rc2k_core::gfx::picture::rescale(
            &rc2k_core::gfx::picture::Sizes {
                from_width,
                from_height,
                width,
                height,
            },
            &|row, byte| match decoded.get(rows[row as usize].wrapping_add(byte) as usize) {
                Some(&b) => b,
                None => crate::fatal::raise::end(
                    "gfx::rescale_to_surface",
                    BadInput("a pixel read past the decoded picture"),
                ),
            },
            &mut |parts| {
                last_parts = Some(parts);
                let components = u32::from_le_bytes([parts[0], parts[1], parts[2], 0]) | alpha;
                let (pixel, scratch) = rc2k_core::gfx::pixels::pack(format, components);
                last_scratch = scratch.or(last_scratch);
                #[cfg(feature = "remote")]
                {
                    fold = fold.rotate_left(5).wrapping_mul(0x0100_0000_01B3) ^ u64::from(pixel);
                }
                if size == 2 {
                    pixels.extend_from_slice(&(pixel as u16).to_le_bytes());
                } else {
                    pixels.extend_from_slice(&pixel.to_le_bytes());
                }
            },
        )
        .expect("the sizes were checked")
    })?;
    Ok(Rescaled {
        key,
        pixels,
        last_parts,
        last_scratch,
        #[cfg(feature = "remote")]
        fold,
        work,
    })
}
