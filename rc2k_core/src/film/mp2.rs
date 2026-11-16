//! An MPEG audio layer II decoder in whole numbers, giving the same 16-bit samples as FFmpeg's
//! `mp2` decoder (its fixed-point one).
//!
//! `cut` divides a stream into packets of one frame each, as FFmpeg's parser does for a stream
//! without damage; `AudioDecoder::decode` takes one packet and appends its 1152 samples a
//! channel, interleaved. Frames of layers I and III are refused, and so is the free format; the
//! checksum of a protected frame is read over and not checked, as in FFmpeg.
//!
//! The steps and their arithmetic (the samples' requantisation, the 32-point transform, the
//! synthesis window with its rounding carried from one sample to the next) are ported from
//! FFmpeg (libavcodec/mpegaudiodec_template.c, mpegaudiodecheader.c, mpegaudio.c,
//! mpegaudio_parser.c, mpegaudiodsp_template.c, dct32_template.c; LGPL-2.1-or-later; copyright
//! (c) 2001-2003 Fabrice Bellard).

use super::mp2_tables as tables;
use super::mpeg_bits::{Bits, FilmError, refuse};

/// The samples a channel in a layer II frame.
pub const FRAME_SAMPLES: usize = 1152;

const SUBBANDS: usize = 32;
/// The fractional bits of a subband sample.
const FRACTION_BITS: u32 = 23;
const ONE: f64 = (1u32 << FRACTION_BITS) as f64;

/// What a frame's four header bytes say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub layer: u8,
    /// A stream of the lower sampling rates (MPEG-2 and 2.5).
    lower_rates: bool,
    pub sample_rate: u32,
    pub channels: usize,
    /// Bits a second.
    pub bit_rate: u32,
    /// The frame's bytes, header included.
    pub frame_bytes: usize,
    /// Two bytes of checksum follow the header.
    protected: bool,
    joint_stereo: bool,
    /// For joint stereo: the first subband whose samples both channels share, in fours.
    mode_extension: u32,
}

impl Header {
    /// The header in `word` (the four bytes, first byte highest); None for what is no header
    /// or names no bit rate.
    pub fn read(word: u32) -> Option<Self> {
        let no_header = word & 0xFFE0_0000 != 0xFFE0_0000
            || word & (3 << 19) == 1 << 19
            || word & (3 << 17) == 0
            || word & (0xF << 12) == 0xF << 12
            || word & (3 << 10) == 3 << 10;
        if no_header {
            return None;
        }
        let (lower_rates, quarter_rates) = if word & (1 << 20) != 0 {
            (word & (1 << 19) == 0, false)
        } else {
            (true, true)
        };
        let layer = 4 - ((word >> 17) & 3);
        let rate_shift = lower_rates as u32 + quarter_rates as u32;
        let sample_rate = [44100u32, 48000, 32000][((word >> 10) & 3) as usize] >> rate_shift;
        let bit_rate_index = ((word >> 12) & 0xF) as usize;
        if bit_rate_index == 0 {
            return None;
        }
        let padding = (word >> 9) & 1;
        let thousands = tables::BIT_RATES
            [lower_rates as usize * 45 + (layer as usize - 1) * 15 + bit_rate_index]
            as u32;
        let frame_bytes = match layer {
            1 => (thousands * 12000 / sample_rate + padding) * 4,
            2 => thousands * 144_000 / sample_rate + padding,
            _ => thousands * 144_000 / (sample_rate << lower_rates as u32) + padding,
        };
        let mode = (word >> 6) & 3;
        Some(Self {
            layer: layer as u8,
            lower_rates,
            sample_rate,
            channels: if mode == 3 { 1 } else { 2 },
            bit_rate: thousands * 1000,
            frame_bytes: frame_bytes as usize,
            protected: (word >> 16) & 1 == 0,
            joint_stereo: mode == 1,
            mode_extension: (word >> 4) & 3,
        })
    }
}

