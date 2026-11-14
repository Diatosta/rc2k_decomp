//! The CPU executor of a recorded frame: every command drawn into a canvas by the drawing code the
//! funnels use, so a replay is the direct drawing again.

use super::{Background, Canvas, Clear, Command, Frame, Glyph, Image, Rect, Shape};
use crate::gfx::blit_glyph::{blit_glyph_by, glyph_record};
use crate::gfx::surface::{Clip, Surface, Target};
use rc2k_core::gfx::raster;

/// Why a command could not be replayed, or replayed differently.
#[derive(Debug, PartialEq, Eq)]
pub enum Problem {
    /// The drawing code refused it.
    Refused(String),
    /// A write past the canvas.
    PastCanvas,
    /// The blitter's work variables after the blit differ from the recorded ones.
    ScratchDiffers,
    /// The picture buffer changed after the background was noted: the record names a state it no
    /// longer holds.
    PictureChanged,
    /// The glyph's palette could not be read when it was drawn, but the replay needs it.
    NoPalette,
}

/// Writes into `canvas`, noting a write past its end.
struct Writer<'a> {
    canvas: &'a mut [u8],
    past: bool,
}

impl Writer<'_> {
    fn put(&mut self, offset: u32, bytes: &[u8]) {
        let at = offset as usize;
        match self.canvas.get_mut(at..at.wrapping_add(bytes.len())) {
            Some(place) => place.copy_from_slice(bytes),
            None => self.past = true,
        }
    }

    fn ok(&self) -> Result<(), Problem> {
        if self.past {
            Err(Problem::PastCanvas)
        } else {
            Ok(())
        }
    }
}

/// Replays `frame` into `canvas`; on a problem the number of the command and the problem.
pub fn replay(
    frame: &Frame,
    fonts: &crate::gfx::owner::FontStores,
    canvas: &mut [u8],
) -> Result<(), (usize, Problem)> {
    let mut writer = Writer {
        canvas,
        past: false,
    };
    for (n, command) in frame.commands.iter().enumerate() {
        let done = match command {
            Command::Glyph(glyph) => draw_glyph(glyph, frame, fonts, &mut writer),
            Command::Rect(rect) => draw_rect(rect, &mut writer),
            Command::Background(background) => draw_background(background, &mut writer),
            Command::Image(image) => draw_image(image, &mut writer),
            Command::Clear(clear) => draw_clear(clear, &mut writer),
        };
        done.and_then(|()| writer.ok()).map_err(|p| (n, p))?;
    }
    Ok(())
}

fn refused(e: crate::errors::Refused) -> Problem {
    Problem::Refused(format!("{e:?}"))
}

fn draw_glyph(
    glyph: &Glyph,
    frame: &Frame,
    fonts: &crate::gfx::owner::FontStores,
    writer: &mut Writer<'_>,
) -> Result<(), Problem> {
    let Glyph {
        call,
        canvas: Canvas { surface, clip },
        ..
    } = glyph;
    let on = Surface {
        target: Some(Target::Locked(crate::gfx::locked_pixels::SCREEN)),
        pitch: surface.pitch,
        bytes_per_pixel: surface.bytes_per_pixel,
        bits_per_pixel: surface.bits,
    };
    let record = glyph_record(fonts, &on, call.table, call.glyph).map_err(refused)?;
    let colours = match glyph.palette {
        Some(n) => Some(frame.palettes.get(n).ok_or(Problem::NoPalette)?.as_slice()),
        None if call.palette.is_some() => return Err(Problem::NoPalette),
        None => None,
    };
    let colour_at = |n: u8| {
        let at = 4 * n as usize;
        let entry = colours?.get(at..at + 4)?;
        Some(u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]))
    };
    let mut scratch = glyph.before;
    blit_glyph_by(
        &record,
        call.x,
        call.y,
        (&colour_at, call.palette.is_some()),
        &on,
        &Clip {
            left: clip.left,
            right: clip.right,
            top: clip.top,
            bottom: clip.bottom,
        },
        &mut scratch,
        &mut |offset, bytes| writer.put(offset, bytes),
    )
    .map_err(refused)?;
    if scratch == glyph.after {
        Ok(())
    } else {
        Err(Problem::ScratchDiffers)
    }
}

