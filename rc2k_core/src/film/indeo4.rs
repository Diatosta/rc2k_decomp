//! A decoder for Indeo Video Interactive 4 (the `IV41` pictures of an AVI film): packets
//! in, pictures out as three planes, one chroma sample for every four by four of luma.
//!
//! This is a port to Rust of FFmpeg's decoder, and gives the same pictures byte for byte:
//!
//! - `libavcodec/indeo4.c`: Indeo Video Interactive v4 compatible decoder,
//!   Copyright (c) 2009-2011 Maxim Poliakovski;
//! - `libavcodec/ivi.c`: common functions for Indeo Video Interactive codecs (Indeo 4 and
//!   Indeo 5), Copyright (c) 2009 Maxim Poliakovski;
//! - `libavcodec/ivi_dsp.c`: DSP functions for Indeo Video Interactive codecs,
//!   Copyright (c) 2009-2011 Maxim Poliakovski (in `transforms`);
//! - `libavcodec/indeo4data.h`: the tables (in `tables`).
//!
//! FFmpeg is free software under the GNU Lesser General Public License, version 2.1 or
//! (at your option) any later version; this port is distributed with the rest of the
//! program under the GNU General Public License, version 3 or later, as that licence
//! allows.
//!
//! What a film's pictures are made of: each of the three planes is one band of samples,
//! cut into tiles, each tile into macroblocks, each macroblock into one or four blocks. A
//! block is either coded (Huffman-coded runs and values of transform coefficients) or
//! taken from the picture before, moved by the macroblock's motion vector. Decoding goes
//! into one of the band's sample buffers while another holds the reference picture.
//!
//! Left out, and refused with `Refused::NotImplemented`: pictures split into four luma
//! bands (the scalable mode) and bidirectional frames. A damaged packet is refused with
//! `Refused::Damaged`; nothing here panics or reads outside a packet or a buffer.

mod bits;
mod tables;
mod transforms;

#[cfg(test)]
mod test_streams;
#[cfg(test)]
mod tests;

use bits::{BLOCK_BOOKS, Bits, BookShape, CodeBook, MACROBLOCK_BOOKS};
use std::sync::Arc;
use tables::{
    COMMON_PICTURE_SIZES, QUANT_4X4_INTER, QUANT_4X4_INTRA, QUANT_8X8_INTER, QUANT_8X8_INTRA,
    QUANT_INDEX_TO_TABLE, RUN_VALUE_MAPS, RunValueMap, SCANS,
};
use transforms::{Interpolation, Transform, compensate};

/// Why a packet gave no picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    /// The packet is not a sound Indeo 4 frame.
    Damaged(&'static str),
    /// The packet asks for something this decoder does not do.
    NotImplemented(&'static str),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Damaged(why) => write!(f, "a damaged Indeo 4 frame: {why}"),
            Self::NotImplemented(what) => write!(f, "Indeo 4: {what} is not implemented"),
        }
    }
}

impl std::error::Error for Refused {}

/// The largest picture side taken (the format allows 65,535; the buffers for that would
/// be gigabytes).
pub const LARGEST_SIDE: usize = 2048;

/// A decoded picture: full-size luma, and two chroma planes a quarter as wide and high
/// (rounded up), each plane row after row with no padding.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub chroma_width: usize,
    pub chroma_height: usize,
    /// Luma, then the blue-difference and red-difference planes.
    pub planes: [Vec<u8>; 3],
}

impl Picture {
    /// The pixel format's name as FFmpeg has it.
    pub const PIXEL_FORMAT_NAME: &'static str = "yuv410p";

    /// The picture's width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The picture's height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Bytes a row of plane `p` takes.
    pub fn line_size(&self, p: usize) -> usize {
        if p == 0 {
            self.width
        } else {
            self.chroma_width
        }
    }

    /// The first `rows` rows of plane `p`; None for a plane with fewer.
    pub fn plane(&self, p: usize, rows: usize) -> Option<&[u8]> {
        self.planes.get(p)?.get(..rows * self.line_size(p))
    }
}

// The kinds of frame.
const INTRA: u32 = 0;
const INTRA_OTHER: u32 = 1;
const INTER: u32 = 2;
const BIDIRECTIONAL: u32 = 3;
/// The first frame kind with no picture data: 5 (nothing shown) and 6 (the picture held
/// back is shown).
const EMPTY_FIRST: u32 = 5;
const EMPTY_LAST: u32 = 6;

/// A picture's start code (18 bits), and the same followed by the inter frame kind (21).
const START_CODE: u32 = 0x3FFF8;
const START_CODE_INTER: u32 = 0xBFFF8;

/// The picture's dimensions as its header gives them; buffers are made anew when any
/// changes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Layout {
    width: usize,
    height: usize,
    tile_width: usize,
    tile_height: usize,
}

/// A code book in use and the custom book (with its description) last sent for the place.
#[derive(Default)]
struct BookChoice {
    book: Option<Arc<CodeBook>>,
    custom_shape: BookShape,
    custom: Option<Arc<CodeBook>>,
}

/// A macroblock's values, kept from frame to frame: the chroma planes take theirs from
/// the luma plane's.
#[derive(Clone, Copy, Default)]
struct Macroblock {
    x: i16,
    y: i16,
    /// Where its samples start in the band's buffer.
    offset: u32,
    /// Coded from the picture before (with a motion vector) rather than on its own.
    inter: bool,
    /// A bit for each of its blocks that has coefficients.
    coded_blocks: u8,
    quant_delta: i8,
    motion_x: i8,
    motion_y: i8,
}

#[derive(Default)]
struct Tile {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    /// The macroblock size its macroblocks were counted for.
    macroblock_size: usize,
    macroblocks: Vec<Macroblock>,
}