/// The stream cut into packets, each a range of `stream`: a packet ends where the frame of the
/// first header found after the packet before ends; what is left at the end is the last one
/// (an ID3 tag there is left out).
pub fn cut(stream: &[u8]) -> Vec<std::ops::Range<usize>> {
    let mut packets = Vec::new();
    let mut start = 0;
    let mut state = 0u32;
    // The bytes of the frame found that are still to pass.
    let mut frame_left = 0usize;
    let mut i = 0;
    while i < stream.len() {
        if frame_left > 0 {
            let step = frame_left.min(stream.len() - i);
            i += step;
            frame_left -= step;
            state = 0;
            if frame_left == 0 {
                packets.push(start..i);
                start = i;
            }
        } else {
            state = state << 8 | stream[i] as u32;
            i += 1;
            if let Some(header) = Header::read(state)
                && header.frame_bytes >= 4
            {
                frame_left = header.frame_bytes - 4;
            }
        }
    }
    let rest = &stream[start..];
    if !rest.is_empty() && !(rest.len() >= 128 && rest.starts_with(b"TAG")) {
        packets.push(start..stream.len());
    }
    packets
}

/// `x` as a fixed-point number of `FRACTION_BITS`, rounded.
fn fixed(x: f64) -> i32 {
    (x * ONE + 0.5) as i32
}

// The 32-point transform's weights as 32-bit fractions: 1 / (2 cos(pi (2 k + 1) / 2^(6 - pass)))
// for each pass, each divided by the power of two that keeps it below a half (the transform
// multiplies that back in), rounded.
const COS0: [i32; 16] = [
    1075036753, 1085490621, 1106914669, 1140405281, 1187781572, 1251843312, 1336817425, 1449139879,
    1598879467, 1802489638, 2088574387, 1255676567, 1593609622, 1104762768, 1829445839, 1367679739,
];
const COS1: [i32; 8] = [
    1078937202, 1122057232, 1217503044, 1389039203, 1692549166, 1138893993, 1849463489, 1369329156,
];
const COS2: [i32; 4] = [1094777670, 1291378312, 1932684223, 1375954754];
const COS3: [i32; 2] = [1162209775, 1402911301];
const COS4: i32 = 1518500250;

