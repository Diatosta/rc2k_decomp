//! An MPEG-1 video decoder (I, P and B pictures, 4:2:0), giving the same pictures byte for byte
//! as FFmpeg's `mpeg1video` decoder with its default inverse transform.
//!
//! `cut` divides an elementary stream into packets of one picture each (with the headers before
//! it), as FFmpeg's parser does; `VideoDecoder::decode` takes one packet and gives at most one
//! picture, in display order: a B picture at once, an I or P picture when the next I or P picture
//! has been decoded, and the last one for the empty packet that ends the stream. B pictures
//! before the first I picture's successor are passed over when their group of pictures is open,
//! P pictures until a sequence or group header or an I picture was seen.
//!
//! Where this differs from FFmpeg: a damaged slice is refused (`Err`, the picture left as far as
//! it came) where FFmpeg conceals it; a motion vector reaching outside the picture leaves the
//! macroblock as the buffer held it, as in FFmpeg, but what the buffer held is not FFmpeg's
//! (`VideoDecoder::vectors_outside` counts them); an MPEG-2 stream is refused.
//!
//! The decoding steps, their order and their arithmetic (the dequantisation's rounding, the
//! "simple" inverse transform, the half-sample averages, which macroblocks are passed over) are
//! ported from FFmpeg (libavcodec/mpeg12dec.c, mpeg12.c, mpegvideo_dec.c, mpegvideo_motion.c,
//! mpegvideo_parser.c, simple_idct_template.c, hpel_template.c; LGPL-2.1-or-later; copyright
//! (c) 2000, 2001 Fabrice Bellard, (c) 2001-2013 Michael Niedermayer).

use super::mpeg_bits::{Bits, Codes, FilmError, refuse};
use super::mpeg1_video_tables as tables;

const SEQUENCE_START: u8 = 0xB3;
const SEQUENCE_END: u8 = 0xB7;
const GROUP_START: u8 = 0xB8;
const EXTENSION_START: u8 = 0xB5;
const PICTURE_START: u8 = 0x00;
const SLICE_FIRST: u8 = 0x01;
const SLICE_LAST: u8 = 0xAF;

/// The stream cut into packets, each a range of `stream`: a packet ends before the first start
/// code that is no slice's after a slice, or after a sequence end code; what is left at the end
/// is the last packet.
pub fn cut(stream: &[u8]) -> Vec<std::ops::Range<usize>> {
    let mut packets = Vec::new();
    let mut start = 0;
    // The last four bytes, and how far into a picture the scan is: 0 before its first slice, 4
    // after it; odd while an extension's first bytes are looked at.
    let mut state = u32::MAX;
    let mut found = 0u32;
    let mut i = 0;
    while i < stream.len() {
        let byte = stream[i];
        if found & 1 == 1 {
            if state == 0x1B5 && byte & 0xF0 != 0x80 {
                found -= 1;
            } else if state == 0x1B5 + 2 {
                found = if byte & 3 == 3 { 0 } else { (found + 1) & 3 };
            }
            state = state.wrapping_add(1);
            i += 1;
            continue;
        }
        state = state << 8 | byte as u32;
        if state & 0xFFFF_FF00 != 0x100 {
            i += 1;
            continue;
        }
        let is_slice = (SLICE_FIRST..=SLICE_LAST).contains(&byte);
        if found == 0 && is_slice {
            // A picture's first slice; the byte after its start code is not looked at.
            found = 4;
            i += 2;
            continue;
        }
        if byte == SEQUENCE_END {
            packets.push(start..i + 1);
            start = i + 1;
            (state, found) = (u32::MAX, 0);
            i += 1;
            continue;
        }
        if found == 2 && byte == SEQUENCE_START {
            found = 0;
        }
        if found < 4 && byte == EXTENSION_START {
            found += 1;
        }
        if found == 4 && !is_slice {
            // The packet ends before this start code, which is read again as the next one's.
            let end = i - 3;
            packets.push(start..end);
            start = end;
            (state, found) = (u32::MAX, 0);
            i = end;
            continue;
        }
        i += 1;
    }
    if start < stream.len() {
        packets.push(start..stream.len());
    }
    packets
}

/// A decoded picture: three planes (luma, then the two chroma planes at half the width and
/// height), each of rows `stride` bytes apart of which the first `width` (chroma: half, rounded
/// up) are the picture's.
#[derive(Default, Clone)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    planes: [Vec<u8>; 3],
    strides: [usize; 3],
}

impl Picture {
    fn new(width: usize, height: usize, blocks_wide: usize, blocks_high: usize) -> Self {
        Self {
            width,
            height,
            planes: [
                vec![0; blocks_wide * blocks_high * 256],
                vec![0; blocks_wide * blocks_high * 64],
                vec![0; blocks_wide * blocks_high * 64],
            ],
            strides: [blocks_wide * 16, blocks_wide * 8, blocks_wide * 8],
        }
    }

    /// Plane `p`'s bytes (0: luma, 1 and 2: chroma) and the bytes from one row to the next.
    pub fn plane(&self, p: usize) -> (&[u8], usize) {
        (&self.planes[p], self.strides[p])
    }

    /// The width and height of plane `p`'s part that is picture.
    pub fn plane_size(&self, p: usize) -> (usize, usize) {
        if p == 0 {
            (self.width, self.height)
        } else {
            (self.width.div_ceil(2), self.height.div_ceil(2))
        }
    }

    /// Plane `p` without the bytes past the picture's edge, row after row.
    pub fn packed_plane(&self, p: usize) -> Vec<u8> {
        let (width, height) = self.plane_size(p);
        let mut packed = Vec::with_capacity(width * height);
        for row in self.planes[p].chunks(self.strides[p].max(1)).take(height) {
            packed.extend_from_slice(&row[..width]);
        }
        packed
    }

    fn fill(&mut self, value: u8) {
        for plane in &mut self.planes {
            plane.fill(value);
        }
    }
}