/// What a band's header sets; a header that leaves a part out keeps the last one's.
#[derive(Clone, Default)]
struct BandSetup {
    empty: bool,
    macroblock_size: usize,
    block_size: usize,
    /// Motion vectors count half samples.
    half_sample: bool,
    inherit_motion: bool,
    inherit_quant_delta: bool,
    quant_matrix: usize,
    quant: u32,
    scan: Option<&'static [u8]>,
    scan_size: usize,
    transform: Option<Transform>,
    transform_size: usize,
    run_value_map: usize,
    /// Pairs of symbols whose meanings are swapped in the run / value map, for this band
    /// in this frame.
    corrections: Vec<u8>,
}

/// One plane's samples: four buffers of which one is being decoded into and another is
/// the reference.
#[derive(Default)]
struct Band {
    plane: usize,
    width: usize,
    height: usize,
    /// The buffers' row length and row count: width and height rounded up to whole
    /// macroblocks.
    pitch: usize,
    rows: usize,
    buffers: [Vec<i16>; 4],
    /// The buffer last decoded into.
    current: Option<usize>,
    setup: BandSetup,
    books: BookChoice,
    tiles: Vec<Tile>,
}

/// What decoding a band needs of the frame.
struct FrameValues<'a> {
    kind: u32,
    /// Quantiser deltas are sent for every luma macroblock.
    quant_deltas_sent: bool,
    macroblock_book: &'a CodeBook,
    block_book: Option<&'a Arc<CodeBook>>,
    block_books: &'a [Arc<CodeBook>; 8],
    target: usize,
    reference: usize,
}

/// An Indeo 4 decoder: its state runs from one packet to the next.
pub struct Decoder {
    macroblock_books: [Arc<CodeBook>; 8],
    block_books: [Arc<CodeBook>; 8],
    frame_kind: u32,
    frame_kind_before: u32,
    layout: Option<Layout>,
    bands: [Band; 3],
    /// The buffers' numbers: decoded into, the reference, and the one kept aside while a
    /// frame nothing refers to is decoded.
    target: usize,
    reference: usize,
    aside: usize,
    /// A buffer whose frame did not decode whole.
    spoiled: [bool; 4],
    macroblock_book: BookChoice,
    block_book: BookChoice,
    quant_deltas_sent: bool,
    /// The picture of an inter frame that came in an intra frame's packet, shown at the
    /// next empty frame of the showing kind.
    held: Option<Picture>,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

/// A value whose sign is in its lowest bit, as a signed number.
fn signed(value: i32) -> i32 {
    ((value >> 1) ^ (value & 1).wrapping_neg()).wrapping_neg()
}

/// A motion vector of the luma plane scaled down for a band with smaller macroblocks.
fn scaled_motion(motion: i8, scale: u32) -> i8 {
    let motion = i32::from(motion);
    if scale == 0 {
        return motion as i8;
    }
    ((motion + i32::from(motion > 0) + (scale as i32 - 1)) >> scale) as i8
}

fn books(shapes: &[BookShape; 8]) -> [Arc<CodeBook>; 8] {
    std::array::from_fn(|i| {
        // the listed shapes have no code longer than a book takes (tested)
        Arc::new(CodeBook::build(&shapes[i]).unwrap_or_else(|_| unreachable!()))
    })
}

/// A code book choice read from a header: the usual book when `sent` is not set, else a
/// listed book's number or a custom book's description.
fn read_book_choice(
    bits: &mut Bits,
    sent: bool,
    listed: &[Arc<CodeBook>; 8],
    choice: &mut BookChoice,
) -> Result<(), Refused> {
    if !sent {
        choice.book = Some(listed[7].clone());
        return Ok(());
    }
    let number = bits.take(3) as usize;
    if number != 7 {
        choice.book = Some(listed[number].clone());
        return Ok(());
    }
    let mut shape = BookShape {
        rows: bits.take(4) as usize,
        ..BookShape::default()
    };
    if shape.rows == 0 {
        return Err(Refused::Damaged("an empty custom code book"));
    }
    for row in 0..shape.rows {
        shape.extra_bits[row] = bits.take(4) as u8;
    }
    if choice.custom.is_none() || !shape.same_as(&choice.custom_shape) {
        choice.custom = None;
        choice.custom_shape = BookShape::default();
        choice.custom = Some(Arc::new(CodeBook::build(&shape)?));
        choice.custom_shape = shape;
    }
    choice.book.clone_from(&choice.custom);
    Ok(())
}

/// A tile's data size from its header (0 when none is given); the reader is left at a
/// whole byte.
fn read_tile_data_size(bits: &mut Bits) -> usize {
    let mut size = 0;
    if bits.flag() {
        size = bits.take(8);
        if size == 255 {
            size = bits.take(24);
        }
    }
    bits.align();
    size as usize
}

/// How many macroblocks of `size` a side cover a tile.
fn macroblocks_in(width: usize, height: usize, size: usize) -> usize {
    width.div_ceil(size) * height.div_ceil(size)
}

impl Band {
    fn samples(&self) -> usize {
        self.pitch * self.rows
    }

    /// Buffer `number`, made (all zero) at its first use.
    fn prepare(&mut self, number: usize) {
        if self.buffers[number].is_empty() {
            self.buffers[number] = vec![0; self.samples()];
        }
    }