/// The 32-point transform of `input` into `out`. Sums wrap at 32 bits.
fn transform32(out: &mut [i32], input: &[i32; SUBBANDS]) {
    let mut v = *input;
    // A butterfly: the sum stays in `a`, the difference times `weight` (which is a fraction of
    // `2^-shift` of the true weight) goes to `b`.
    fn bf(v: &mut [i32; 32], a: usize, b: usize, weight: i32, shift: u32) {
        let sum = v[a].wrapping_add(v[b]);
        let difference = v[a].wrapping_sub(v[b]);
        v[a] = sum;
        v[b] = ((difference.wrapping_shl(shift) as i64 * weight as i64) >> 32) as i32;
    }
    // The first four passes, by groups of the inputs they join.
    bf(&mut v, 0, 31, COS0[0], 1);
    bf(&mut v, 15, 16, COS0[15], 5);
    bf(&mut v, 0, 15, COS1[0], 1);
    bf(&mut v, 16, 31, -COS1[0], 1);
    bf(&mut v, 7, 24, COS0[7], 1);
    bf(&mut v, 8, 23, COS0[8], 1);
    bf(&mut v, 7, 8, COS1[7], 4);
    bf(&mut v, 23, 24, -COS1[7], 4);
    bf(&mut v, 0, 7, COS2[0], 1);
    bf(&mut v, 8, 15, -COS2[0], 1);
    bf(&mut v, 16, 23, COS2[0], 1);
    bf(&mut v, 24, 31, -COS2[0], 1);
    bf(&mut v, 3, 28, COS0[3], 1);
    bf(&mut v, 12, 19, COS0[12], 2);
    bf(&mut v, 3, 12, COS1[3], 1);
    bf(&mut v, 19, 28, -COS1[3], 1);
    bf(&mut v, 4, 27, COS0[4], 1);
    bf(&mut v, 11, 20, COS0[11], 2);
    bf(&mut v, 4, 11, COS1[4], 1);
    bf(&mut v, 20, 27, -COS1[4], 1);
    bf(&mut v, 3, 4, COS2[3], 3);
    bf(&mut v, 11, 12, -COS2[3], 3);
    bf(&mut v, 19, 20, COS2[3], 3);
    bf(&mut v, 27, 28, -COS2[3], 3);
    for group in 0..4 {
        let at = group * 8;
        bf(&mut v, at, at + 3, COS3[0], 1);
        bf(&mut v, at + 4, at + 7, -COS3[0], 1);
    }
    bf(&mut v, 1, 30, COS0[1], 1);
    bf(&mut v, 14, 17, COS0[14], 3);
    bf(&mut v, 1, 14, COS1[1], 1);
    bf(&mut v, 17, 30, -COS1[1], 1);
    bf(&mut v, 6, 25, COS0[6], 1);
    bf(&mut v, 9, 22, COS0[9], 1);
    bf(&mut v, 6, 9, COS1[6], 2);
    bf(&mut v, 22, 25, -COS1[6], 2);
    bf(&mut v, 1, 6, COS2[1], 1);
    bf(&mut v, 9, 14, -COS2[1], 1);
    bf(&mut v, 17, 22, COS2[1], 1);
    bf(&mut v, 25, 30, -COS2[1], 1);
    bf(&mut v, 2, 29, COS0[2], 1);
    bf(&mut v, 13, 18, COS0[13], 3);
    bf(&mut v, 2, 13, COS1[2], 1);
    bf(&mut v, 18, 29, -COS1[2], 1);
    bf(&mut v, 5, 26, COS0[5], 1);
    bf(&mut v, 10, 21, COS0[10], 1);
    bf(&mut v, 5, 10, COS1[5], 2);
    bf(&mut v, 21, 26, -COS1[5], 2);
    bf(&mut v, 2, 5, COS2[2], 1);
    bf(&mut v, 10, 13, -COS2[2], 1);
    bf(&mut v, 18, 21, COS2[2], 1);
    bf(&mut v, 26, 29, -COS2[2], 1);
    for group in 0..4 {
        let at = group * 8;
        bf(&mut v, at + 1, at + 2, COS3[1], 2);
        bf(&mut v, at + 5, at + 6, -COS3[1], 2);
    }
    // The fifth pass, four values at a time; every second four is summed further.
    for four in 0..8 {
        let at = four * 4;
        bf(&mut v, at, at + 1, COS4, 1);
        bf(&mut v, at + 2, at + 3, -COS4, 1);
        v[at + 2] = v[at + 2].wrapping_add(v[at + 3]);
        if four % 2 == 1 {
            v[at] = v[at].wrapping_add(v[at + 2]);
            v[at + 2] = v[at + 2].wrapping_add(v[at + 1]);
            v[at + 1] = v[at + 1].wrapping_add(v[at + 3]);
        }
    }
    // The sixth pass and the outputs' order.
    let add = |v: &mut [i32; 32], a: usize, b: usize| v[a] = v[a].wrapping_add(v[b]);
    for (a, b) in [
        (8, 12),
        (12, 10),
        (10, 14),
        (14, 9),
        (9, 13),
        (13, 11),
        (11, 15),
    ] {
        add(&mut v, a, b);
    }
    for (place, value) in [0, 16, 8, 24, 4, 20, 12, 28, 2, 18, 10, 26, 6, 22, 14, 30]
        .into_iter()
        .zip(0..16)
    {
        out[place] = v[value];
    }
    for (a, b) in [
        (24, 28),
        (28, 26),
        (26, 30),
        (30, 25),
        (25, 29),
        (29, 27),
        (27, 31),
    ] {
        add(&mut v, a, b);
    }
    for (place, (a, b)) in [1, 17, 9, 25, 5, 21, 13, 29, 3, 19, 11, 27, 7, 23, 15]
        .into_iter()
        .zip([
            (16, 24),
            (17, 25),
            (18, 26),
            (19, 27),
            (20, 28),
            (21, 29),
            (22, 30),
            (23, 31),
            (24, 20),
            (25, 21),
            (26, 22),
            (27, 23),
            (28, 18),
            (29, 19),
            (30, 17),
        ])
    {
        out[place] = v[a].wrapping_add(v[b]);
    }
    out[31] = v[31];
}

/// The bits of a sum the output leaves below a sample.
const OUT_SHIFT: u32 = 16 + FRACTION_BITS - 15;

