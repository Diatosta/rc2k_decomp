//! The game's own software mixer: the playing channels mixed into a work buffer of left and right
//! `i32`s, then clipped into the sound buffer's format.
//!
//! A channel's frame is blended with the next by the position's fraction, as DirectSound
//! resamples (fixed: the game took the frame the position is in, whose steps gave the Dolby
//! mode's effects a harsh edge).

use crate::Invalid;

/// A sample format bit: eight bits a value (unsigned, 0x80 the middle), else sixteen.
pub(crate) const EIGHT_BITS: u32 = 0x10;
/// A sample format bit: one value a frame, else left and right.
pub(crate) const MONO: u32 = 0x100;
/// A channel's kind that goes back to its start at its end.
pub const LOOPING: u32 = 1;

/// What the mixer reads and changes of a channel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    /// 1 while it plays.
    pub used: u32,
    pub format: u32,
    /// Its length in frames.
    pub length: u32,
    pub kind: u32,
    /// The frame it is at, 32.32.
    pub position: u32,
    pub fraction: u32,
    /// What a frame of the mix moves it, 32.32.
    pub step: u32,
    pub step_fraction: u32,
    /// The volumes the sample is multiplied by.
    pub left: i32,
    pub right: i32,
}

/// The frame at `position` of a channel's data, (left, right) before the volumes (a mono sample is
/// both); `byte` reads the data at an offset (offsets wrap).
fn sample(format: u32, position: u32, byte: &impl Fn(u32) -> u8) -> (i32, i32) {
    let b = |at: u32| byte(at) as i32 - 0x80;
    let w = |at: u32| i16::from_le_bytes([byte(at), byte(at.wrapping_add(1))]) as i32;
    match (format & EIGHT_BITS != 0, format & MONO != 0) {
        (true, true) => {
            let s = b(position);
            (s, s)
        }
        (true, false) => {
            let at = position.wrapping_mul(2);
            (b(at), b(at.wrapping_add(1)))
        }
        (false, true) => {
            let s = w(position.wrapping_mul(2));
            (s, s)
        }
        (false, false) => {
            let at = position.wrapping_mul(4);
            (w(at), w(at.wrapping_add(2)))
        }
    }
}

/// The channel's frame blended with the next by its position's fraction (`a + (b - a) *
/// fraction / 2^32`, which stays between the two).
fn blended(c: &Channel, byte: &impl Fn(u32) -> u8) -> (i32, i32) {
    let (l0, r0) = sample(c.format, c.position, byte);
    if c.fraction == 0 {
        return (l0, r0);
    }
    let next = match c.position.wrapping_add(1) {
        n if n < c.length => n,
        _ if c.kind == LOOPING => 0,
        _ => c.position,
    };
    let (l1, r1) = sample(c.format, next, byte);
    let mix = |a: i32, b: i32| a + ((i64::from(b - a) * i64::from(c.fraction)) >> 32) as i32;
    (mix(l0, l1), mix(r0, r1))
}

/// `frames` frames of every playing channel added into `mix` (zeroed first, left and right a
/// frame); `data(n, offset)` is a byte of channel `n`'s sample data. A frame is blended with the
/// next (the first for a looping channel at its end, itself for any other) by the fraction. After
/// each frame a channel's position steps; at its length a looping channel goes back to 0 (the
/// fraction kept), any other stops playing. Refused (before anything): no channels or no frames
/// with one playing (the counts would run 2^32 times), more frames than `mix` holds.
pub fn mix_channels(
    frames: u32,
    channels: &mut [Channel],
    data: &impl Fn(usize, u32) -> u8,
    mix: &mut [i32],
) -> Result<(), Invalid> {
    if channels.is_empty() {
        return Err(Invalid(
            "no channels (the original's count runs 2^32 times)",
        ));
    }
    if frames as usize * 2 > mix.len() {
        return Err(Invalid("more frames than the mix buffer holds"));
    }
    if frames == 0 && channels.iter().any(|c| c.used == 1) {
        return Err(Invalid(
            "no frames to mix (the original's count runs 2^32 times)",
        ));
    }
    let mix = &mut mix[..frames as usize * 2];
    mix.fill(0);
    for (n, c) in channels.iter_mut().enumerate() {
        if c.used != 1 {
            continue;
        }
        let byte = |at: u32| data(n, at);
        for out in mix.as_chunks_mut::<2>().0 {
            let (l, r) = blended(c, &byte);
            out[0] = out[0].wrapping_add(l.wrapping_mul(c.left));
            out[1] = out[1].wrapping_add(r.wrapping_mul(c.right));
            let (fraction, carry) = c.fraction.overflowing_add(c.step_fraction);
            c.fraction = fraction;
            c.position = c.position.wrapping_add(c.step).wrapping_add(carry as u32);
            if c.position >= c.length {
                if c.kind == LOOPING {
                    c.position = 0;
                } else {
                    c.used = 0;
                    break;
                }
            }
        }
    }
    Ok(())
}