    /// The band's header: which parts follow, the block sizes, transform, scan order,
    /// quantisation and code book. Nothing is kept of a header that is refused (but a
    /// custom code book it sent).
    fn read_header(&mut self, bits: &mut Bits, frame: &FrameValues) -> Result<(), Refused> {
        let mut setup = self.setup.clone();
        let plane = bits.take(2) as usize;
        let band_number = bits.take(4);
        if plane != self.plane || band_number != 0 {
            return Err(Refused::Damaged("a band header out of sequence"));
        }
        setup.empty = bits.flag();
        if !setup.empty {
            let block_size_before = setup.block_size;
            // the header's size, when given, is not needed
            if bits.flag() {
                bits.skip(16);
            }
            let motion_resolution = bits.take(2);
            if motion_resolution >= 2 {
                return Err(Refused::Damaged("an unknown motion vector resolution"));
            }
            setup.half_sample = motion_resolution == 1;
            // a checksum of the band's samples, not checked
            if bits.flag() {
                bits.skip(16);
            }
            let sizes = bits.take(2);
            if sizes == 3 {
                return Err(Refused::Damaged("an unknown block size"));
            }
            setup.macroblock_size = 16 >> sizes;
            setup.block_size = 8 >> (sizes >> 1);
            setup.inherit_motion = bits.flag();
            setup.inherit_quant_delta = bits.flag();
            setup.quant = bits.take(5);

            // The transform, scan and matrix: always in an intra frame, else when the
            // flag is clear. (The flag's bit is there in either case.)
            let kept = bits.flag();
            if !kept || frame.kind == INTRA {
                let number = bits.take(5);
                let Some(transform) = Transform::numbered(number) else {
                    return Err(Refused::NotImplemented("this transform"));
                };
                if transform.size() == 8 && setup.block_size < 8 {
                    return Err(Refused::Damaged("an 8x8 transform for 4x4 blocks"));
                }
                setup.transform = Some(transform);
                setup.transform_size = transform.size();
                if setup.block_size != setup.transform_size {
                    return Err(Refused::Damaged("transform and block size differ"));
                }
                let scan = bits.take(4) as usize;
                if scan == 15 {
                    return Err(Refused::Damaged("a custom scan order"));
                }
                if ((5..10).contains(&scan) && setup.block_size != 4)
                    || (!(5..10).contains(&scan) && setup.block_size != 8)
                {
                    return Err(Refused::Damaged("a scan order of another block size"));
                }
                setup.scan = Some(SCANS[scan]);
                setup.scan_size = setup.block_size;
                let matrix = bits.take(5) as usize;
                if matrix == 31 {
                    return Err(Refused::Damaged("a custom quantisation matrix"));
                }
                if matrix >= QUANT_INDEX_TO_TABLE.len() {
                    return Err(Refused::Damaged("an unknown quantisation matrix"));
                }
                setup.quant_matrix = matrix;
            } else if block_size_before != setup.block_size {
                return Err(Refused::Damaged("a block size other than the one kept"));
            }
            if QUANT_INDEX_TO_TABLE[setup.quant_matrix] > 4 && setup.block_size == 4 {
                return Err(Refused::Damaged(
                    "an 8x8 quantisation matrix for 4x4 blocks",
                ));
            }
            if setup.scan_size != setup.block_size {
                return Err(Refused::Damaged("a scan order of another block size"));
            }
            if setup.transform_size == 8 && setup.block_size < 8 {
                return Err(Refused::Damaged("an 8x8 transform for 4x4 blocks"));
            }

            // the block code book: the frame's, or the band's own
            if bits.flag() {
                read_book_choice(bits, true, frame.block_books, &mut self.books)?;
            } else {
                self.books.book = frame.block_book.cloned();
            }

            setup.run_value_map = if bits.flag() {
                bits.take(3) as usize
            } else {
                8
            };
            setup.corrections.clear();
            if bits.flag() {
                let pairs = bits.take(8) as usize;
                if pairs > 61 {
                    return Err(Refused::Damaged("too many run / value corrections"));
                }
                for _ in 0..pairs * 2 {
                    setup.corrections.push(bits.take(8) as u8);
                }
            }
        }
        bits.align();
        if setup.scan.is_none() {
            return Err(Refused::Damaged("a band with no scan order yet"));
        }
        self.setup = setup;
        Ok(())
    }

    /// The band's part of a frame: its header, then tile after tile into buffer
    /// `frame.target`. `luma` is the luma band for the chroma bands, whose macroblocks
    /// take values from the luma band's.
    fn decode(
        &mut self,
        bits: &mut Bits,
        frame: &FrameValues,
        luma: Option<&Band>,
    ) -> Result<(), Refused> {
        self.current = Some(frame.target);
        self.prepare(frame.target);
        self.prepare(frame.reference);
        self.read_header(bits, frame)?;
        if self.setup.empty {
            return Err(Refused::Damaged("an empty band"));
        }

        // The run / value map with this band's corrections: swaps of two symbols' meanings.
        let mut map = RUN_VALUE_MAPS[self.setup.run_value_map].clone();
        for pair in self.setup.corrections.as_chunks::<2>().0 {
            let (a, b) = (pair[0], pair[1]);
            map.runs.swap(a as usize, b as usize);
            map.values.swap(a as usize, b as usize);
            if a == map.end_of_block || b == map.end_of_block {
                map.end_of_block ^= a ^ b;
            }
            if a == map.escape || b == map.escape {
                map.escape ^= a ^ b;
            }
        }

        let luma_macroblock_size =
            luma.map_or(self.setup.macroblock_size, |l| l.setup.macroblock_size);
        // The target buffer is taken out for the tiles' decoding, so that the reference
        // buffer can be read beside it.
        let mut target = std::mem::take(&mut self.buffers[frame.target]);
        let done = self.decode_tiles(bits, frame, luma, luma_macroblock_size, &map, &mut target);
        self.buffers[frame.target] = target;
        done?;
        bits.align();
        Ok(())
    }

    fn decode_tiles(
        &mut self,
        bits: &mut Bits,
        frame: &FrameValues,
        luma: Option<&Band>,
        luma_macroblock_size: usize,
        map: &RunValueMap,
        target: &mut [i16],
    ) -> Result<(), Refused> {
        let mut tiles = std::mem::take(&mut self.tiles);
        let mut data_start = bits.taken();
        let mut done = Ok(());
        for (number, tile) in tiles.iter_mut().enumerate() {
            let luma_macroblocks = match luma {
                None => None,
                Some(luma) => match luma.tiles.get(number) {
                    Some(luma_tile) => Some(luma_tile.macroblocks.as_slice()),
                    None => {
                        done = Err(Refused::Damaged("a tile without its luma tile"));
                        break;
                    }
                },
            };
            done = self.decode_tile(
                bits,
                frame,
                tile,
                luma_macroblocks,
                luma_macroblock_size,
                map,
                target,
                &mut data_start,
            );
            if done.is_err() {
                break;
            }
        }
        self.tiles = tiles;
        done
    }

