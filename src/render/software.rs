//! The software backend: the commands drawn by the rasteriser into the target, in tiles over the
//! processor's threads. Pixel for pixel what the game showed before there were backends.

#![allow(unsafe_code)]

use super::*;
use std::sync::Mutex;

/// The rasteriser as a backend.
pub struct Software;

impl Backend for Software {
    fn draw(&mut self, target: &Target, commands: Vec<Command>) {
        draw(target, commands);
    }
}

/// Rows in each band of the frame the threads draw (16 drew faster than 64 on the PC replays).
const TILE: usize = 16;

/// Below this many commands a flush draws on its own thread, the screen one tile.
const FEW: usize = 48;

/// The depth buffer, a float a pixel of the screen: 1.0 is the farthest.
pub struct Depth {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

pub static DEPTH: Mutex<Option<Depth>> = Mutex::new(None);

/// The depth buffer at the screen's size, made or made again.
pub fn with_depth<R>(f: impl FnOnce(&mut Depth) -> R) -> R {
    let (width, height) = (
        crate::session::SCREEN.width.get() as usize,
        crate::session::SCREEN.height.get() as usize,
    );
    let mut depth = DEPTH.lock().unwrap();
    if depth
        .as_ref()
        .is_none_or(|d| d.width != width || d.height != height)
    {
        *depth = Some(Depth {
            width,
            height,
            values: vec![1.0; width * height],
        });
    }
    f(depth.as_mut().unwrap())
}

/// The small frame draw_noted draws into: only the handheld has one.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
use crate::handheld::Small;
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
enum Small {}
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
impl Small {
    fn frame_parts(&mut self) -> (&mut [u32], usize, usize, &mut [f32]) {
        match *self {}
    }
}

/// The noted things' screen places for the handheld's small frame.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn scale_noted(noted: &mut [Command]) {
    const BY: f32 = crate::handheld::SMALLER as f32;
    let by = crate::handheld::SMALLER;
    for what in noted {
        match what {
            Command::Fan {
                vertices, bounds, ..
            } => {
                for v in vertices.iter_mut() {
                    v[0] = (f32::from_bits(v[0]) / BY).to_bits();
                    v[1] = (f32::from_bits(v[1]) / BY).to_bits();
                }
                *bounds = (bounds.0 / BY, bounds.1 / BY, bounds.2 / BY, bounds.3 / BY);
            }
            Command::Clear([left, top, right, bottom]) => {
                *left /= by;
                *top /= by;
                *right = right.div_ceil(by);
                *bottom = bottom.div_ceil(by);
            }
        }
    }
}

fn draw(into: &Target, noted: Vec<Command>) {
    use crate::rasteriser::{Frame, Pixels, States, Texture};
    use rayon::prelude::*;
    #[cfg_attr(
        not(all(target_os = "linux", target_arch = "aarch64")),
        allow(unused_mut) // only the handheld changes them
    )]
    let mut noted = noted;
    FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let started = std::time::Instant::now();
    let fans = noted
        .iter()
        .filter(|n| matches!(n, Command::Fan { .. }))
        .count() as u64;
    // The handheld draws a race's 3D smaller (`handheld`): the noted things made to fit a small
    // frame, which takes the screen's picture so far and gives it back enlarged.
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let mut small = (into.bits == 32
        && crate::handheld::draws_small()
        && !crate::render::gpu::wanted())
    .then(|| {
        scale_noted(&mut noted);
        // SAFETY: not kept: the screen's rows, checked by `slice`, nothing else writes them
        // while the game's code waits here.
        let screen = unsafe { into.bytes() };
        crate::handheld::Small::from_screen(screen, into.pitch, into.width, into.height)
    });
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    let mut small: Option<Small> = None;
    with_depth(|depth| {
        if small.is_none() && (depth.width != into.width || depth.height != into.height) {
            return;
        }
        // The screen's pixels are lent (a region the size of its rows), checked here; the
        // depths are Rust's own, borrowed for the frame.
        crate::pointers::check_or_end(
            into.pixels as u32,
            into.pitch * into.height,
            "the screen's pixels",
        );
        let mut frame = match small.as_mut().map(Small::frame_parts) {
            Some((pixels, width, height, depth)) => {
                Frame::new(Pixels::Bits32(pixels), width, width, height, depth)
            }
            None => Frame::new(
                // SAFETY: the screen's rows, checked by `pixels`; the game's code does not
                // run while the frame is drawn (it waits for `Backend::draw`).
                unsafe { into.pixels() },
                into.pitch / (into.bits / 8) as usize,
                into.width,
                into.height,
                &mut depth.values,
            ),
        };
        let mut tiles = frame.tiles(if noted.len() < FEW { 0 } else { TILE });
        let noted = &noted;
        let draw_tile = move |tile: &mut crate::rasteriser::Tile| {
            let tile_started = std::time::Instant::now();
            let clip = *tile.clip();
            let mut written = 0u64;
            for what in noted {
                match what {
                    Command::Fan {
                        vertices,
                        states,
                        texture,
                        bounds: (left, top, right, bottom),
                    } => {
                        // Not this tile's (the bounds a pixel wider for the edges' rule).
                        if *right < clip.left as f32 - 1.0
                            || *left > clip.right as f32 + 1.0
                            || *bottom < clip.top as f32 - 1.0
                            || *top > clip.bottom as f32 + 1.0
                        {
                            continue;
                        }
                        let texture = texture.map(|t| Texture {
                            // SAFETY: a noted texture is not written while it is drawn
                            // from: locking it flushes the noted fans first (`lock`).
                            pixels: unsafe {
                                crate::pointers::GamePtr::new(t.pixels as u32)
                                    .slice(t.len, "a texture's pixels")
                            },
                            width: t.width,
                            height: t.height,
                            pitch: t.pitch,
                            bytes: t.bytes,
                            masks: t.masks,
                        });
                        written += tile.draw_fan(vertices, &States(&states[..]), texture.as_ref());
                    }
                    Command::Clear(rect) => tile.clear_depth(*rect),
                }
            }
            PIXELS.fetch_add(written, std::sync::atomic::Ordering::Relaxed);
            TILE_MICROS.fetch_add(
                tile_started.elapsed().as_micros() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
        };
        let parallel_started = std::time::Instant::now();
        if tiles.len() == 1 {
            draw_tile(&mut tiles[0]);
        } else {
            tiles.par_iter_mut().for_each(draw_tile);
        }
        PARALLEL_MICROS.fetch_add(
            parallel_started.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    });
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    if let Some(s) = small {
        // SAFETY: as above.
        s.enlarge_to(unsafe { into.bytes() }, into.pitch, into.width, into.height);
    }
    use std::sync::atomic::Ordering::Relaxed;
    FANS.fetch_add(fans, Relaxed);
    RASTER_MICROS.fetch_add(started.elapsed().as_micros() as u64, Relaxed);
}