/// The bytes a frame of the sound buffer's format (`mode`'s `EIGHT_BITS` and `MONO`):
/// 1, 2 or 4 (sixteen bits are always stereo here).
pub fn frame_bytes(mode: u32) -> usize {
    match (mode & EIGHT_BITS != 0, mode & MONO != 0) {
        (false, _) => 4,
        (true, true) => 1,
        (true, false) => 2,
    }
}

/// The mix clipped into the sound buffer's parts (each as many whole frames as fit), in its format:
/// eight bits mono (left + right `>> 15`), eight bits stereo (each `>> 14`), held to -0x80 ..= 0x7F
/// and moved up by 0x80; else sixteen bits stereo (each `>> 14`, held to -0x8000 ..= 0x7FFF).
/// Refused (before anything): more frames than `mix` holds; sixteen bits with a right sample that
/// clips.
pub fn write_mix(mode: u32, mix: &[i32], parts: &mut [&mut [u8]]) -> Result<(), Invalid> {
    let size = frame_bytes(mode);
    let total: usize = parts.iter().map(|p| p.len() / size).sum();
    if total * 2 > mix.len() {
        return Err(Invalid("more frames than the mix buffer holds"));
    }
    if size == 4
        && mix[..total * 2]
            .as_chunks::<2>()
            .0
            .iter()
            .any(|f| !(-0x8000..=0x7FFF).contains(&(f[1] >> 14)))
    {
        return Err(Invalid(
            "a sixteen-bit right sample that clips (the original loops for ever)",
        ));
    }
    let byte = |v: i32| (v.clamp(-0x80, 0x7F) as u8).wrapping_add(0x80);
    let mut frames = mix.as_chunks::<2>().0.iter();
    for part in parts.iter_mut() {
        for out in part.chunks_exact_mut(size) {
            let f = frames.next().expect("counted above");
            let (l, r) = (f[0], f[1]);
            match size {
                4 => {
                    out[..2]
                        .copy_from_slice(&((l >> 14).clamp(-0x8000, 0x7FFF) as i16).to_le_bytes());
                    out[2..].copy_from_slice(&((r >> 14) as i16).to_le_bytes());
                }
                1 => out[0] = byte(l.wrapping_add(r) >> 15),
                _ => {
                    out[0] = byte(l >> 14);
                    out[1] = byte(r >> 14);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixes_and_writes() {
        // Two channels: a looping eight-bit mono one of 3 frames, a sixteen-bit stereo one
        // that ends after 2.
        let data: [Vec<u8>; 2] = [vec![0x90, 0x70, 0x80], vec![1, 0, 2, 0, 3, 0, 4, 0]];
        let mut channels = [
            Channel {
                used: 1,
                format: EIGHT_BITS | MONO,
                length: 3,
                kind: LOOPING,
                step: 1,
                left: 2,
                right: 3,
                ..Channel::default()
            },
            Channel {
                used: 1,
                length: 2,
                step: 1,
                left: 1,
                right: 1,
                ..Channel::default()
            },
        ];
        let mut mix = [7i32; 10];
        mix_channels(4, &mut channels, &|n, at| data[n][at as usize], &mut mix).unwrap();
        assert_eq!(
            &mix[..8],
            &[0x20 + 1, 0x30 + 2, -0x20 + 3, -0x30 + 4, 0, 0, 0x20, 0x30]
        );
        assert_eq!(mix[8], 7);
        assert_eq!((channels[0].position, channels[0].used), (1, 1));
        assert_eq!((channels[1].position, channels[1].used), (2, 0));
        // Eight bits stereo, split over two parts.
        let mix = [0x4000 * 3, -0x4000 * 200, 0, 0x4000];
        let (mut a, mut b) = ([0u8; 2], [0u8; 2]);
        write_mix(EIGHT_BITS, &mix, &mut [&mut a, &mut b]).unwrap();
        assert_eq!((a, b), ([0x83, 0x00], [0x80, 0x81]));
        // Sixteen bits: a right sample that clips is refused.
        let mut c = [0u8; 4];
        assert!(write_mix(0, &[0, 0x4000 * 0x9000], &mut [&mut c]).is_err());
        assert!(mix_channels(1, &mut [], &|_, _| 0, &mut [0; 2]).is_err());
    }

    #[test]
    fn blends_between_frames() {
        // Sixteen bits mono, 100 then 200, a half step: 100, 150, 200, then the end (not
        // looping) held at 200; looping, the end blends back towards the first.
        let data = [100i16, 200].map(i16::to_le_bytes).concat();
        let byte = |_: usize, at: u32| data[at as usize];
        let channel = |kind| Channel {
            used: 1,
            format: MONO,
            length: 2,
            kind,
            step_fraction: 0x8000_0000,
            left: 1,
            right: 1,
            ..Channel::default()
        };
        let mut c = [channel(0)];
        let mut mix = [0i32; 8];
        mix_channels(4, &mut c, &byte, &mut mix).unwrap();
        assert_eq!(mix, [100, 100, 150, 150, 200, 200, 200, 200]);
        let mut c = [channel(LOOPING)];
        mix_channels(4, &mut c, &byte, &mut mix).unwrap();
        assert_eq!(mix, [100, 100, 150, 150, 200, 200, 150, 150]);
    }

    /// FNV-1a over bytes, folded into `hash`.
    fn fnv(hash: u64, bytes: &[u8]) -> u64 {
        bytes.iter().fold(hash, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }

    const FROZEN_MIX: u64 = 2_630_935_495_345_432_356;

    /// Guards the game's own mixer (the Dolby mode's path): five channels of mixed formats (a
    /// looped one, a resampled one, one that ends part-way, a stopped one) mixed in an odd
    /// schedule of pulls, each pull clipped into all three buffer formats, give the same bytes
    /// and the same channel positions as before.
    #[test]
    fn the_game_mix_of_channels_is_frozen() {
        let pattern = |seed: u32, len: usize| -> Vec<u8> {
            let mut x = seed;
            (0..len)
                .map(|_| {
                    x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    (x >> 24) as u8
                })
                .collect()
        };
        let data = [
            pattern(1, 4000),
            pattern(2, 300),
            pattern(3, 1600),
            pattern(4, 2000),
            pattern(5, 800),
        ];
        let channel = |format, length, kind, step, step_fraction, left, right, used| Channel {
            used,
            format,
            length,
            kind,
            step,
            step_fraction,
            left,
            right,
            ..Channel::default()
        };
        let mut channels = [
            channel(0, 1000, LOOPING, 1, 0, 0x800, 0x800, 1),
            channel(EIGHT_BITS | MONO, 300, 0, 0, 0xC000_0000, 0xC00, 0x400, 1),
            channel(MONO, 800, LOOPING, 0, 0x5555_5555, 0x600, 0xA00, 1),
            channel(0, 500, 0, 1, 0x2000_0000, 0x200, 0xE00, 1),
            channel(0, 200, LOOPING, 1, 0, 0x800, 0x800, 0),
        ];
        let byte = |n: usize, at: u32| data[n][at as usize % data[n].len()];
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        let mut mix = [0i32; 0x3C00 / 4];
        let mut frames = 0u32;
        for (round, pull) in [1u32, 7, 64, 1023, 333, 1500, 5, 1200, 3]
            .into_iter()
            .enumerate()
        {
            mix_channels(pull, &mut channels, &byte, &mut mix).unwrap();
            frames += pull;
            let used = &mix[..pull as usize * 2];
            hash = fnv(
                hash,
                &used
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<u8>>(),
            );
            // Clipped into each format, the part sizes cut at odd places.
            for mode in [0, EIGHT_BITS, EIGHT_BITS | MONO] {
                let size = frame_bytes(mode);
                let total = pull as usize * size;
                let cut = (round * 3 + 1) % pull as usize * size;
                let mut out = vec![0xAAu8; total];
                let (a, b) = out.split_at_mut(cut);
                // A mono mix writes one value a frame from both: only the clipped sixteen-bit
                // right sample can be refused, and these volumes never reach it.
                write_mix(mode, used, &mut [a, b]).unwrap();
                hash = fnv(hash, &out);
            }
        }
        for c in &channels {
            hash = fnv(hash, &[c.used as u8]);
            hash = fnv(hash, &c.position.to_le_bytes());
            hash = fnv(hash, &c.fraction.to_le_bytes());
        }
        assert_eq!(frames, 4136);
        assert_eq!(hash, FROZEN_MIX);
    }
}