/// What a sequence header says.
#[derive(Clone)]
struct Sequence {
    width: usize,
    height: usize,
    blocks_wide: usize,
    blocks_high: usize,
    frame_rate_code: u8,
    /// The quantiser matrices by a coefficient's place in its block (raster).
    intra_matrix: [u16; 64],
    inter_matrix: [u16; 64],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PictureKind {
    Intra,
    Predicted,
    Bidirectional,
}

// A macroblock type's parts.
const TYPE_INTRA: u8 = 0x01;
const TYPE_PATTERN: u8 = 0x02;
const TYPE_BACKWARD: u8 = 0x04;
const TYPE_FORWARD: u8 = 0x08;
const TYPE_QUANT: u8 = 0x10;
/// The macroblock types a P picture's codes stand for (`tables::P_TYPE_CODES`).
const P_TYPES: [u8; 7] = [0x01, 0x02, 0x08, 0x0A, 0x11, 0x12, 0x1A];
/// The macroblock types a B picture's codes stand for (`tables::B_TYPE_CODES`).
const B_TYPES: [u8; 11] = [
    0x01, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E, 0x11, 0x16, 0x1A, 0x1E,
];

const FORWARD: u8 = 1;
const BACKWARD: u8 = 2;

/// The picture buffers: two for the pictures others are predicted from, one for B pictures, one
/// grey for a missing reference.
const B_BUFFER: usize = 2;
const GREY_BUFFER: usize = 3;

struct CodeTables {
    dc_luma: Codes,
    dc_chroma: Codes,
    coefficient: Codes,
    address_increment: Codes,
    coded_pattern: Codes,
    motion: Codes,
    p_type: Codes,
    b_type: Codes,
}

/// The state that runs from one macroblock to the next.
#[derive(Default)]
struct Running {
    /// Twice the quantiser scale.
    quantiser: i32,
    /// The DC predictions: luma, the two chroma planes.
    last_dc: [i32; 3],
    /// The vector predictions, forward and backward, as coded (not doubled for whole samples).
    last_vector: [[i32; 2]; 2],
    /// The vectors used, in half samples.
    vector: [[i32; 2]; 2],
    directions: u8,
    intra: bool,
}

/// The decoder.
pub struct VideoDecoder {
    codes: CodeTables,
    sequence: Option<Sequence>,
    /// The size the buffers were made for.
    buffers_for: (usize, usize),
    pictures: [Picture; 4],
    /// The pictures predicted from: the earlier and the later one in display order.
    last: Option<usize>,
    next: Option<usize>,
    /// The picture being decoded, until it or the one it releases has been given out.
    current: Option<usize>,
    /// A picture header was read and no slice of it decoded yet.
    awaiting_first_slice: bool,
    /// A sequence or group header or an I picture was seen: P pictures are decoded.
    synchronised: bool,
    closed_group: bool,
    kind: Option<PictureKind>,
    /// Whether the forward and backward vectors count whole samples, and their sizes' codes.
    whole_samples: [bool; 2],
    vector_code: [u32; 2],
    running: Running,
    /// Which macroblocks of the picture being decoded are intra.
    intra_map: Vec<bool>,
    blocks: [[i16; 64]; 6],
    /// Macroblocks whose vector reached outside the picture, left as the buffer held them.
    pub vectors_outside: u64,
}

impl Default for VideoDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoDecoder {
    pub fn new() -> Self {
        Self {
            codes: CodeTables {
                dc_luma: Codes::new(&tables::DC_LUMA_CODES, 9),
                dc_chroma: Codes::new(&tables::DC_CHROMA_CODES, 9),
                coefficient: Codes::new(&tables::COEFFICIENT_CODES, 9),
                address_increment: Codes::new(&tables::ADDRESS_INCREMENT_CODES, 9),
                coded_pattern: Codes::new(&tables::CODED_PATTERN_CODES, 9),
                motion: Codes::new(&tables::MOTION_CODES, 8),
                p_type: Codes::new(&tables::P_TYPE_CODES, 6),
                b_type: Codes::new(&tables::B_TYPE_CODES, 6),
            },
            sequence: None,
            buffers_for: (0, 0),
            pictures: Default::default(),
            last: None,
            next: None,
            current: None,
            awaiting_first_slice: false,
            synchronised: false,
            closed_group: false,
            kind: None,
            whole_samples: [false; 2],
            vector_code: [1; 2],
            running: Running::default(),
            intra_map: Vec::new(),
            blocks: [[0; 64]; 6],
            vectors_outside: 0,
        }
    }

    /// The pictures a second the sequence header names, as a fraction; None before one was read
    /// or for a code that names none.
    pub fn frame_rate(&self) -> Option<(u32, u32)> {
        match self.sequence.as_ref()?.frame_rate_code {
            1 => Some((24000, 1001)),
            2 => Some((24, 1)),
            3 => Some((25, 1)),
            4 => Some((30000, 1001)),
            5 => Some((30, 1)),
            6 => Some((50, 1)),
            7 => Some((60000, 1001)),
            8 => Some((60, 1)),
            _ => None,
        }
    }

    /// One packet decoded (`cut`'s; an empty one ends the stream): the picture that comes out
    /// of it, if one does.
    pub fn decode(&mut self, packet: &[u8]) -> Result<Option<&Picture>, FilmError> {
        if packet.is_empty() || packet == [0, 0, 1, SEQUENCE_END] {
            // The end: the last I or P picture is still held back.
            return Ok(self.next.take().map(|held| &self.pictures[held]));
        }
        let shown = self.decode_headers_and_slices(packet);
        // The picture's kind does not outlast its packet.
        self.kind = None;
        let shown = shown?;
        if shown.is_some() {
            self.current = None;
        }
        Ok(shown.map(|picture| &self.pictures[picture]))
    }