fn draw_rect(rect: &Rect, writer: &mut Writer<'_>) -> Result<(), Problem> {
    let Canvas { surface, clip } = &rect.canvas;
    let plans = match rect.shape {
        Shape::Fill(shape) => vec![raster::fill(shape, clip, surface)],
        Shape::HLine { x, y, length } => vec![Ok(raster::hline(x, y, length, clip, surface))],
        Shape::Frame { rect, dx, dy } => raster::frame(rect, dx, dy, clip, surface)
            .into_iter()
            .map(Ok)
            .collect(),
    };
    for plan in plans {
        let plan = plan.map_err(|e| Problem::Refused(e.0.to_string()))?;
        if !plan.pixels.is_empty() && rect.pixel.is_empty() {
            return Err(Problem::NoPalette);
        }
        for &offset in &plan.pixels {
            writer.put(offset as u32, &rect.pixel);
        }
    }
    Ok(())
}

fn draw_background(background: &Background, writer: &mut Writer<'_>) -> Result<(), Problem> {
    if crate::gfx::draw2d::picture_generation(background.buffer) != Some(background.generation) {
        return Err(Problem::PictureChanged);
    }
    crate::gfx::picture::with_buffer_unchanged(background.buffer, |from| {
        for y in 0..background.rows {
            let at = y as usize * background.row_bytes;
            let row = from
                .get(at..at + background.row_bytes)
                .ok_or(Problem::PastCanvas)?;
            writer.put(y * background.pitch, row);
        }
        Ok(())
    })
    .map_err(|e| Problem::Refused(format!("{e:?}")))?
}

fn draw_image(image: &Image, writer: &mut Writer<'_>) -> Result<(), Problem> {
    for (n, row) in image.pixels.chunks_exact(image.row_len).enumerate() {
        writer.put((n as u32).wrapping_mul(image.pitch), row);
    }
    Ok(())
}

