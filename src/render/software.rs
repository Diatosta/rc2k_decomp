//! The software backend: the commands drawn by the rasteriser into the target, in tiles over the
//! processor's threads. Pixel for pixel what the rasteriser alone draws.

#![allow(unsafe_code)]

use super::*;
use crate::render::rasteriser::{MipLevel, Texture, mip_chain};
use std::sync::{Arc, Mutex};

/// The rasteriser as a backend, with the screen's size (the depth buffer's). A race's 3D at a
/// share of that size goes through a small frame (`small_frame`).
pub struct Software {
    pub screen: (usize, usize),
}

impl Backend for Software {
    fn draw(&mut self, target: &Target, mut commands: CommandList) {
        draw(target, &mut commands, self.screen);
        give_back(commands);
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

static DEPTH: Mutex<Option<Depth>> = Mutex::new(None);

/// The depth buffer at the screen's size, made or made again.
fn with_depth<R>(width: usize, height: usize, f: impl FnOnce(&mut Depth) -> R) -> R {
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

/// The smaller levels made for the textures drawn with the game's mip filter, by texture. A
/// texture written or gone is dropped (`render::take_level_changes`) and made again when next
/// drawn from.
static MIP_CHAINS: Mutex<Option<crate::render::TextureMap<Arc<Vec<MipLevel>>>>> = Mutex::new(None);

/// The smaller levels of each texture in `levels` (`Texture.lower`), made once per texture; empty
/// unless a noted fan asks for them (minifying filter 6, which the game sets when its mipmapping
/// option is on), so a frame without the option does nothing here.
fn mip_chains(
    noted: &CommandList,
    levels: &[(super::TextureId, &[u8])],
) -> Vec<(super::TextureId, Arc<Vec<MipLevel>>)> {
    let asked = noted.commands.iter().any(|what| {
        matches!(what, Command::Fan { texture: Some(_), states, .. } if states.get(0x12) == Some(&6))
    });
    if !asked {
        return Vec::new();
    }
    super::keep_level_changes();
    let mut cache = MIP_CHAINS.lock().unwrap();
    let cache = cache.get_or_insert_default();
    for id in super::take_level_changes() {
        cache.remove(&id);
    }
    let mut chains = Vec::new();
    for what in &noted.commands {
        if let Command::Fan {
            texture: Some(t),
            states,
            ..
        } = what
            && states.get(0x12) == Some(&6)
            && !chains.iter().any(|(id, _)| *id == t.pixels)
        {
            let chain = cache.entry(t.pixels).or_insert_with(|| {
                let pixels = levels
                    .iter()
                    .find(|l| l.0 == t.pixels)
                    .map_or(&[][..], |l| l.1);
                Arc::new(mip_chain(&Texture {
                    pixels,
                    width: t.width,
                    height: t.height,
                    pitch: t.pitch,
                    bytes: t.bytes,
                    masks: t.masks,
                    lower: &[],
                }))
            });
            chains.push((t.pixels, Arc::clone(chain)));
        }
    }
    chains
}

fn draw(into: &Target, noted: &mut CommandList, screen: (usize, usize)) {
    use crate::render::rasteriser::{Frame, States, Texture};
    use rayon::prelude::*;
    FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let started = std::time::Instant::now();
    let fans = noted
        .commands
        .iter()
        .filter(|n| matches!(n, Command::Fan { .. }))
        .count() as u64;
    // A race's 3D at a share of the screen's size: the noted things made to fit a small frame,
    // which takes the screen's picture so far and gives it back enlarged.
    let percent = super::flush_scale();
    let mut small =
        (into.bits == 32 && percent != 100 && !crate::render::gpu_running()).then(|| {
            // SAFETY: not kept: the screen's rows, checked by `slice`, nothing else writes them
            // while the game's code waits here.
            let screen = unsafe { into.bytes() };
            let small = crate::render::small_frame::Small::from_screen(
                screen,
                into.pitch,
                into.width,
                into.height,
                percent,
            );
            small.fit_noted(noted);
            small
        });
    with_depth(screen.0, screen.1, |depth| {
        if small.is_none() && (depth.width != into.width || depth.height != into.height) {
            return;
        }
        // The screen's pixels are checked by `pixels`; the depths are Rust's own, borrowed
        // for the frame.
        let mut frame = match small
            .as_mut()
            .map(crate::render::small_frame::Small::frame_parts)
        {
            Some((pixels, width, height, depth)) => Frame::new(pixels, width, width, height, depth),
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
        let noted = &*noted;
        // The noted textures' bytes, taken once (a noted texture is not written while it is
        // drawn from: locking it flushes the noted fans first), by id.
        let mut notes: Vec<super::TextureNote> = Vec::new();
        for what in noted.commands.iter() {
            if let Command::Fan {
                texture: Some(t), ..
            } = what
                && notes.last().is_none_or(|l| l.pixels != t.pixels)
            {
                notes.push(*t);
            }
        }
        notes.sort_unstable_by_key(|t| t.pixels);
        notes.dedup_by_key(|t| t.pixels);
        let levels: Vec<(super::TextureId, &[u8])> = notes
            .iter()
            .map(|t| (t.pixels, super::noted_bytes(t)))
            .collect();
        let chains = mip_chains(noted, &levels);
        let levels = &levels;
        let chains = &chains;
        let draw_tile = move |tile: &mut crate::render::rasteriser::Tile| {
            let tile_started = std::time::Instant::now();
            let clip = *tile.clip();
            let mut written = 0u64;
            for what in &noted.commands {
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
                            pixels: levels
                                .binary_search_by_key(&t.pixels, |l| l.0)
                                .map(|n| levels[n].1)
                                .expect("a noted texture's level was taken above"),
                            width: t.width,
                            height: t.height,
                            pitch: t.pitch,
                            bytes: t.bytes,
                            masks: t.masks,
                            lower: chains
                                .iter()
                                .find(|c| c.0 == t.pixels)
                                .map_or(&[][..], |c| &c.1[..]),
                        });
                        written += tile.draw_fan(
                            noted.fan_vertices(*vertices),
                            &States(&states[..]),
                            texture.as_ref(),
                        );
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
    if let Some(s) = small {
        // SAFETY: as above.
        s.enlarge_to(unsafe { into.bytes() }, into.pitch);
    }
    use std::sync::atomic::Ordering::Relaxed;
    FANS.fetch_add(fans, Relaxed);
    RASTER_MICROS.fetch_add(started.elapsed().as_micros() as u64, Relaxed);
}