    /// The packet's start codes in turn; the number of the picture to show, if any.
    fn decode_headers_and_slices(&mut self, packet: &[u8]) -> Result<Option<usize>, FilmError> {
        // The start code before this one, as far as it matters: none, a picture's or a slice's.
        #[derive(PartialEq)]
        enum Before {
            Nothing,
            Picture,
            Slice,
        }
        let mut before = Before::Nothing;
        let mut passed_over = false;
        let mut picture_seen = false;
        let mut at = 0;
        loop {
            let Some(code_at) = next_start_code(packet, at) else {
                return Ok(if passed_over {
                    None
                } else {
                    self.picture_to_show()
                });
            };
            let code = packet[code_at];
            at = code_at + 1;
            let rest = &packet[at..];
            match code {
                SEQUENCE_START => {
                    if before == Before::Nothing {
                        self.read_sequence_header(rest)?;
                        self.synchronised = true;
                    }
                }
                PICTURE_START => {
                    if picture_seen {
                        continue;
                    }
                    picture_seen = true;
                    if self.sequence.is_none() {
                        return refuse("a picture before any sequence header");
                    }
                    if before != Before::Picture {
                        self.make_buffers();
                        self.read_picture_header(rest);
                        self.awaiting_first_slice = true;
                        before = Before::Picture;
                    }
                }
                EXTENSION_START => {
                    if rest.first().is_some_and(|b| b >> 4 == 1) && before == Before::Nothing {
                        return refuse("an MPEG-2 sequence, which is not decoded");
                    }
                }
                GROUP_START => {
                    if before == Before::Nothing {
                        // 25 bits of time code, then whether the group is closed.
                        let mut bits = Bits::new(rest);
                        bits.skip(25);
                        self.closed_group = bits.take_flag();
                        self.synchronised = true;
                    }
                }
                SLICE_FIRST..=SLICE_LAST if before != Before::Nothing => {
                    before = Before::Slice;
                    let row = (code - SLICE_FIRST) as usize;
                    let Some(sequence) = &self.sequence else {
                        continue;
                    };
                    if rest.len() < 2 {
                        return refuse("a slice of less than two bytes");
                    }
                    if row >= sequence.blocks_high {
                        return refuse(format!("a slice below the picture (row {row})"));
                    }
                    if self.last.is_none()
                        && self.kind == Some(PictureKind::Bidirectional)
                        && !self.closed_group
                    {
                        // A B picture of an open group without the picture before it.
                        passed_over = true;
                        continue;
                    }
                    if self.kind == Some(PictureKind::Intra) {
                        self.synchronised = true;
                    }
                    if self.next.is_none()
                        && self.kind == Some(PictureKind::Predicted)
                        && !self.synchronised
                    {
                        passed_over = true;
                        continue;
                    }
                    let Some(kind) = self.kind else {
                        // A slice without a readable picture header.
                        continue;
                    };
                    if self.awaiting_first_slice {
                        // A packet far too small for its picture is no picture.
                        if sequence.blocks_wide * sequence.blocks_high * 11 / (33 * 2 * 8)
                            > packet.len()
                        {
                            return refuse("a packet too small for a picture");
                        }
                        passed_over = false;
                        self.awaiting_first_slice = false;
                        self.start_picture(kind);
                    }
                    let Some(current) = self.current else {
                        return refuse("a slice without a picture to decode it into");
                    };
                    let mut picture = std::mem::take(&mut self.pictures[current]);
                    let read = self.decode_slice(kind, row, rest, &mut picture);
                    self.pictures[current] = picture;
                    // On after the slice's last byte but one (the rest may begin a start code).
                    at += read?.saturating_sub(1) / 8;
                }
                _ => {}
            }
        }
    }

    /// At a packet's end: the picture that is shown, when one was decoded in it or is waiting.
    fn picture_to_show(&mut self) -> Option<usize> {
        let current = self.current?;
        if self.awaiting_first_slice {
            return None;
        }
        if current == B_BUFFER {
            Some(current)
        } else {
            // An I or P picture releases the one before it.
            self.last.filter(|&last| last != GREY_BUFFER)
        }
    }

    fn read_sequence_header(&mut self, bytes: &[u8]) -> Result<(), FilmError> {
        let mut bits = Bits::new(bytes);
        let width = bits.take(12) as usize;
        let height = bits.take(12) as usize;
        if width == 0 || height == 0 {
            return refuse("a sequence header with no width or height");
        }
        bits.skip(4); // the aspect ratio
        let frame_rate_code = bits.take(4) as u8;
        bits.skip(18); // the bit rate
        if !bits.take_flag() {
            return refuse("a sequence header without its marker bit");
        }
        bits.skip(11); // the buffer size, the constrained flag
        let mut intra_matrix = tables::DEFAULT_INTRA_MATRIX;
        let mut inter_matrix = [16u16; 64];
        for (matrix, is_intra) in [(&mut intra_matrix, true), (&mut inter_matrix, false)] {
            if !bits.take_flag() {
                continue;
            }
            for (i, &place) in tables::ZIGZAG.iter().enumerate() {
                let value = bits.take(8) as u16;
                if value == 0 {
                    return refuse("a quantiser matrix with a zero");
                }
                // The intra DC quantiser is always 8.
                matrix[place as usize] = if is_intra && i == 0 { 8 } else { value };
            }
        }
        if bits.peek(23) != 0 || bits.overrun() {
            return refuse("a damaged sequence header");
        }
        self.sequence = Some(Sequence {
            width,
            height,
            blocks_wide: width.div_ceil(16),
            blocks_high: height.div_ceil(16),
            frame_rate_code,
            intra_matrix,
            inter_matrix,
        });
        Ok(())
    }

    /// The picture buffers for the sequence's size; a new size drops the pictures held.
    fn make_buffers(&mut self) {
        let Some(sequence) = &self.sequence else {
            return;
        };
        let size = (sequence.width, sequence.height);
        if self.buffers_for == size {
            return;
        }
        self.buffers_for = size;
        for picture in &mut self.pictures {
            *picture = Picture::new(
                sequence.width,
                sequence.height,
                sequence.blocks_wide,
                sequence.blocks_high,
            );
        }
        self.pictures[GREY_BUFFER].fill(0x80);
        self.intra_map = vec![false; sequence.blocks_wide * sequence.blocks_high];
        (self.last, self.next, self.current) = (None, None, None);
    }

    /// A picture header: its kind (none for one that is not I, P or B) and its vectors' sizes.
    fn read_picture_header(&mut self, bytes: &[u8]) {
        let mut bits = Bits::new(bytes);
        bits.skip(10); // the temporal reference
        self.kind = match bits.take(3) {
            1 => Some(PictureKind::Intra),
            2 => Some(PictureKind::Predicted),
            3 => Some(PictureKind::Bidirectional),
            _ => None,
        };
        bits.skip(16); // the buffer delay
        let directions = match self.kind {
            Some(PictureKind::Predicted) => 1,
            Some(PictureKind::Bidirectional) => 2,
            _ => 0,
        };
        for direction in 0..directions {
            self.whole_samples[direction] = bits.take_flag();
            // A size code of 0 is not allowed and read as 1.
            self.vector_code[direction] = bits.take(3).max(1);
        }
    }

    /// Before a picture's first slice: its buffer chosen, the references moved on.
    fn start_picture(&mut self, kind: PictureKind) {
        if kind == PictureKind::Bidirectional {
            self.current = Some(B_BUFFER);
        } else {
            let into = match self.next {
                Some(0) => 1,
                _ => 0,
            };
            self.last = self.next;
            self.next = Some(into);
            self.current = Some(into);
        }
        if kind != PictureKind::Intra && self.last.is_none() {
            self.last = Some(GREY_BUFFER);
        }
        self.intra_map.fill(false);
    }