/// The sample in `sum`, which keeps what the sample leaves over for the next one.
fn rounded(sum: &mut i64) -> i16 {
    let sample = (*sum >> OUT_SHIFT) as i32;
    *sum &= (1 << OUT_SHIFT) - 1;
    sample.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

/// The tables made once for a decoder.
struct Made {
    /// The synthesis window: its 512 values (the second half the first mirrored, with signs).
    window: [i32; 512],
    /// For a sample of n + 2 bits: 2^(n+2) / (2^(n+2) - 1) times 2, 2^(2/3) and 2^(1/3).
    sample_scale: [[i32; 3]; 15],
    /// The same for the grouped quantisers of 3, 5 and 9 steps.
    grouped_scale: [[i32; 3]; 3],
    /// For the quantisers of 3, 5 and 9 steps: a code's three samples, four bits each.
    grouped: [Vec<u16>; 3],
}

impl Made {
    fn new() -> Self {
        let mut window = [0i32; 512];
        for (i, &value) in tables::SYNTHESIS_WINDOW_HALF.iter().enumerate() {
            window[i] = value;
            if i != 0 {
                window[512 - i] = if i & 63 != 0 { -value } else { value };
            }
        }
        const THIRDS: [f64; 3] = [1.0, 0.793_700_525_9, 0.629_960_524_9];
        let mut sample_scale = [[0i32; 3]; 15];
        for (i, scale) in sample_scale.iter_mut().enumerate() {
            let bits = i as u32 + 2;
            let norm = (((1i64 << bits) << FRACTION_BITS) / ((1i64 << bits) - 1)) as i32;
            for (third, value) in scale.iter_mut().enumerate() {
                *value =
                    ((norm as i64 * fixed(THIRDS[third] * 2.0) as i64) >> FRACTION_BITS) as i32;
            }
        }
        let grouped_scale = [4.0 / 3.0, 4.0 / 5.0, 4.0 / 9.0].map(|v| THIRDS.map(|t| fixed(t * v)));
        let grouped = [(3u16, 6u32), (5, 8), (9, 11)].map(|(steps, bits)| {
            (0..1u16 << bits)
                .map(|code| {
                    let first = code % steps;
                    let rest = code / steps;
                    first + ((rest % steps) << 4) + ((rest / steps) << 8)
                })
                .collect()
        });
        Self {
            window,
            sample_scale,
            grouped_scale,
            grouped,
        }
    }
}

/// A sample of `bits` bits requantised with scale factor `scale_factor`.
fn requantised(made: &Made, bits: u32, sample: u32, scale_factor: usize) -> i32 {
    let n = bits - 1;
    let shift = (scale_factor / 3) as u32 + n;
    let centred = sample as i32 - (1 << n) + 1;
    let value = centred as i64 * made.sample_scale[n as usize - 1][scale_factor % 3] as i64;
    ((value + (1i64 << (shift - 1))) >> shift) as i32
}

/// One of a group's three samples (`sample` of `steps` steps) requantised.
fn requantised_grouped(made: &Made, steps: i32, sample: i32, scale_factor: usize) -> i32 {
    let shift = (scale_factor / 3) as u32;
    let value = (sample - (steps >> 1))
        .wrapping_mul(made.grouped_scale[(steps >> 2) as usize][scale_factor % 3]);
    if shift > 0 {
        value.wrapping_add(1 << (shift - 1)) >> shift
    } else {
        value
    }
}

/// A channel's synthesis filter: the last sixteen transforms, newest at `offset`.
struct Synthesis {
    values: Vec<i32>,
    offset: usize,
}

impl Synthesis {
    fn new() -> Self {
        Self {
            values: vec![0; 1024],
            offset: 0,
        }
    }

    /// 32 subband samples in, 32 samples out (to `out`, `step` apart); `carry` is what the
    /// sample before left over.
    fn run(
        &mut self,
        window: &[i32; 512],
        carry: &mut i32,
        subbands: &[i32; SUBBANDS],
        out: &mut [i16],
        step: usize,
    ) {
        let buffer = &mut self.values[self.offset..self.offset + 544];
        transform32(buffer, subbands);
        // The first 32 again after the 512, so that no sum has to wrap around.
        buffer.copy_within(0..32, 512);
        let buffer = &*buffer;
        let sum8 = |w: usize, p: usize| -> i64 {
            (0..8)
                .map(|k| window[w + k * 64] as i64 * buffer[p + k * 64] as i64)
                .sum()
        };
        let mut sum = *carry as i64;
        sum += sum8(0, 16);
        sum -= sum8(32, 48);
        out[0] = rounded(&mut sum);
        // Two samples at a time, one from each end.
        for j in 1..16 {
            let mut other = 0i64;
            for k in 0..8 {
                let value = buffer[16 + j + k * 64] as i64;
                sum += window[j + k * 64] as i64 * value;
                other -= window[32 - j + k * 64] as i64 * value;
                let value = buffer[48 - j + k * 64] as i64;
                sum -= window[j + 32 + k * 64] as i64 * value;
                other -= window[64 - j + k * 64] as i64 * value;
            }
            out[j * step] = rounded(&mut sum);
            sum += other;
            out[(32 - j) * step] = rounded(&mut sum);
        }
        sum -= sum8(48, 32);
        out[16 * step] = rounded(&mut sum);
        *carry = sum as i32;
        self.offset = (self.offset + 512 - 32) & 511;
    }
}

/// What a decoded frame was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub sample_rate: u32,
    pub channels: usize,
    /// Samples a channel.
    pub samples: usize,
}