fn draw_clear(clear: &Clear, writer: &mut Writer<'_>) -> Result<(), Problem> {
    let zeroes = vec![0; clear.row_len];
    for y in 0..clear.rows {
        writer.put(y.wrapping_mul(clear.pitch), &zeroes);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::blit_glyph::{BlitCall, Scratch, blit_glyph};
    use crate::gfx::draw2d::Glyph;
    use rc2k_core::gfx::raster::Clip as RasterClip;

    const WIDTH: i32 = 40;
    const HEIGHT: i32 = 30;
    const PITCH: i32 = WIDTH * 4 + 8;

    fn surface() -> rc2k_core::gfx::raster::Surface {
        rc2k_core::gfx::raster::Surface {
            bits: 32,
            bytes_per_pixel: 4,
            pitch: PITCH,
        }
    }

    struct Random(u64);

    impl Random {
        fn between(&mut self, low: i32, high: i32) -> i32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            low + ((self.0 >> 33) % (high - low + 1) as u64) as i32
        }
    }

    /// A 3 x 2 glyph: row 1 a gap of 1 and pixels 1, 2; row 2 pixels 3, 4, 5.
    const GLYPH: [u8; 15] = [0, 0, 3, 2, 2, 1, 1, 2, 0, 3, 0, 3, 4, 5, 0];

    /// Draws random glyphs with two states of one palette both directly and as recorded commands:
    /// the replay is the direct canvas, the states being what each glyph drew with.
    #[test]
    fn a_replayed_frame_is_the_direct_drawing() {
        let fonts = &crate::gfx::owner::gfx().fonts;
        let _lock = crate::test_support::test_lock();
        let list = crate::memory::ListCells::new(crate::memory::MemoryList {
            head: 0,
            total: 0,
            cursor: 0,
            remaining: 0,
        });
        let key = crate::gfx::glyph_store::key(&list);
        let palette = crate::gfx::glyph_store::new_palette(&list).unwrap();
        let table = crate::gfx::glyph_store::new_table(&list).unwrap();
        let mut bytes = vec![8, 0, 0, 0, 8 + GLYPH.len() as u8, 0, 0, 0];
        bytes.extend_from_slice(&GLYPH);
        bytes.extend_from_slice(&GLYPH);
        crate::gfx::glyph_store::write_bytes(fonts, table, 0, &bytes, "a table").unwrap();
        let write_palette = |seed: u32| {
            let colours: Vec<u8> = (0..256u32)
                .flat_map(|n| (n.wrapping_mul(0x0101_0101) ^ seed).to_le_bytes())
                .collect();
            crate::gfx::glyph_store::write_bytes(fonts, palette, 0, &colours, "a palette").unwrap();
            colours
        };

        let mut random = Random(11);
        let mut direct = vec![0xAAu8; PITCH as usize * HEIGHT as usize];
        let mut frame = Frame::default();
        let on = Surface {
            target: Some(Target::Locked(crate::gfx::locked_pixels::SCREEN)),
            pitch: PITCH,
            bytes_per_pixel: 4,
            bits_per_pixel: 32,
        };
        let mut scratch = Scratch {
            left_skip: 0,
            visible_w: 0,
            top_skip: 0,
            rows_left: 0,
        };
        for state in 0..2 {
            frame
                .palettes
                .push(write_palette(0x0040_2010 * (state + 1)));
            for _ in 0..300 {
                let (x, y) = (
                    random.between(-4, WIDTH + 2),
                    random.between(-4, HEIGHT + 2),
                );
                let left = random.between(0, 20);
                let top = random.between(0, 14);
                let clip = RasterClip {
                    left,
                    right: (left + random.between(0, 30)).min(WIDTH - 1),
                    top,
                    bottom: (top + random.between(0, 20)).min(HEIGHT - 1),
                };
                let number = random.between(0, 1) as u32;
                let record = glyph_record(fonts, &on, table, number).unwrap();
                let before = scratch;
                blit_glyph(
                    fonts,
                    &record,
                    x,
                    y,
                    Some(palette),
                    &on,
                    &Clip {
                        left: clip.left,
                        right: clip.right,
                        top: clip.top,
                        bottom: clip.bottom,
                    },
                    &mut scratch,
                    &mut |offset, bytes| {
                        direct[offset as usize..][..bytes.len()].copy_from_slice(bytes);
                    },
                )
                .unwrap();
                frame.commands.push(Command::Glyph(Glyph {
                    call: BlitCall::glyph(number, x, y, palette, table, clip.right),
                    canvas: Canvas {
                        surface: surface(),
                        clip,
                    },
                    palette: Some(frame.palettes.len() - 1),
                    before,
                    after: scratch,
                }));
            }
        }
        let mut replayed = vec![0xAAu8; direct.len()];
        assert_eq!(replay(&frame, fonts, &mut replayed), Ok(()));
        assert_eq!(replayed, direct);
        // A recorded work variable that is not what the blit leaves is a difference.
        let Some(Command::Glyph(last)) = frame.commands.last_mut() else {
            unreachable!()
        };
        last.after.rows_left += 1;
        assert_eq!(
            replay(&frame, fonts, &mut vec![0; direct.len()]),
            Err((frame.commands.len() - 1, Problem::ScratchDiffers))
        );
        crate::gfx::glyph_store::give_back(key);
    }

    #[test]
    fn solid_shapes_images_and_clears_replay_as_drawn() {
        let fonts = &crate::gfx::owner::gfx().fonts;
        let clip = RasterClip {
            left: 2,
            right: 30,
            top: 1,
            bottom: 20,
        };
        let canvas = Canvas {
            surface: rc2k_core::gfx::raster::Surface {
                bits: 16,
                bytes_per_pixel: 2,
                pitch: 80,
            },
            clip,
        };
        let pixel = vec![0x34, 0x12];
        let mut frame = Frame::default();
        for shape in [
            Shape::HLine {
                x: 0,
                y: 3,
                length: 12,
            },
            Shape::Frame {
                rect: [4, 5, 10, 8],
                dx: 1,
                dy: 0,
            },
            Shape::Fill([20, 10, 30, 30]),
        ] {
            frame.commands.push(Command::Rect(Rect {
                shape,
                canvas,
                pixel: pixel.clone(),
            }));
        }
        frame.commands.push(Command::Image(Image {
            row_len: 6,
            pitch: 80,
            pixels: (0..12).collect(),
        }));
        frame.commands.push(Command::Clear(Clear {
            row_len: 4,
            pitch: 80,
            rows: 2,
        }));
        let mut canvas = vec![0xAAu8; 80 * 30];
        assert_eq!(replay(&frame, fonts, &mut canvas), Ok(()));
        // The line's pixel 2 in row 3; its 13th pixel is past the 12 asked for.
        assert_eq!(canvas[80 * 3 + 4..][..2], pixel[..]);
        assert_eq!(canvas[80 * 3 + 24..][..2], [0xAA; 2]);
        // The image's two rows, then the clear over the start of both.
        assert_eq!(canvas[4..][..2], [4, 5]);
        assert_eq!(canvas[80 + 4..][..2], [10, 11]);
        assert_eq!(canvas[..4], [0; 4]);
        assert_eq!(canvas[80..][..4], [0; 4]);
        // A write past the canvas is a problem.
        assert_eq!(
            replay(&frame, fonts, &mut vec![0; 80 * 10]),
            Err((1, Problem::PastCanvas))
        );
    }
}