    #[allow(clippy::too_many_arguments)] // one tile's decoding: the frame, band and tile parts
    fn decode_tile(
        &self,
        bits: &mut Bits,
        frame: &FrameValues,
        tile: &mut Tile,
        luma_macroblocks: Option<&[Macroblock]>,
        luma_macroblock_size: usize,
        map: &RunValueMap,
        target: &mut [i16],
        data_start: &mut usize,
    ) -> Result<(), Refused> {
        let size = self.setup.macroblock_size;
        if tile.macroblock_size != size || luma_macroblock_size < size {
            return Err(Refused::Damaged("macroblock sizes that do not match"));
        }
        // how much smaller this band's motion vectors are than the luma band's
        let motion_scale = (luma_macroblock_size >> 3) as u32 - (size >> 3) as u32;
        if bits.flag() {
            return self.fill_empty_tile(frame, tile, luma_macroblocks, motion_scale, target);
        }
        let data_size = read_tile_data_size(bits);
        if data_size == 0 {
            return Err(Refused::Damaged("a tile with no data"));
        }
        self.read_macroblocks(bits, frame, tile, luma_macroblocks, motion_scale)?;
        self.decode_blocks(bits, frame, tile, map, target)?;
        if (bits.taken() - *data_start) >> 3 != data_size {
            return Err(Refused::Damaged(
                "a tile of another size than its header says",
            ));
        }
        *data_start += data_size << 3;
        Ok(())
    }

    /// Whether a block of `size` a side, moved by a motion vector, lies inside the buffer.
    fn motion_inside(&self, block: &Macroblock, size: usize) -> bool {
        let half = i32::from(self.setup.half_sample);
        let (mx, my) = (i32::from(block.motion_x), i32::from(block.motion_y));
        let left = i32::from(block.x) + (mx >> half);
        let top = i32::from(block.y) + (my >> half);
        left >= 0
            && left + size as i32 + (mx & half) <= self.pitch as i32
            && top >= 0
            && top + size as i32 + (my & half) <= self.rows as i32
    }

    /// The values of every macroblock of a tile: kind, coded blocks, quantiser delta,
    /// motion vector; read, or taken from the luma band's macroblock at the same place.
    fn read_macroblocks(
        &self,
        bits: &mut Bits,
        frame: &FrameValues,
        tile: &mut Tile,
        luma_macroblocks: Option<&[Macroblock]>,
        motion_scale: u32,
    ) -> Result<(), Refused> {
        let setup = &self.setup;
        let size = setup.macroblock_size;
        if macroblocks_in(tile.width, tile.height, size) != tile.macroblocks.len() {
            return Err(Refused::Damaged("a tile's macroblock count"));
        }
        let blocks_each = if size == setup.block_size { 1 } else { 4 };
        let own_deltas = self.plane == 0 && frame.quant_deltas_sent;
        let book = frame.macroblock_book;
        let (mut motion_x, mut motion_y) = (0i32, 0i32);
        let mut number = 0;
        for y in (tile.y..tile.y + tile.height).step_by(size) {
            for x in (tile.x..tile.x + tile.width).step_by(size) {
                let luma = match luma_macroblocks {
                    None => None,
                    Some(all) => Some(
                        all.get(number)
                            .ok_or(Refused::Damaged("a macroblock without its luma macroblock"))?,
                    ),
                };
                let block = &mut tile.macroblocks[number];
                block.x = x as i16;
                block.y = y as i16;
                block.offset = (y * self.pitch + x) as u32;
                if bits.left() < 1 {
                    return Err(Refused::Damaged("a tile that ends in its macroblocks"));
                }
                if bits.flag() {
                    // An empty macroblock: nothing coded, taken from the picture before.
                    if frame.kind == INTRA {
                        return Err(Refused::Damaged("an empty macroblock in an intra frame"));
                    }
                    block.inter = true;
                    block.coded_blocks = 0;
                    block.quant_delta = 0;
                    if own_deltas {
                        block.quant_delta = signed(i32::from(bits.symbol(book) as i8)) as i8;
                    }
                    block.motion_x = 0;
                    block.motion_y = 0;
                    if let (true, Some(luma)) = (setup.inherit_motion, luma) {
                        block.motion_x = scaled_motion(luma.motion_x, motion_scale);
                        block.motion_y = scaled_motion(luma.motion_y, motion_scale);
                    }
                } else {
                    if setup.inherit_motion {
                        let Some(luma) = luma else {
                            return Err(Refused::Damaged("a luma band that inherits"));
                        };
                        block.inter = luma.inter;
                    } else if frame.kind == INTRA || frame.kind == INTRA_OTHER {
                        block.inter = false;
                    } else {
                        block.inter = bits.flag();
                    }
                    block.coded_blocks = bits.take(blocks_each) as u8;
                    block.quant_delta = 0;
                    if setup.inherit_quant_delta {
                        if let Some(luma) = luma {
                            block.quant_delta = luma.quant_delta;
                        }
                    } else if block.coded_blocks != 0 || own_deltas {
                        block.quant_delta = signed(i32::from(bits.symbol(book) as i8)) as i8;
                    }
                    if !block.inter {
                        block.motion_x = 0;
                        block.motion_y = 0;
                    } else if setup.inherit_motion {
                        if let Some(luma) = luma {
                            block.motion_x = scaled_motion(luma.motion_x, motion_scale);
                            block.motion_y = scaled_motion(luma.motion_y, motion_scale);
                        }
                    } else {
                        // the vector's change from the last one sent, down then across
                        motion_y = motion_y.wrapping_add(signed(bits.symbol(book)));
                        motion_x = motion_x.wrapping_add(signed(bits.symbol(book)));
                        block.motion_x = motion_x as i8;
                        block.motion_y = motion_y as i8;
                    }
                }

                if block.inter {
                    // the block the vector points at must lie in the buffer
                    let half = i64::from(setup.half_sample);
                    let (mx, my) = (i64::from(block.motion_x), i64::from(block.motion_y));
                    let (x, y, size, pitch) = (x as i64, y as i64, size as i64, self.pitch as i64);
                    let first = x + (mx >> half) + (y + (my >> half)) * pitch;
                    let last = x + ((mx + half) >> half) + size - 1
                        + (y + size - 1 + ((my + half) >> half)) * pitch;
                    if first < 0 || last > self.samples() as i64 - 1 {
                        return Err(Refused::Damaged("a motion vector out of the picture"));
                    }
                }
                number += 1;
            }
        }
        bits.align();
        Ok(())
    }