    /// One slice from its first byte after the start code, beginning in macroblock row `row`;
    /// the bits it took.
    fn decode_slice(
        &mut self,
        kind: PictureKind,
        row: usize,
        bytes: &[u8],
        picture: &mut Picture,
    ) -> Result<usize, FilmError> {
        let Some(sequence) = self.sequence.clone() else {
            return refuse("a slice before any sequence header");
        };
        let (blocks_wide, blocks_high) = (sequence.blocks_wide, sequence.blocks_high);
        let mut bits = Bits::new(bytes);
        self.running.last_dc = [128; 3];
        self.running.last_vector = [[0; 2]; 2];
        self.running.quantiser = (bits.take(5) as i32) << 1;
        if self.running.quantiser == 0 {
            return refuse("a slice with quantiser 0");
        }
        // extra information: a set bit, then a byte, until a clear bit
        while bits.take_flag() {
            if bits.left() <= 0 {
                return refuse("a slice's extra information past its end");
            }
            bits.skip(8);
        }
        let mut column = 0usize;
        while bits.left() > 0 {
            match self.codes.address_increment.read(&mut bits) {
                None => return refuse("a damaged first macroblock address"),
                Some(33) => column += 33,
                Some(code) if code > 33 => {}
                Some(code) => {
                    column += code;
                    break;
                }
            }
        }
        if column >= blocks_wide {
            return refuse("a slice starting past the picture's right edge");
        }
        let mut row = row;
        // Macroblocks still to pass over; -1: the next address increment is to be read.
        let mut skip_run = 0i32;
        loop {
            let coded = if skip_run != 0 {
                skip_run -= 1;
                if kind == PictureKind::Bidirectional {
                    let before = if column > 0 {
                        self.intra_map[row * blocks_wide + column - 1]
                    } else {
                        row > 0 && self.intra_map[row * blocks_wide - 1]
                    };
                    if before {
                        return refuse("a macroblock passed over after an intra one");
                    }
                }
                [false; 6]
            } else {
                skip_run = -1;
                self.decode_macroblock(kind, &mut bits, &sequence)?
            };
            self.intra_map[row * blocks_wide + column] = self.running.intra;
            self.reconstruct(&sequence, picture, column, row, coded)?;
            if bits.left() < -64 {
                return refuse("a slice read far past its end");
            }
            column += 1;
            if column >= blocks_wide {
                column = 0;
                row += 1;
                if row >= blocks_high {
                    let left = bits.left();
                    if left < 0 || (left > 0 && bits.peek(left.min(23) as u32) != 0) {
                        return refuse("a slice that does not end with the picture");
                    }
                    break;
                }
            }
            if skip_run != -1 {
                continue;
            }
            skip_run = 0;
            loop {
                match self.codes.address_increment.read(&mut bits) {
                    None => return refuse("a damaged macroblock address"),
                    Some(33) => skip_run += 33,
                    Some(34) => {}
                    Some(35) => {
                        // the zeros before the next start code
                        if skip_run != 0 || bits.peek(15) != 0 {
                            return refuse("a slice ending inside a macroblock address");
                        }
                        if bits.overrun() {
                            return refuse("a slice read past its end");
                        }
                        return Ok(bits.position());
                    }
                    Some(code) => {
                        skip_run += code as i32;
                        break;
                    }
                }
            }
            if skip_run != 0 {
                let running = &mut self.running;
                running.intra = false;
                match kind {
                    PictureKind::Intra => {
                        return refuse("a macroblock passed over in an I picture");
                    }
                    PictureKind::Predicted => {
                        // no vector
                        running.directions = FORWARD;
                        running.vector[0] = [0; 2];
                        running.last_vector[0] = [0; 2];
                    }
                    PictureKind::Bidirectional => {
                        // the vectors and directions of the macroblock before
                        running.vector = running.last_vector;
                    }
                }
            }
        }
        if bits.overrun() {
            return refuse("a slice read past its end");
        }
        Ok(bits.position())
    }

    /// A coded macroblock's type, vectors and coefficients; which of its six blocks hold
    /// coefficients.
    fn decode_macroblock(
        &mut self,
        kind: PictureKind,
        bits: &mut Bits,
        sequence: &Sequence,
    ) -> Result<[bool; 6], FilmError> {
        let kind_of = match kind {
            PictureKind::Intra => {
                if bits.take_flag() {
                    TYPE_INTRA
                } else if bits.take_flag() {
                    TYPE_INTRA | TYPE_QUANT
                } else {
                    return refuse("a macroblock type no I picture has");
                }
            }
            PictureKind::Predicted => match self.codes.p_type.read(bits) {
                Some(code) => P_TYPES[code],
                None => return refuse("a macroblock type no P picture has"),
            },
            PictureKind::Bidirectional => match self.codes.b_type.read(bits) {
                Some(code) => B_TYPES[code],
                None => return refuse("a macroblock type no B picture has"),
            },
        };
        let running = &mut self.running;
        if kind_of & TYPE_INTRA != 0 {
            self.blocks.fill([0; 64]);
            if kind_of & TYPE_QUANT != 0 {
                running.quantiser = (bits.take(5) as i32) << 1;
            }
            running.last_vector = [[0; 2]; 2];
            running.intra = true;
            for (number, block) in self.blocks.iter_mut().enumerate() {
                let component = number.saturating_sub(3);
                let codes = if component == 0 {
                    &self.codes.dc_luma
                } else {
                    &self.codes.dc_chroma
                };
                let Some(size) = codes.read(bits) else {
                    return refuse("a damaged DC coefficient");
                };
                let difference = if size == 0 {
                    0
                } else {
                    let value = bits.take(size as u32) as i32;
                    // A value whose first bit is clear is negative.
                    if value >> (size - 1) != 0 {
                        value
                    } else {
                        value - (1 << size) + 1
                    }
                };
                running.last_dc[component] += difference;
                block[0] =
                    running.last_dc[component].wrapping_mul(sequence.intra_matrix[0] as i32) as i16;
                read_coefficients(
                    bits,
                    &self.codes.coefficient,
                    block,
                    &sequence.intra_matrix,
                    running.quantiser,
                    true,
                )?;
            }
            return Ok([true; 6]);
        }
        running.intra = false;
        if kind_of & (TYPE_FORWARD | TYPE_BACKWARD) == 0 {
            // coefficients over the same place of the picture before
            if kind_of & TYPE_QUANT != 0 {
                running.quantiser = (bits.take(5) as i32) << 1;
            }
            running.directions = FORWARD;
            running.last_vector[0] = [0; 2];
            running.vector[0] = [0; 2];
        } else {
            if kind_of & TYPE_QUANT != 0 {
                running.quantiser = (bits.take(5) as i32) << 1;
            }
            running.directions = (if kind_of & TYPE_FORWARD != 0 {
                FORWARD
            } else {
                0
            }) | (if kind_of & TYPE_BACKWARD != 0 {
                BACKWARD
            } else {
                0
            });
            for direction in 0..2 {
                if running.directions & (1 << direction) == 0 {
                    continue;
                }
                for axis in 0..2 {
                    let value = read_vector(
                        bits,
                        &self.codes.motion,
                        self.vector_code[direction],
                        running.last_vector[direction][axis],
                    )?;
                    running.last_vector[direction][axis] = value;
                    running.vector[direction][axis] = if self.whole_samples[direction] {
                        value * 2
                    } else {
                        value
                    };
                }
            }
        }
        let mut coded = [false; 6];
        if kind_of & TYPE_PATTERN != 0 {
            self.blocks.fill([0; 64]);
            let pattern = match self.codes.coded_pattern.read(bits) {
                Some(pattern) if pattern > 0 => pattern,
                _ => return refuse("a damaged coded block pattern"),
            };
            for (number, block) in self.blocks.iter_mut().enumerate() {
                if pattern & (32 >> number) != 0 {
                    read_coefficients(
                        bits,
                        &self.codes.coefficient,
                        block,
                        &sequence.inter_matrix,
                        running.quantiser,
                        false,
                    )?;
                    coded[number] = true;
                }
            }
        }
        Ok(coded)
    }