/// The decoder.
pub struct AudioDecoder {
    made: Made,
    synthesis: [Synthesis; 2],
    /// What the last sample's rounding left over (one for both channels).
    carry: i32,
    /// The frame's subband samples: by channel, 36 times 32.
    subbands: Vec<[[i32; SUBBANDS]; 36]>,
}

impl Default for AudioDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDecoder {
    pub fn new() -> Self {
        Self {
            made: Made::new(),
            synthesis: [Synthesis::new(), Synthesis::new()],
            carry: 0,
            subbands: vec![[[0; SUBBANDS]; 36]; 2],
        }
    }

    /// One packet (a frame; zero bytes before it are passed over, bytes after it left) decoded:
    /// its samples appended to `pcm`, the channels interleaved. None for an ID3 tag.
    pub fn decode(
        &mut self,
        packet: &[u8],
        pcm: &mut Vec<i16>,
    ) -> Result<Option<Frame>, FilmError> {
        let zeros = packet.iter().take_while(|&&b| b == 0).count();
        let packet = &packet[zeros..];
        let Some(&[a, b, c, d]) = packet.first_chunk::<4>() else {
            return refuse("a sound packet of less than a header");
        };
        if [a, b, c] == *b"TAG" {
            return Ok(None);
        }
        let Some(header) = Header::read(u32::from_be_bytes([a, b, c, d])) else {
            return refuse("a sound packet without a frame header");
        };
        if header.layer != 2 {
            return refuse(format!(
                "sound of layer {}, which is not decoded",
                header.layer
            ));
        }
        let frame = &packet[..header.frame_bytes.min(packet.len())];
        let mut bits = Bits::new(&frame[4..]);
        if header.protected {
            bits.skip(16);
        }
        self.read_subbands(&header, &mut bits);
        let channels = header.channels;
        let base = pcm.len();
        pcm.resize(base + FRAME_SAMPLES * channels, 0);
        for channel in 0..channels {
            for (i, subbands) in self.subbands[channel].iter().enumerate() {
                self.synthesis[channel].run(
                    &self.made.window,
                    &mut self.carry,
                    subbands,
                    &mut pcm[base + i * SUBBANDS * channels + channel..],
                    channels,
                );
            }
        }
        Ok(Some(Frame {
            sample_rate: header.sample_rate,
            channels,
            samples: FRAME_SAMPLES,
        }))
    }