    /// A reference block put into (or added onto) the target at `offset`.
    #[allow(clippy::too_many_arguments)] // one block's place, vector and kind
    fn move_block(
        &self,
        frame: &FrameValues,
        target: &mut [i16],
        offset: i64,
        motion_x: i32,
        motion_y: i32,
        kind: Interpolation,
        add: bool,
    ) -> Result<(), Refused> {
        let size = self.setup.block_size;
        let pitch = self.pitch;
        let from = offset + i64::from(motion_y) * pitch as i64 + i64::from(motion_x);
        let room = self.samples() as i64 - (pitch * (size - 1) + size) as i64;
        if offset < 0 || from < 0 || room < offset || room - (kind.reach(pitch) as i64) < from {
            return Err(Refused::Damaged("a moved block out of the picture"));
        }
        let reference = &self.buffers[frame.reference];
        compensate(
            &mut target[offset as usize..],
            &reference[from as usize..],
            pitch,
            size,
            kind,
            add,
        );
        Ok(())
    }

    /// The blocks of every macroblock of a tile: coefficients read and transformed, or
    /// the block taken from the reference; then the reader is left at a whole byte.
    fn decode_blocks(
        &self,
        bits: &mut Bits,
        frame: &FrameValues,
        tile: &Tile,
        map: &RunValueMap,
        target: &mut [i16],
    ) -> Result<(), Refused> {
        let setup = &self.setup;
        let block_size = setup.block_size;
        let Some(transform) = setup.transform else {
            return Err(Refused::Damaged("a band with no transform yet"));
        };
        let blocks_each = if setup.macroblock_size == block_size {
            1
        } else {
            4
        };
        // intra blocks' first coefficients are differences from the block before
        let mut first_before = 0i32;
        let mut kind = Interpolation::None;
        let (mut motion_x, mut motion_y) = (0i32, 0i32);
        for block in &tile.macroblocks {
            let mut coded = block.coded_blocks;
            let mut offset = block.offset;
            let quant = (setup.quant as i32 + i32::from(block.quant_delta)).clamp(0, 31) as u32;
            if block.inter {
                motion_x = i32::from(block.motion_x);
                motion_y = i32::from(block.motion_y);
                if setup.half_sample {
                    kind = Interpolation::of(motion_x & 1 != 0, motion_y & 1 != 0);
                    motion_x >>= 1;
                    motion_y >>= 1;
                }
                if !self.motion_inside(block, setup.macroblock_size) {
                    return Err(Refused::Damaged("a motion vector out of the picture"));
                }
            }
            for number in 0..blocks_each {
                // the four blocks: left to right, top row then bottom row
                if number & 1 != 0 {
                    offset = offset.wrapping_add(block_size as u32);
                } else if number == 2 {
                    offset = offset
                        .wrapping_sub(block_size as u32)
                        .wrapping_add((block_size * self.pitch) as u32);
                }
                let at = i64::from(offset as i32);
                let room = self.samples() as i64 - at;
                if at < 0 || ((block_size - 1) * self.pitch + block_size) as i64 > room {
                    return Err(Refused::Damaged("a block out of the picture"));
                }
                let out = &mut target[at as usize..];
                if coded & 1 != 0 {
                    self.decode_coded_block(
                        bits,
                        map,
                        out,
                        !block.inter,
                        quant,
                        &mut first_before,
                    )?;
                    if block.inter {
                        self.move_block(frame, target, at, motion_x, motion_y, kind, true)?;
                    }
                } else if block.inter {
                    self.move_block(frame, target, at, motion_x, motion_y, kind, false)?;
                } else {
                    transform.lone_first_block(first_before, out, self.pitch, block_size);
                }
                coded >>= 1;
            }
        }
        bits.align();
        Ok(())
    }