    /// The macroblock at (`column`, `row`) put into the picture: intra blocks as they are,
    /// others predicted from the reference pictures with their coded blocks added.
    fn reconstruct(
        &mut self,
        sequence: &Sequence,
        picture: &mut Picture,
        column: usize,
        row: usize,
        coded: [bool; 6],
    ) -> Result<(), FilmError> {
        let (luma_stride, chroma_stride) = (picture.strides[0], picture.strides[1]);
        let luma_at = row * 16 * luma_stride + column * 16;
        let chroma_at = row * 8 * chroma_stride + column * 8;
        let block_places = [
            (0, luma_at, luma_stride),
            (0, luma_at + 8, luma_stride),
            (0, luma_at + 8 * luma_stride, luma_stride),
            (0, luma_at + 8 * luma_stride + 8, luma_stride),
            (1, chroma_at, chroma_stride),
            (2, chroma_at, chroma_stride),
        ];
        if self.running.intra {
            for (block, (plane, at, stride)) in self.blocks.iter_mut().zip(block_places) {
                transform(block, &mut picture.planes[plane], at, stride, false)?;
            }
            return Ok(());
        }
        self.running.last_dc = [128; 3];
        let mut average = false;
        for (direction, reference) in [(0, self.last), (1, self.next)] {
            if self.running.directions & (1 << direction) == 0 {
                continue;
            }
            let reference = &self.pictures[reference.unwrap_or(GREY_BUFFER)];
            let [x, y] = self.running.vector[direction];
            let from_x = column as i32 * 16 + (x >> 1);
            let from_y = row as i32 * 16 + (y >> 1);
            let most_x = (sequence.blocks_wide as i32 * 16 - (x & 1) - 15).max(0);
            let most_y = (sequence.blocks_high as i32 * 16 - (y & 1) - 15).max(0);
            if from_x < 0 || from_x >= most_x || from_y < 0 || from_y >= most_y {
                // A vector reaching outside the picture: this prediction is left out.
                self.vectors_outside += 1;
                average = true;
                continue;
            }
            predict(
                &mut picture.planes[0],
                luma_at,
                &reference.planes[0],
                from_y as usize * luma_stride + from_x as usize,
                luma_stride,
                16,
                (x & 1) != 0,
                (y & 1) != 0,
                average,
            )?;
            // The chroma vector is half the luma one, towards zero.
            let (chroma_x, chroma_y) = (x / 2, y / 2);
            let from = (row as i32 * 8 + (chroma_y >> 1)) as i64 * chroma_stride as i64
                + (column as i32 * 8 + (chroma_x >> 1)) as i64;
            let Ok(from) = usize::try_from(from) else {
                return refuse("a chroma vector reaching before the picture");
            };
            for plane in 1..3 {
                predict(
                    &mut picture.planes[plane],
                    chroma_at,
                    &reference.planes[plane],
                    from,
                    chroma_stride,
                    8,
                    (chroma_x & 1) != 0,
                    (chroma_y & 1) != 0,
                    average,
                )?;
            }
            average = true;
        }
        for ((block, (plane, at, stride)), coded) in
            self.blocks.iter_mut().zip(block_places).zip(coded)
        {
            if coded {
                transform(block, &mut picture.planes[plane], at, stride, true)?;
            }
        }
        Ok(())
    }
}

/// The place of the byte after the next `00 00 01` at or after `from`.
fn next_start_code(bytes: &[u8], from: usize) -> Option<usize> {
    let rest = bytes.get(from..)?;
    let found = rest.windows(3).position(|w| w == [0, 0, 1])?;
    let code_at = from + found + 3;
    (code_at < bytes.len()).then_some(code_at)
}

/// A vector component: the prediction `before` and the coded difference, wrapped into the
/// range the size code `size_code` gives.
fn read_vector(
    bits: &mut Bits,
    codes: &Codes,
    size_code: u32,
    before: i32,
) -> Result<i32, FilmError> {
    let Some(code) = codes.read(bits) else {
        return refuse("a damaged motion vector");
    };
    if code == 0 {
        return Ok(before);
    }
    let negative = bits.take_flag();
    let shift = size_code - 1;
    let mut value = code as i32;
    if shift != 0 {
        value = (value - 1) << shift;
        value |= bits.take(shift) as i32;
        value += 1;
    }
    if negative {
        value = -value;
    }
    value += before;
    // kept to its 5 + shift bits, signed
    let unused = 32 - (5 + shift);
    Ok((value << unused) >> unused)
}

/// A block's coefficients after its first (an intra block's DC is read by the caller),
/// dequantised into `block`: each `level` becomes `level * quantiser * matrix / 16` for an
/// intra block and `(2 level + 1) * quantiser * matrix / 32` otherwise (`quantiser` is twice the
/// scale), made odd towards zero.
fn read_coefficients(
    bits: &mut Bits,
    codes: &Codes,
    block: &mut [i16; 64],
    matrix: &[u16; 64],
    quantiser: i32,
    intra: bool,
) -> Result<(), FilmError> {
    const ESCAPE: usize = tables::COEFFICIENT_LEVEL.len();
    let dequantised = |level: i32, place: usize| -> i32 {
        let scaled = if intra {
            (level * quantiser * matrix[place] as i32) >> 4
        } else {
            ((level * 2 + 1) * quantiser * matrix[place] as i32) >> 5
        };
        (scaled - 1) | 1
    };
    // The place in coding order of the last coefficient read.
    let mut i: i32 = if intra { 0 } else { -1 };
    let mut ended = false;
    if !intra && bits.peek(1) == 1 {
        // A first coefficient of level 1 has a code of its own: a set bit and the sign.
        let level = dequantised(1, 0);
        block[0] = if bits.peek(2) & 1 != 0 { -level } else { level } as i16;
        i = 0;
        bits.skip(2);
        ended = bits.peek(2) == 0b10;
    } else if intra {
        ended = bits.peek(2) == 0b10;
    }
    while !ended {
        let Some(code) = codes.read(bits) else {
            return refuse("a damaged coefficient");
        };
        let (run, level, negative) = if code < ESCAPE {
            let negative = bits.take_flag();
            (
                tables::COEFFICIENT_RUN[code] as i32 + 1,
                tables::COEFFICIENT_LEVEL[code] as i32,
                negative,
            )
        } else if code == ESCAPE {
            let run = bits.take(6) as i32 + 1;
            let mut level = bits.take(8) as i8 as i32;
            if level == -128 {
                level = bits.take(8) as i32 - 256;
            } else if level == 0 {
                level = bits.take(8) as i32;
            }
            (run, level.abs(), level < 0)
        } else {
            return refuse("a block's end where a coefficient must be");
        };
        i += run;
        if i > 63 {
            return refuse("more than 64 coefficients in a block");
        }
        let place = tables::ZIGZAG[i as usize] as usize;
        let value = dequantised(level, place);
        block[place] = if negative { -value } else { value } as i16;
        ended = bits.peek(2) == 0b10;
    }
    bits.skip(2);
    Ok(())
}

/// A square of `size` samples predicted from `source` at `from` (half a sample further right
/// and / or down with `half_x` / `half_y`: the rounded mean of the neighbours), written to
/// `target` at `to` or, with `average`, averaged with what is there (rounded up).
#[allow(clippy::too_many_arguments)] // two places in two planes, and how to predict
fn predict(
    target: &mut [u8],
    to: usize,
    source: &[u8],
    from: usize,
    stride: usize,
    size: usize,
    half_x: bool,
    half_y: bool,
    average: bool,
) -> Result<(), FilmError> {
    let across = size + half_x as usize;
    let rows = size + half_y as usize;
    let needed = from + (rows - 1) * stride + across;
    if needed > source.len() || to + (size - 1) * stride + size > target.len() {
        return refuse("a prediction reaching outside the picture");
    }
    for line in 0..size {
        let above = &source[from + line * stride..][..across];
        let out = &mut target[to + line * stride..][..size];
        let mut predicted = [0u8; 16];
        let predicted = &mut predicted[..size];
        match (half_x, half_y) {
            (false, false) => predicted.copy_from_slice(above),
            (true, false) => {
                for (p, pair) in predicted.iter_mut().zip(above.windows(2)) {
                    *p = ((pair[0] as u16 + pair[1] as u16 + 1) >> 1) as u8;
                }
            }
            (false, true) => {
                let below = &source[from + (line + 1) * stride..][..across];
                for ((p, &a), &b) in predicted.iter_mut().zip(above).zip(below) {
                    *p = ((a as u16 + b as u16 + 1) >> 1) as u8;
                }
            }
            (true, true) => {
                let below = &source[from + (line + 1) * stride..][..across];
                for ((p, a), b) in predicted
                    .iter_mut()
                    .zip(above.windows(2))
                    .zip(below.windows(2))
                {
                    *p = ((a[0] as u16 + a[1] as u16 + b[0] as u16 + b[1] as u16 + 2) >> 2) as u8;
                }
            }
        }
        if average {
            for (o, &p) in out.iter_mut().zip(predicted.iter()) {
                *o = ((*o as u16 + p as u16 + 1) >> 1) as u8;
            }
        } else {
            out.copy_from_slice(predicted);
        }
    }
    Ok(())
}

// The inverse transform's weights: cos(i pi / 16) sqrt(2) 2^14, rounded (the fourth one down).
const W1: i32 = 22725;
const W2: i32 = 21407;
const W3: i32 = 19266;
const W4: i32 = 16383;
const W5: i32 = 12873;
const W6: i32 = 8867;
const W7: i32 = 4520;
const ROW_SHIFT: u32 = 11;
const COLUMN_SHIFT: u32 = 20;

/// One row of a block transformed in place; a row of only its first coefficient is that one
/// times 8 throughout. All sums wrap at 32 bits and are cut to 16 when stored.
#[inline]
fn transform_row(row: &mut [i16; 8]) {
    if row[1..].iter().all(|&c| c == 0) {
        *row = [(row[0] as i32).wrapping_mul(8) as i16; 8];
        return;
    }
    let r = row.map(|c| c as i32);
    let m = |weight: i32, value: i32| weight.wrapping_mul(value);
    let mut a0 = m(W4, r[0]).wrapping_add(1 << (ROW_SHIFT - 1));
    let (mut a1, mut a2, mut a3) = (a0, a0, a0);
    a0 = a0.wrapping_add(m(W2, r[2]));
    a1 = a1.wrapping_add(m(W6, r[2]));
    a2 = a2.wrapping_sub(m(W6, r[2]));
    a3 = a3.wrapping_sub(m(W2, r[2]));
    let mut b0 = m(W1, r[1]).wrapping_add(m(W3, r[3]));
    let mut b1 = m(W3, r[1]).wrapping_sub(m(W7, r[3]));
    let mut b2 = m(W5, r[1]).wrapping_sub(m(W1, r[3]));
    let mut b3 = m(W7, r[1]).wrapping_sub(m(W5, r[3]));
    if r[4..].iter().any(|&c| c != 0) {
        a0 = a0.wrapping_add(m(W4, r[4])).wrapping_add(m(W6, r[6]));
        a1 = a1.wrapping_sub(m(W4, r[4])).wrapping_sub(m(W2, r[6]));
        a2 = a2.wrapping_sub(m(W4, r[4])).wrapping_add(m(W2, r[6]));
        a3 = a3.wrapping_add(m(W4, r[4])).wrapping_sub(m(W6, r[6]));
        b0 = b0.wrapping_add(m(W5, r[5])).wrapping_add(m(W7, r[7]));
        b1 = b1.wrapping_sub(m(W1, r[5])).wrapping_sub(m(W5, r[7]));
        b2 = b2.wrapping_add(m(W7, r[5])).wrapping_add(m(W3, r[7]));
        b3 = b3.wrapping_add(m(W3, r[5])).wrapping_sub(m(W1, r[7]));
    }
    row[0] = (a0.wrapping_add(b0) >> ROW_SHIFT) as i16;
    row[7] = (a0.wrapping_sub(b0) >> ROW_SHIFT) as i16;
    row[1] = (a1.wrapping_add(b1) >> ROW_SHIFT) as i16;
    row[6] = (a1.wrapping_sub(b1) >> ROW_SHIFT) as i16;
    row[2] = (a2.wrapping_add(b2) >> ROW_SHIFT) as i16;
    row[5] = (a2.wrapping_sub(b2) >> ROW_SHIFT) as i16;
    row[3] = (a3.wrapping_add(b3) >> ROW_SHIFT) as i16;
    row[4] = (a3.wrapping_sub(b3) >> ROW_SHIFT) as i16;
}

/// One column of a block (its eight values top to bottom) transformed to samples' values.
#[inline]
fn transform_column(c: [i32; 8]) -> [i32; 8] {
    let m = |weight: i32, value: i32| weight.wrapping_mul(value);
    let mut a0 = m(W4, c[0].wrapping_add((1 << (COLUMN_SHIFT - 1)) / W4));
    let (mut a1, mut a2, mut a3) = (a0, a0, a0);
    a0 = a0.wrapping_add(m(W2, c[2]));
    a1 = a1.wrapping_add(m(W6, c[2]));
    a2 = a2.wrapping_sub(m(W6, c[2]));
    a3 = a3.wrapping_sub(m(W2, c[2]));
    let mut b0 = m(W1, c[1]).wrapping_add(m(W3, c[3]));
    let mut b1 = m(W3, c[1]).wrapping_sub(m(W7, c[3]));
    let mut b2 = m(W5, c[1]).wrapping_sub(m(W1, c[3]));
    let mut b3 = m(W7, c[1]).wrapping_sub(m(W5, c[3]));
    a0 = a0.wrapping_add(m(W4, c[4]));
    a1 = a1.wrapping_sub(m(W4, c[4]));
    a2 = a2.wrapping_sub(m(W4, c[4]));
    a3 = a3.wrapping_add(m(W4, c[4]));
    b0 = b0.wrapping_add(m(W5, c[5]));
    b1 = b1.wrapping_sub(m(W1, c[5]));
    b2 = b2.wrapping_add(m(W7, c[5]));
    b3 = b3.wrapping_add(m(W3, c[5]));
    a0 = a0.wrapping_add(m(W6, c[6]));
    a1 = a1.wrapping_sub(m(W2, c[6]));
    a2 = a2.wrapping_add(m(W2, c[6]));
    a3 = a3.wrapping_sub(m(W6, c[6]));
    b0 = b0.wrapping_add(m(W7, c[7]));
    b1 = b1.wrapping_sub(m(W5, c[7]));
    b2 = b2.wrapping_add(m(W3, c[7]));
    b3 = b3.wrapping_sub(m(W1, c[7]));
    [
        a0.wrapping_add(b0) >> COLUMN_SHIFT,
        a1.wrapping_add(b1) >> COLUMN_SHIFT,
        a2.wrapping_add(b2) >> COLUMN_SHIFT,
        a3.wrapping_add(b3) >> COLUMN_SHIFT,
        a3.wrapping_sub(b3) >> COLUMN_SHIFT,
        a2.wrapping_sub(b2) >> COLUMN_SHIFT,
        a1.wrapping_sub(b1) >> COLUMN_SHIFT,
        a0.wrapping_sub(b0) >> COLUMN_SHIFT,
    ]
}

/// A block of coefficients transformed (the block is used up) and written to `plane` at `at`,
/// or with `add` added to what is there; either way kept to 0 .. 255.
fn transform(
    block: &mut [i16; 64],
    plane: &mut [u8],
    at: usize,
    stride: usize,
    add: bool,
) -> Result<(), FilmError> {
    if at + 7 * stride + 8 > plane.len() {
        return refuse("a block outside the picture");
    }
    for row in block.as_chunks_mut::<8>().0 {
        transform_row(row);
    }
    let mut samples = [[0i32; 8]; 8];
    for (x, column) in samples.iter_mut().enumerate() {
        *column = transform_column(std::array::from_fn(|y| block[y * 8 + x] as i32));
    }
    for y in 0..8 {
        let out = &mut plane[at + y * stride..][..8];
        for (x, o) in out.iter_mut().enumerate() {
            let value = samples[x][y];
            *o = if add { *o as i32 + value } else { value }.clamp(0, 255) as u8;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::film::mpeg_bits::bytes_of;

    /// A start code and what follows it, from a text of bits.
    fn unit(code: u8, bits: &str) -> Vec<u8> {
        let mut unit = vec![0, 0, 1, code];
        unit.extend(bytes_of(bits));
        unit
    }

    /// A sequence of 32 x 16 at 25 pictures a second with the default matrices, and a closed
    /// group's header.
    fn headers() -> Vec<u8> {
        let mut headers = unit(
            SEQUENCE_START,
            "000000100000 000000010000 0001 0011 000000000000000001 1 0000010100 0 0 0",
        );
        headers.extend(unit(GROUP_START, "0000000000000000000000000 1 0"));
        headers
    }

    /// An intra macroblock: its four luma blocks' first DC difference as given (the others
    /// 0), chroma unchanged, no other coefficients.
    fn intra_macroblock(first_dc: &str) -> String {
        format!("1 1 {first_dc} 10 100 10 100 10 100 10 00 10 00 10")
    }

    /// An I picture of two macroblocks, luma 100 and 128, chroma 128.
    fn intra_picture() -> Vec<u8> {
        let mut picture = unit(PICTURE_START, "0000000000 001 1111111111111111");
        // quantiser 1, no extra information; DC differences -28 and +28 (five bits each)
        picture.extend(unit(
            SLICE_FIRST,
            &format!(
                "00001 0 {} {}",
                intra_macroblock("1110 00011"),
                intra_macroblock("1110 11100")
            ),
        ));
        picture
    }

    /// A P picture: the first macroblock from half a sample to the right, the second from
    /// its own place (its vector's difference takes the half sample back).
    fn predicted_picture() -> Vec<u8> {
        let mut picture = unit(PICTURE_START, "0000000001 010 1111111111111111 0 001");
        picture.extend(unit(SLICE_FIRST, "00001 0  1 001 01 0 1  1 001 01 1 1"));
        picture
    }

    /// A B picture: both macroblocks the mean of the two pictures around it.
    fn bidirectional_picture() -> Vec<u8> {
        let mut picture = unit(PICTURE_START, "0000000010 011 1111111111111111 0 001 0 001");
        picture.extend(unit(SLICE_FIRST, "00001 0  1 10 1 1 1 1  1 10 1 1 1 1"));
        picture
    }

    fn luma_row(picture: &Picture) -> Vec<u8> {
        picture.packed_plane(0)[..32].to_vec()
    }

    #[test]
    fn pictures_come_out_in_display_order_with_the_predicted_values() {
        let mut first = headers();
        first.extend(intra_picture());
        let stream = [first, predicted_picture(), bidirectional_picture()].concat();
        let packets = cut(&stream);
        assert_eq!(packets.len(), 3);
        assert_eq!(packets[0].start, 0);
        assert_eq!(packets[2].end, stream.len());

        let mut decoder = VideoDecoder::new();
        assert!(
            decoder
                .decode(&stream[packets[0].clone()])
                .unwrap()
                .is_none()
        );
        assert_eq!(decoder.frame_rate(), Some((25, 1)));

        // the P picture releases the I picture
        let intra = decoder
            .decode(&stream[packets[1].clone()])
            .unwrap()
            .unwrap();
        assert_eq!((intra.width, intra.height), (32, 16));
        let mut wanted = [100u8; 32];
        wanted[16..].fill(128);
        assert_eq!(luma_row(intra), wanted);
        assert!(intra.packed_plane(0).chunks(32).all(|row| row == wanted));
        assert!(intra.packed_plane(1).iter().all(|&c| c == 128));
        assert_eq!(intra.packed_plane(2).len(), 16 * 8);

        // the B picture comes at once: the mean of the two, rounded up
        let between = decoder
            .decode(&stream[packets[2].clone()])
            .unwrap()
            .unwrap();
        wanted[15] = 107;
        assert_eq!(luma_row(between), wanted);

        // the end brings the P picture: its sixteenth column is half way to the next
        let predicted = decoder.decode(&[]).unwrap().unwrap();
        wanted[15] = 114;
        assert_eq!(luma_row(predicted), wanted);
        assert!(predicted.packed_plane(2).iter().all(|&c| c == 128));
        assert!(decoder.decode(&[]).unwrap().is_none());
        assert_eq!(decoder.vectors_outside, 0);
    }

    #[test]
    fn b_pictures_of_an_open_group_wait_for_two_pictures() {
        // the group's header without its closed flag
        let mut first = headers();
        let closed_flag = first.len() - 1;
        first[closed_flag] = 0;
        first.extend(intra_picture());
        let mut decoder = VideoDecoder::new();
        assert!(decoder.decode(&first).unwrap().is_none());
        assert!(decoder.decode(&bidirectional_picture()).unwrap().is_none());
        assert!(decoder.decode(&predicted_picture()).unwrap().is_some());
        assert!(decoder.decode(&bidirectional_picture()).unwrap().is_some());
        assert!(decoder.decode(&[0, 0, 1, SEQUENCE_END]).unwrap().is_some());
    }

    #[test]
    fn a_vector_outside_the_picture_is_counted_and_refusals_are_errors() {
        let mut first = headers();
        first.extend(intra_picture());
        let mut decoder = VideoDecoder::new();
        decoder.decode(&first).unwrap();
        // both macroblocks from half a sample to the right: the second reaches outside
        let mut outside = unit(PICTURE_START, "0000000001 010 1111111111111111 0 001");
        outside.extend(unit(SLICE_FIRST, "00001 0  1 001 01 0 1  1 001 1 1"));
        decoder.decode(&outside).unwrap();
        assert_eq!(decoder.vectors_outside, 1);

        let mut decoder = VideoDecoder::new();
        assert!(
            decoder.decode(&intra_picture()).is_err(),
            "no sequence header"
        );
        let mpeg2 = [headers(), unit(EXTENSION_START, "0001 0000")].concat();
        assert!(decoder.decode(&mpeg2).is_err());
        // a slice with quantiser 0
        let mut zero = headers();
        zero.extend(unit(PICTURE_START, "0000000000 001 1111111111111111"));
        zero.extend(unit(SLICE_FIRST, "00000 0 1 1"));
        assert!(decoder.decode(&zero).is_err());
    }

    #[test]
    fn damaged_streams_are_refused_or_decoded_but_never_more() {
        let mut first = headers();
        first.extend(intra_picture());
        let stream = [first, predicted_picture(), bidirectional_picture()].concat();
        let run = |damaged: &[u8]| {
            let mut decoder = VideoDecoder::new();
            for packet in cut(damaged) {
                let _ = decoder.decode(&damaged[packet]);
            }
            let _ = decoder.decode(&[]);
        };
        for end in 0..stream.len() {
            run(&stream[..end]);
        }
        for bit in 0..stream.len() * 8 {
            let mut damaged = stream.clone();
            damaged[bit / 8] ^= 0x80 >> (bit % 8);
            run(&damaged);
        }
    }

    #[test]
    fn the_transform_of_one_coefficient_is_flat() {
        let mut block = [0i16; 64];
        block[0] = 800;
        let mut plane = [7u8; 64];
        transform(&mut block, &mut plane, 0, 8, false).unwrap();
        assert_eq!(plane, [100; 64]);
        let mut block = [0i16; 64];
        block[0] = -2048;
        transform(&mut block, &mut plane, 0, 8, true).unwrap();
        assert_eq!(plane, [0; 64]);
        assert!(transform(&mut block, &mut plane, 1, 8, true).is_err());
    }
}
