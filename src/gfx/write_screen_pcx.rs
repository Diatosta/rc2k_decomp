//! Writes the locked surface as a 24-bit PNG file (an older version wrote PCX). The
//! file is `name` when set, else the next screenshot in `savedata\snapshot\`: one more than the
//! highest `NNN.png` already there (999 goes round to 000), so no picture is written over.
//!
//! Kept as the game's screenshots had it: a 16-bit pixel goes into the low half of the running
//! value, whose upper half the conversion gets as it was, and pixels of other sizes are not
//! read (the last value is used). Refuses what `open_stream` refuses.

use crate::errors::Refused;
use crate::files::search::DIRECTORY;
use crate::files::stream::with_stream;
use crate::files::write_stream_block::write_stream_all;

/// The folder every screenshot goes into (the key's and the replay viewer's snapshot button).
pub const SNAPSHOT_FOLDER: &[u8] = b"savedata\\snapshot";

/// Makes the screenshots' folder when it is not there yet (a failure shows when the file is
/// opened).
pub fn make_snapshot_folder() {
    let _ = std::fs::create_dir_all(crate::files::host_path(SNAPSHOT_FOLDER));
}

/// The highest number of the `NNN.png` files in the screenshots' folder (0 for none).
pub fn highest_snapshot_number() -> u32 {
    let mut pattern = SNAPSHOT_FOLDER.to_vec();
    pattern.extend_from_slice(b"\\???.png");
    let mut highest = 0;
    if crate::files::search::first(&pattern, DIRECTORY) {
        loop {
            let name = crate::files::search::found_name();
            if let Some(number) =
                rc2k_core::race::viewer::snapshot_number(|n| name.get(n).copied().unwrap_or(0))
            {
                highest = highest.max(number);
            }
            if !crate::files::search::next(DIRECTORY) {
                break;
            }
        }
    }
    crate::files::search::close();
    highest
}

/// The next screenshot's path: one more than the folder's highest number (999 goes round to 000).
fn next_snapshot_path() -> Vec<u8> {
    make_snapshot_folder();
    let highest = format!("{:03}", highest_snapshot_number() % 1000);
    let digits: [u8; 3] = highest.as_bytes().try_into().expect("three digits");
    let (next, _, _) = rc2k_core::race::viewer::next_snapshot(digits);
    let mut path = SNAPSHOT_FOLDER.to_vec();
    path.push(b'\\');
    path.extend_from_slice(&next);
    path.extend_from_slice(b".png");
    path
}

/// Whether it worked. `name` is the file's path without its NUL, or None for the next numbered
/// screenshot. `to_photos`: a picture written is also added to the phone's photo library
/// (the screenshots; not the replay's picture sequence).
pub fn write_screen_pcx(
    name: Option<&[u8]>,
    to_photos: bool,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    // A frame the GPU holds (a race, a replay's view, a 3D menu) is not in the canvas yet: the
    // picture would be the canvas's key colour where the 3D is.
    if let Some(screen) = crate::d3d::device::screen(system.session.screen.get()) {
        crate::render::bring_back(&screen);
    }
    let path = match name {
        Some(name) => name.to_vec(),
        None => next_snapshot_path(),
    };
    with_stream(|stream| {
        if !crate::files::open_stream::open_stream(stream, &path, 1)? {
            return Ok(false);
        }
        let mut ok = match picture(gfx, system)? {
            Some(png) => {
                let written = write_stream_all(stream, &png)?;
                // The browser keeps no file the player can reach: the picture is also handed
                // over as a download.
                #[cfg(web)]
                if written {
                    crate::platform::ffi::web::download(
                        &String::from_utf8_lossy(&path),
                        &png,
                        "image/png",
                    );
                }
                written
            }
            None => false,
        };
        let closed = crate::files::close_stream::close_stream(stream)?;
        ok = ok && closed;
        Ok(ok)
    })
    .inspect(|&written| {
        if written && to_photos {
            crate::platform::ffi::photo_library::add(&crate::files::host_path(&path));
        }
    })
}

/// The surface as a PNG file's bytes.
fn picture(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<Option<Vec<u8>>, Refused> {
    let rows = crate::gfx::rows::Rows::surface(gfx)?;
    // The running value the conversion carries from pixel to pixel.
    let mut carried: u32 = 0x80;
    let mut rgb = Vec::new();
    let width = gfx.drawing.draw_surface.width.get();
    let mut y = 0u32;
    loop {
        let format =
            crate::gfx::pixel_format::layout(system, gfx, gfx.drawing.draw_surface.format.get())?;
        // The row's pixels (none for other sizes).
        let bits = gfx.drawing.draw_surface.bits_per_pixel.get();
        let pixels = match bits {
            0x10 => rows.read(y, 0, 2 * width as usize, "the surface")?,
            0x20 => rows.read(y, 0, 4 * width as usize, "the surface")?,
            _ => Vec::new(),
        };
        let planes = rc2k_core::gfx::pcx::screen_planes(width, &mut carried, &mut |x, carried| {
            let x = x as usize;
            match bits {
                0x10 => crate::gfx::pixel_to_components::pixel_to_components(
                    &format,
                    carried & 0xFFFF_0000
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
                _ => carried,
            }
        });
        let w = width as usize;
        for x in 0..w {
            rgb.extend_from_slice(&[planes[x], planes[w + x], planes[2 * w + x]]);
        }
        y += 1;
        if y == rows.count() {
            break;
        }
    }
    Ok(Some(rc2k_core::gfx::png_write::encode_rgb(
        width,
        rows.count(),
        &rgb,
    )))
}