    /// A coded block: runs and values read until the block's end, each value put at its
    /// place in scan order and multiplied back up, then the inverse transform into `out`.
    #[allow(clippy::too_many_arguments)] // one block's reading
    fn decode_coded_block(
        &self,
        bits: &mut Bits,
        map: &RunValueMap,
        out: &mut [i16],
        intra: bool,
        quant: u32,
        first_before: &mut i32,
    ) -> Result<(), Refused> {
        let setup = &self.setup;
        let size = setup.block_size;
        let (Some(scan), Some(transform), Some(book)) =
            (setup.scan, setup.transform, &self.books.book)
        else {
            return Err(Refused::Damaged(
                "a band with no scan order or code book yet",
            ));
        };
        let table = usize::from(QUANT_INDEX_TO_TABLE[setup.quant_matrix]);
        let matrix: Option<&[u16]> = match (size == 8, intra) {
            (true, true) => QUANT_8X8_INTRA.get(table).map(|m| &m[..]),
            (true, false) => QUANT_8X8_INTER.get(table).map(|m| &m[..]),
            (false, true) => QUANT_4X4_INTRA.get(table).map(|m| &m[..]),
            (false, false) => QUANT_4X4_INTER.get(table).map(|m| &m[..]),
        };
        let Some(matrix) = matrix else {
            return Err(Refused::Damaged(
                "a quantisation matrix of another block size",
            ));
        };
        let count = (size * size) as i32;
        if scan.len() != count as usize || transform.size() != size {
            return Err(Refused::Damaged("a scan order of another block size"));
        }

        let mut coefficients = [0i32; 64];
        let mut columns = [0u8; 8];
        let mut place = -1i32;
        let mut ended = false;
        while place <= count {
            let symbol = bits.symbol(book);
            if symbol == i32::from(map.end_of_block) {
                ended = true;
                break;
            }
            let (run, value);
            if symbol == i32::from(map.escape) {
                // run and value written out: the run, then the value's low and high parts
                run = bits.symbol(book) + 1;
                let low = bits.symbol(book) as u32;
                let high = bits.symbol(book) as u32;
                let joined = (high << 6) | low;
                value = ((joined >> 1) ^ (joined & 1).wrapping_neg()).wrapping_neg() as i32;
            } else {
                if !(0..256).contains(&symbol) {
                    return Err(Refused::Damaged("bits that are no block symbol"));
                }
                run = i32::from(map.runs[symbol as usize]);
                value = i32::from(map.values[symbol as usize]);
            }
            place += run;
            if place >= count || place < 0 {
                break;
            }
            let at = usize::from(scan[place as usize]);
            let step = (u32::from(matrix[at]) * quant) >> 9;
            let mut value = value;
            if step > 1 {
                // (a value of 0 rounds as a negative one does)
                let rounding = (((step ^ 1) - 1) >> 1) as i32;
                let towards = if value > 0 {
                    rounding
                } else {
                    rounding.wrapping_neg()
                };
                value = value.wrapping_mul(step as i32).wrapping_add(towards);
            }
            coefficients[at] = value;
            columns[at & (size - 1)] |= u8::from(value != 0);
        }
        if place < 0 || (place >= count && !ended) {
            return Err(Refused::Damaged("a block's coefficients"));
        }
        if intra && transform.predicts_first() {
            *first_before = first_before.wrapping_add(coefficients[0]);
            coefficients[0] = *first_before;
            columns[0] |= u8::from(*first_before != 0);
        }
        transform.inverse(&coefficients, out, self.pitch, &columns);
        Ok(())
    }

    /// A tile with nothing coded: its macroblocks keep (or take from the luma band) their
    /// values, and its samples come from the reference, moved where a vector is not 0.
    fn fill_empty_tile(
        &self,
        frame: &FrameValues,
        tile: &mut Tile,
        luma_macroblocks: Option<&[Macroblock]>,
        motion_scale: u32,
        target: &mut [i16],
    ) -> Result<(), Refused> {
        let setup = &self.setup;
        let size = setup.macroblock_size;
        if macroblocks_in(tile.width, tile.height, size) != tile.macroblocks.len() {
            return Err(Refused::Damaged("a tile's macroblock count"));
        }
        let mut any_motion = false;
        let mut number = 0;
        for y in (tile.y..tile.y + tile.height).step_by(size) {
            for x in (tile.x..tile.x + tile.width).step_by(size) {
                let block = &mut tile.macroblocks[number];
                block.x = x as i16;
                block.y = y as i16;
                block.offset = (y * self.pitch + x) as u32;
                block.inter = true;
                block.coded_blocks = 0;
                if self.plane == 0 {
                    block.quant_delta = setup.quant as i8;
                    block.motion_x = 0;
                    block.motion_y = 0;
                }
                if let Some(luma) = luma_macroblocks.and_then(|all| all.get(number)) {
                    if setup.inherit_quant_delta {
                        block.quant_delta = luma.quant_delta;
                    }
                    if setup.inherit_motion {
                        block.motion_x = scaled_motion(luma.motion_x, motion_scale);
                        block.motion_y = scaled_motion(luma.motion_y, motion_scale);
                        any_motion |= block.motion_x != 0 || block.motion_y != 0;
                        if !self.motion_inside(block, size) {
                            return Err(Refused::Damaged("a motion vector out of the picture"));
                        }
                    }
                }
                number += 1;
            }
        }

        if setup.inherit_motion && any_motion {
            let block_size = setup.block_size;
            let blocks_each = if size == block_size { 1 } else { 4 };
            for block in &tile.macroblocks {
                let (mut motion_x, mut motion_y) =
                    (i32::from(block.motion_x), i32::from(block.motion_y));
                let mut kind = Interpolation::None;
                if setup.half_sample {
                    kind = Interpolation::of(motion_x & 1 != 0, motion_y & 1 != 0);
                    motion_x >>= 1;
                    motion_y >>= 1;
                }
                for number in 0..blocks_each {
                    let across = (number & 1) * block_size;
                    let down = (number >> 1) * block_size * self.pitch;
                    let at = i64::from(block.offset) + (across + down) as i64;
                    self.move_block(frame, target, at, motion_x, motion_y, kind, false)?;
                }
            }
        } else {
            let reference = &self.buffers[frame.reference];
            for y in tile.y..tile.y + tile.height {
                let row = y * self.pitch + tile.x..y * self.pitch + tile.x + tile.width;
                let (Some(to), Some(from)) = (target.get_mut(row.clone()), reference.get(row))
                else {
                    return Err(Refused::Damaged("a tile out of the picture"));
                };
                to.copy_from_slice(from);
            }
        }
        Ok(())
    }