    /// The frame's bit allocation, scale factors and samples, requantised into `subbands`.
    // The loops' numbers name a channel, a subband or a part in several tables at once.
    #[allow(clippy::needless_range_loop)]
    fn read_subbands(&mut self, header: &Header, bits: &mut Bits) {
        let channels = header.channels;
        let rate_a_channel = header.bit_rate / 1000 / channels as u32;
        let table = if header.lower_rates {
            4
        } else if (header.sample_rate == 48000 && rate_a_channel >= 56)
            || (56..=80).contains(&rate_a_channel)
        {
            0
        } else if header.sample_rate != 48000 && rate_a_channel >= 96 {
            1
        } else if header.sample_rate != 32000 && rate_a_channel <= 48 {
            2
        } else {
            3
        };
        let limit = tables::SUBBAND_LIMITS[table] as usize;
        let allocations: &[u8] = match table {
            0 | 1 => &tables::ALLOC_TABLE_1,
            2 | 3 => &tables::ALLOC_TABLE_3,
            _ => &tables::ALLOC_TABLE_4,
        };
        // The subbands from `bound` on are coded once for both channels.
        let bound = if header.joint_stereo {
            ((header.mode_extension as usize + 1) * 4).min(limit)
        } else {
            limit
        };

        let mut allocation = [[0u8; SUBBANDS]; 2];
        let mut at = 0;
        for subband in 0..limit {
            let width = allocations[at] as u32;
            if subband < bound {
                for channel in allocation.iter_mut().take(channels) {
                    channel[subband] = bits.take(width) as u8;
                }
            } else {
                let both = bits.take(width) as u8;
                allocation[0][subband] = both;
                allocation[1][subband] = both;
            }
            at += 1 << width;
        }

        // How each subband's three scale factors are coded, then the factors.
        let mut pattern = [[0u8; SUBBANDS]; 2];
        for subband in 0..limit {
            for channel in 0..channels {
                if allocation[channel][subband] != 0 {
                    pattern[channel][subband] = bits.take(2) as u8;
                }
            }
        }
        let mut factors = [[[0u8; 3]; SUBBANDS]; 2];
        for subband in 0..limit {
            for channel in 0..channels {
                if allocation[channel][subband] == 0 {
                    continue;
                }
                let first = bits.take(6) as u8;
                factors[channel][subband] = match pattern[channel][subband] {
                    0 => [first, bits.take(6) as u8, bits.take(6) as u8],
                    2 => [first; 3],
                    1 => [first, first, bits.take(6) as u8],
                    _ => {
                        let last = bits.take(6) as u8;
                        [first, last, last]
                    }
                };
            }
        }

        // The samples: three parts of four groups of three samples a subband.
        let made = &self.made;
        for part in 0..3 {
            for group in (0..12).step_by(3) {
                let row = part * 12 + group;
                let mut at = 0;
                for subband in 0..limit {
                    let width = allocations[at] as u32;
                    // The channels that read their own samples; past `bound` the second takes
                    // the first's with its own scale factor.
                    let (readers, shared) = if subband < bound {
                        (channels, false)
                    } else {
                        (1, true)
                    };
                    for channel in 0..readers {
                        let allocated = allocation[channel][subband] as usize;
                        let targets = if shared { 0..2 } else { channel..channel + 1 };
                        if allocated == 0 {
                            for target in targets {
                                for m in 0..3 {
                                    self.subbands[target][row + m][subband] = 0;
                                }
                            }
                            continue;
                        }
                        let quantiser = allocations[at + allocated] as usize;
                        let sample_bits = tables::QUANT_BITS[quantiser];
                        let mut read = [0i32; 3];
                        let steps = tables::QUANT_STEPS[quantiser];
                        if sample_bits < 0 {
                            // three samples in one code
                            let code = bits.take(-sample_bits as u32) as usize;
                            let split = made.grouped[(steps >> 2) as usize]
                                .get(code)
                                .copied()
                                .unwrap_or(0) as i32;
                            read = [split & 15, (split >> 4) & 15, split >> 8];
                        } else {
                            for value in &mut read {
                                *value = bits.take(sample_bits as u32) as i32;
                            }
                        }
                        for target in targets {
                            let factor = factors[target][subband][part] as usize;
                            for (m, &value) in read.iter().enumerate() {
                                self.subbands[target][row + m][subband] = if sample_bits < 0 {
                                    requantised_grouped(made, steps, value, factor)
                                } else {
                                    requantised(made, sample_bits as u32, value as u32, factor)
                                };
                            }
                        }
                    }
                    at += 1 << width;
                }
                for channel in self.subbands.iter_mut().take(channels) {
                    for line in &mut channel[row..row + 3] {
                        line[limit..].fill(0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::film::mpeg_bits::bytes_of;

    /// A frame of 44100 Hz stereo at 224,000 bits a second (731 bytes) with `body` after the
    /// header and zeros after that.
    fn frame(body: &str) -> Vec<u8> {
        let mut frame = vec![0xFF, 0xFD, 0xB0, 0x00];
        frame.extend(bytes_of(body));
        frame.resize(731, 0);
        frame
    }

    #[test]
    fn a_header_names_the_frame() {
        let header = Header::read(0xFFFD_B000).unwrap();
        assert_eq!(
            (
                header.layer,
                header.sample_rate,
                header.channels,
                header.bit_rate
            ),
            (2, 44100, 2, 224_000)
        );
        assert_eq!(header.frame_bytes, 731);
        // with padding, and mono
        assert_eq!(Header::read(0xFFFD_B200).unwrap().frame_bytes, 732);
        assert_eq!(Header::read(0xFFFD_B0C0).unwrap().channels, 1);
        // no sync, no bit rate, a rate that is none
        assert!(Header::read(0xFEFD_B000).is_none());
        assert!(Header::read(0xFFFD_0000).is_none());
        assert!(Header::read(0xFFFD_BC00).is_none());
    }

    #[test]
    fn a_stream_is_cut_at_its_frames_and_silence_decodes_to_zeros() {
        let stream = [frame(""), frame(""), vec![0xFF, 0xFD, 0xB0]].concat();
        let packets = cut(&stream);
        assert_eq!(packets, [0..731, 731..1462, 1462..1465]);
        let mut decoder = AudioDecoder::new();
        let mut pcm = Vec::new();
        let decoded = decoder
            .decode(&stream[packets[0].clone()], &mut pcm)
            .unwrap();
        assert_eq!(
            decoded,
            Some(Frame {
                sample_rate: 44100,
                channels: 2,
                samples: FRAME_SAMPLES
            })
        );
        assert_eq!(pcm, vec![0; 2 * FRAME_SAMPLES]);
        assert!(
            decoder
                .decode(&stream[packets[2].clone()], &mut pcm)
                .is_err()
        );
        assert_eq!(decoder.decode(b"TAG and so on", &mut pcm), Ok(None));
        // layer III
        assert!(decoder.decode(&[0xFF, 0xFB, 0xB0, 0x00], &mut pcm).is_err());
    }

    /// The first subband of the left channel with a three-step quantiser, one scale factor
    /// for the frame, and the same three samples twelve times: a tone at the bottom of the
    /// band, in the left channel only.
    fn tone() -> Vec<u8> {
        let mut body = String::from("0001");
        // no other allocation: the other 59 fields of 4, 4, 3 and 2 bits
        body.push_str(&"0".repeat(4 + 2 * 2 * 4 + 8 * 2 * 4 + 12 * 2 * 3 + 7 * 2 * 2));
        body.push_str("10 001100");
        body.push_str(&"11010".repeat(12));
        frame(&body)
    }

    #[test]
    fn a_tone_comes_out_of_the_channel_it_is_in() {
        let mut decoder = AudioDecoder::new();
        let mut pcm = Vec::new();
        decoder.decode(&tone(), &mut pcm).unwrap();
        decoder.decode(&tone(), &mut pcm).unwrap();
        assert!(pcm.iter().skip(1).step_by(2).all(|&right| right == 0));
        let loudest = pcm
            .iter()
            .step_by(2)
            .map(|s| s.unsigned_abs())
            .max()
            .unwrap();
        assert!(loudest > 1000, "{loudest}");
        // the values this decoder gave when it was compared with FFmpeg's on the films
        let sum = pcm.iter().fold(0u64, |sum, &s| {
            sum.wrapping_mul(31).wrapping_add(s as u16 as u64)
        });
        assert_eq!(sum, TONE_SUM, "{sum:#x}");
    }

    const TONE_SUM: u64 = 0x8a85_5b41_dd37_39bf;

    #[test]
    fn damaged_frames_are_refused_or_decoded_but_never_more() {
        let good = tone();
        let mut decoder = AudioDecoder::new();
        let mut pcm = Vec::new();
        for end in 0..40 {
            let _ = decoder.decode(&good[..end], &mut pcm);
        }
        for bit in 0..40 * 8 {
            let mut damaged = good.clone();
            damaged[bit / 8] ^= 0x80 >> (bit % 8);
            for packet in cut(&damaged) {
                let _ = decoder.decode(&damaged[packet], &mut pcm);
            }
            pcm.clear();
        }
        // every bit rate, sampling rate and mode, all bits set after the header
        for third in 0..=255u8 {
            for fourth in [0x00, 0x40, 0x70, 0xC0] {
                for second in [0xFD, 0xFC, 0xF5, 0xE5] {
                    let mut all = vec![0xFF; 2000];
                    all[1] = second;
                    all[2] = third;
                    all[3] = fourth;
                    let _ = decoder.decode(&all, &mut pcm);
                    pcm.clear();
                }
            }
        }
    }
}