    /// The band's samples as a plane of the picture: 128 added back, kept within a byte.
    fn plane_bytes(&self) -> Result<Vec<u8>, Refused> {
        let samples = self
            .current
            .map(|number| &self.buffers[number])
            .filter(|samples| samples.len() == self.samples())
            .ok_or(Refused::Damaged("a plane never decoded"))?;
        let mut bytes = Vec::with_capacity(self.width * self.height);
        for row in samples.chunks_exact(self.pitch).take(self.height) {
            bytes.extend(
                row[..self.width]
                    .iter()
                    .map(|&s| (i32::from(s) + 128).clamp(0, 255) as u8),
            );
        }
        Ok(bytes)
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            macroblock_books: books(&MACROBLOCK_BOOKS),
            block_books: books(&BLOCK_BOOKS),
            frame_kind: 0,
            frame_kind_before: 0,
            layout: None,
            bands: Default::default(),
            target: 0,
            reference: 1,
            aside: 3,
            spoiled: [false; 4],
            macroblock_book: BookChoice::default(),
            block_book: BookChoice::default(),
            quant_deltas_sent: false,
            held: None,
        }
    }

    /// The buffers for a new layout: every band and tile made anew, nothing kept of the
    /// pictures before.
    fn lay_out(&mut self, layout: Layout) -> Result<(), Refused> {
        self.layout = None;
        self.bands = Default::default();
        if layout.width == 0 || layout.height == 0 {
            return Err(Refused::Damaged("a picture with no size"));
        }
        if layout.width > LARGEST_SIDE || layout.height > LARGEST_SIDE {
            return Err(Refused::NotImplemented("a picture larger than 2048 a side"));
        }
        for plane in 0..3 {
            let luma = plane == 0;
            let (width, height, tile_width, tile_height) = if luma {
                (
                    layout.width,
                    layout.height,
                    layout.tile_width,
                    layout.tile_height,
                )
            } else {
                (
                    layout.width.div_ceil(4),
                    layout.height.div_ceil(4),
                    layout.tile_width.div_ceil(4),
                    layout.tile_height.div_ceil(4),
                )
            };
            if tile_width == 0 || tile_height == 0 {
                return Err(Refused::Damaged("tiles with no size"));
            }
            // buffers hold whole macroblocks of the largest size the plane can have
            let whole = if luma { 16 } else { 8 };
            let macroblock_size = if luma { 16 } else { 4 };
            let mut tiles = Vec::new();
            for y in (0..height).step_by(tile_height) {
                for x in (0..width).step_by(tile_width) {
                    let (tile_w, tile_h) = (tile_width.min(width - x), tile_height.min(height - y));
                    let count = macroblocks_in(tile_w, tile_h, macroblock_size);
                    // each chroma tile goes with the luma tile of the same number
                    if !luma
                        && self.bands[0]
                            .tiles
                            .get(tiles.len())
                            .map(|t| t.macroblocks.len())
                            != Some(count)
                    {
                        return Err(Refused::Damaged(
                            "chroma tiles that do not match the luma tiles",
                        ));
                    }
                    tiles.push(Tile {
                        x,
                        y,
                        width: tile_w,
                        height: tile_h,
                        macroblock_size,
                        macroblocks: vec![Macroblock::default(); count],
                    });
                }
            }
            self.bands[plane] = Band {
                plane,
                width,
                height,
                pitch: width.next_multiple_of(whole),
                rows: height.next_multiple_of(whole),
                setup: BandSetup {
                    macroblock_size,
                    block_size: if luma { 8 } else { 4 },
                    ..BandSetup::default()
                },
                tiles,
                ..Band::default()
            };
        }
        self.layout = Some(layout);
        Ok(())
    }

    /// The picture header. True when picture data follows (false for the empty kinds).
    fn read_picture_header(&mut self, bits: &mut Bits) -> Result<bool, Refused> {
        if bits.take(18) != START_CODE {
            return Err(Refused::Damaged("no picture start code"));
        }
        self.frame_kind_before = self.frame_kind;
        self.frame_kind = bits.take(3);
        if self.frame_kind == 7 {
            return Err(Refused::Damaged("an unknown frame kind"));
        }
        if self.frame_kind == BIDIRECTIONAL {
            return Err(Refused::NotImplemented("a bidirectional frame"));
        }
        // a transparency plane, when the flag says there is one, is not decoded
        bits.skip(1);
        if bits.flag() {
            return Err(Refused::Damaged("the picture header's zero bit is set"));
        }
        // the frame's data size, when given
        if bits.flag() {
            bits.skip(24);
        }
        if self.frame_kind >= EMPTY_FIRST {
            return Ok(false);
        }
        // a locked film's key word: the pictures decode without it
        if bits.flag() {
            bits.skip(32);
        }

        let size_number = bits.take(3) as usize;
        let (height, width) = if size_number == 7 {
            (bits.take(16) as usize, bits.take(16) as usize)
        } else {
            (
                usize::from(COMMON_PICTURE_SIZES[size_number * 2 + 1]),
                usize::from(COMMON_PICTURE_SIZES[size_number * 2]),
            )
        };
        let tile_side = |whole: usize, factor: u32| {
            if factor == 15 {
                whole
            } else {
                (factor as usize + 1) << 5
            }
        };
        let (tile_height, tile_width) = if bits.flag() {
            (
                tile_side(height, bits.take(4)),
                tile_side(width, bits.take(4)),
            )
        } else {
            (height, width)
        };
        if bits.take(2) != 0 {
            return Err(Refused::Damaged(
                "a chroma format other than one in four by four",
            ));
        }
        // How each plane is split into bands: one band, or luma in four (not done here).
        let luma_bands = read_band_count(bits);
        let chroma_bands = if luma_bands == 0 {
            0
        } else {
            read_band_count(bits)
        };
        if width == 0 || height == 0 {
            return Err(Refused::Damaged("a picture with no size"));
        }
        if luma_bands == 4 && chroma_bands == 1 {
            return Err(Refused::NotImplemented("a picture in four luma bands"));
        }
        if luma_bands != 1 || chroma_bands != 1 {
            return Err(Refused::Damaged("an unknown split into bands"));
        }

        let layout = Layout {
            width,
            height,
            tile_width,
            tile_height,
        };
        if self.layout != Some(layout) {
            self.lay_out(layout)?;
        }

        // the frame's number, when given
        if bits.flag() {
            bits.skip(20);
        }
        // an estimate of the decoding time, when given
        if bits.flag() {
            bits.skip(8);
        }
        let sent = bits.flag();
        read_book_choice(
            bits,
            sent,
            &self.macroblock_books,
            &mut self.macroblock_book,
        )?;
        let sent = bits.flag();
        read_book_choice(bits, sent, &self.block_books, &mut self.block_book)?;
        // a run / value map for the frame, when given: each band names its own
        if bits.flag() {
            bits.skip(3);
        }
        bits.skip(1);
        self.quant_deltas_sent = bits.flag();
        // the frame's quantiser (each band has its own), then an unknown value and the
        // frame's checksum when given
        bits.skip(5);
        if bits.flag() {
            bits.skip(3);
        }
        if bits.flag() {
            bits.skip(16);
        }
        // header extensions, a byte each
        while bits.flag() {
            if bits.left() < 10 {
                return Err(Refused::Damaged("a picture header that does not end"));
            }
            bits.skip(8);
        }
        // a flag for bad blocks, of no use here
        bits.skip(1);
        bits.align();
        Ok(true)
    }

    /// Which buffers the frame decodes into and refers to. After a frame that others
    /// refer to, that frame's buffer becomes the reference; a frame that nothing refers
    /// to is decoded into the buffer kept aside.
    fn switch_buffers(&mut self) {
        let referred = |kind| matches!(kind, INTRA | INTRA_OTHER | INTER);
        if referred(self.frame_kind_before) {
            if !referred(self.frame_kind) {
                std::mem::swap(&mut self.reference, &mut self.aside);
            }
            std::mem::swap(&mut self.target, &mut self.reference);
        }
    }

    /// One packet decoded: the picture it gives, or None for a frame that shows nothing
    /// new (an empty frame; the film then stays on the picture before).
    pub fn decode(&mut self, packet: &[u8]) -> Result<Option<Picture>, Refused> {
        self.decode_frame(packet, true)
    }

    fn decode_frame(
        &mut self,
        packet: &[u8],
        may_hold_another: bool,
    ) -> Result<Option<Picture>, Refused> {
        let mut bits = Bits::new(packet);
        let has_data = self.read_picture_header(&mut bits)?;
        if self.frame_kind == EMPTY_LAST {
            return Ok(self.held.take());
        }
        if self.layout.is_none() {
            return Err(Refused::Damaged("no picture layout yet"));
        }
        self.switch_buffers();
        if has_data {
            self.spoiled[self.target] = true;
            let frame = FrameValues {
                kind: self.frame_kind,
                quant_deltas_sent: self.quant_deltas_sent,
                macroblock_book: self
                    .macroblock_book
                    .book
                    .as_deref()
                    .ok_or(Refused::Damaged("no macroblock code book"))?,
                block_book: self.block_book.book.as_ref(),
                block_books: &self.block_books,
                target: self.target,
                reference: self.reference,
            };
            let (luma, chroma) = self.bands.split_at_mut(1);
            luma[0].decode(&mut bits, &frame, None)?;
            for band in chroma {
                band.decode(&mut bits, &frame, Some(&luma[0]))?;
            }
            self.spoiled[self.target] = false;
        } else if self.bands.iter().any(|band| band.current.is_none()) {
            return Err(Refused::Damaged("an empty frame before any picture"));
        }
        if self.spoiled[self.target] {
            return Err(Refused::Damaged("an empty frame after a damaged one"));
        }
        if !has_data {
            return Ok(None);
        }
        // the planes are stored luma, red difference, blue difference
        let picture = Picture {
            width: self.bands[0].width,
            height: self.bands[0].height,
            chroma_width: self.bands[1].width,
            chroma_height: self.bands[1].height,
            planes: [
                self.bands[0].plane_bytes()?,
                self.bands[2].plane_bytes()?,
                self.bands[1].plane_bytes()?,
            ],
        };

        // An intra frame's packet can carry the next inter frame behind it (after the
        // encoder's version text): it is decoded now and shown at the next empty frame.
        if self.frame_kind == INTRA && may_hold_another {
            while bits.take(8) != 0 {
                // a text that does not end: the picture is good all the same
                if bits.left() < 8 {
                    return Ok(Some(picture));
                }
            }
            bits.skip(64 - (bits.taken() & 0x18));
            if bits.left() > 18 && bits.peek(21) == START_CODE_INTER {
                let rest = packet.get(bits.taken() >> 3..).unwrap_or(&[]);
                self.held = None;
                self.held = self.decode_frame(rest, false).ok().flatten();
            }
        }
        Ok(Some(picture))
    }

    /// The film player's way of calling a decoder: a packet in (None at the film's end),
    /// `each` called for every picture that comes out. A packet that is refused gives no
    /// picture and no error, as a player passes over a damaged frame; only `each` can
    /// fail the call.
    pub fn decode_each(
        &mut self,
        packet: Option<&[u8]>,
        each: &mut dyn FnMut(&Picture) -> Result<(), String>,
    ) -> Result<(), String> {
        // nothing is held back for the end: the held inter frame comes out with its own
        // empty frame's packet or not at all
        let Some(packet) = packet else {
            return Ok(());
        };
        match self.decode(packet) {
            Ok(Some(picture)) => each(&picture),
            Ok(None) | Err(_) => Ok(()),
        }
    }
}

/// How many bands a plane is split into: 1, 4, or 0 for a split that is not known.
fn read_band_count(bits: &mut Bits) -> u32 {
    match bits.take(2) {
        3 => 1,
        2 => {
            let mut whole = true;
            for _ in 0..4 {
                whole &= bits.take(2) == 3;
                if !whole {
                    break;
                }
            }
            if whole { 4 } else { 0 }
        }
        _ => 0,
    }
}
