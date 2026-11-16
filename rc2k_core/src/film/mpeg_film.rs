//! An MPEG-1 film (a program stream of MPEG-1 video and layer II sound) made ready for a
//! player: the two streams taken apart, the video cut into packets, the sound decoded.
//!
//! A player goes through `MpegFilm::video_packets` in order, gives each to
//! `VideoDecoder::decode` and shows the picture that comes out when one does, `frame_time`
//! apart; after the last packet an empty one brings the picture still held back:
//!
//! ```
//! # use rc2k_core::film::{mpeg_film::MpegFilm, mpeg1_video::VideoDecoder};
//! # fn show(_: &rc2k_core::film::mpeg1_video::Picture) {}
//! # fn play(bytes: &[u8]) -> Result<(), rc2k_core::film::FilmError> {
//! let film = MpegFilm::open(bytes)?;
//! let mut decoder = VideoDecoder::new();
//! for packet in film.video_packets().chain(std::iter::once(&[][..])) {
//!     // A refused packet may be passed over: the decoder goes on with the next.
//!     if let Some(picture) = decoder.decode(packet)? {
//!         show(picture);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use super::mp2::{self, AudioDecoder};
use super::mpeg_bits::{FilmError, refuse};
use super::mpeg1_video;

/// A film's sound, decoded: 16-bit samples, the channels interleaved.
pub struct Sound {
    pub pcm: Vec<i16>,
    pub sample_rate: u32,
    pub channels: usize,
}

/// A film opened.
pub struct MpegFilm {
    video: Vec<u8>,
    packets: Vec<std::ops::Range<usize>>,
    /// Pictures a second (the sequence header's; 25 when it names none).
    pub frame_rate: f64,
    pub sound: Option<Sound>,
}

impl MpegFilm {
    /// The film in `bytes` (a whole program stream). A film whose sound does not decode is
    /// opened without sound.
    pub fn open(bytes: &[u8]) -> Result<Self, FilmError> {
        let (video, audio) = demux(bytes)?;
        Ok(Self {
            packets: mpeg1_video::cut(&video),
            frame_rate: frame_rate(&video),
            video,
            sound: decoded_sound(&audio),
        })
    }

    /// The video's packets in decoding order.
    pub fn video_packets(&self) -> impl Iterator<Item = &[u8]> {
        self.packets.iter().map(|range| &self.video[range.clone()])
    }

    /// The time from one picture to the next.
    pub fn frame_time(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(1.0 / self.frame_rate)
    }
}

/// A program stream's first video stream (0xE0) and first audio stream (0xC0), each as one run
/// of bytes.
pub fn demux(b: &[u8]) -> Result<(Vec<u8>, Vec<u8>), FilmError> {
    let (mut video, mut audio) = (Vec::new(), Vec::new());
    let mut at = 0usize;
    let word = |at: usize| {
        b.get(at..at.checked_add(4)?)
            .map(|x| u32::from_be_bytes([x[0], x[1], x[2], x[3]]))
    };
    while let Some(code) = word(at) {
        if code >> 8 != 1 {
            // not at a start code: on to the next one
            at += 1;
            continue;
        }
        match code & 0xFF {
            0xB9 => break,
            0xBA => {
                // the pack header: 12 bytes in MPEG-1, else MPEG-2's 14 and its stuffing
                let mpeg1 = b.get(at + 4).is_some_and(|&x| x >> 4 == 2);
                at += if mpeg1 {
                    12
                } else {
                    14 + (b.get(at + 13).copied().unwrap_or(0) & 7) as usize
                };
            }
            id @ 0xBB..=0xFF => {
                let Some(len) = b
                    .get(at + 4..at + 6)
                    .map(|x| u16::from_be_bytes([x[0], x[1]]) as usize)
                else {
                    return refuse("a packet past the file's end");
                };
                let end = (at + 6 + len).min(b.len());
                if matches!(id, 0xC0 | 0xE0) {
                    // the packet header: stuffing, the buffer's size, the times
                    let mut p = at + 6;
                    while b.get(p) == Some(&0xFF) {
                        p += 1;
                    }
                    if b.get(p).is_some_and(|&x| x & 0xC0 == 0x40) {
                        p += 2;
                    }
                    p += match b.get(p).map(|&x| x & 0xF0) {
                        Some(0x20) => 5,
                        Some(0x30) => 10,
                        _ => 1,
                    };
                    let data = b.get(p.min(end)..end).unwrap_or(&[]);
                    if id == 0xE0 {
                        video.extend_from_slice(data);
                    } else {
                        audio.extend_from_slice(data);
                    }
                }
                at = end;
            }
            _ => at += 4,
        }
    }
    if video.is_empty() {
        return refuse("an MPEG stream without video");
    }
    Ok((video, audio))
}

/// The pictures a second of an MPEG-1 video stream (its first sequence header's code; 25 for
/// none or an unknown one).
pub fn frame_rate(video: &[u8]) -> f64 {
    let at = video.windows(4).position(|w| w == [0, 0, 1, 0xB3]);
    match at.and_then(|at| video.get(at + 7)).map(|&b| b & 0xF) {
        Some(1) => 24000.0 / 1001.0,
        Some(2) => 24.0,
        Some(3) => 25.0,
        Some(4) => 30000.0 / 1001.0,
        Some(5) => 30.0,
        Some(6) => 50.0,
        Some(7) => 60000.0 / 1001.0,
        Some(8) => 60.0,
        _ => 25.0,
    }
}

/// A layer II stream decoded whole. Packets that are refused, or whose rate or channels are
/// not the first frame's, are passed over; None when nothing decodes.
pub fn decoded_sound(audio: &[u8]) -> Option<Sound> {
    let mut decoder = AudioDecoder::new();
    let mut pcm = Vec::new();
    let mut format = None;
    for packet in mp2::cut(audio) {
        let before = pcm.len();
        match decoder.decode(&audio[packet], &mut pcm) {
            Ok(Some(frame)) => {
                let this = (frame.sample_rate, frame.channels);
                if *format.get_or_insert(this) != this {
                    pcm.truncate(before);
                }
            }
            Ok(None) | Err(_) => pcm.truncate(before),
        }
    }
    let (sample_rate, channels) = format?;
    Some(Sound {
        pcm,
        sample_rate,
        channels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::film::mpeg1_video::VideoDecoder;
    use std::io::Read;
    use std::path::Path;

    /// A program stream of one video packet and one audio packet.
    #[test]
    fn takes_the_two_streams_apart() {
        let mut stream = vec![0, 0, 1, 0xBA, 0x21, 0, 1, 0, 1, 0x80, 0, 1];
        // video: stuffing, a buffer size, no time stamp (0x0F), then the data
        stream.extend([0, 0, 1, 0xE0, 0, 8, 0xFF, 0x60, 0x20, 0x0F, 1, 2, 3, 4]);
        // audio: a presentation time (0x2x and four bytes), then the data
        stream.extend([0, 0, 1, 0xC0, 0, 7, 0x21, 0, 1, 0, 1, 9, 8]);
        stream.extend([0, 0, 1, 0xB9]);
        let (video, audio) = demux(&stream).unwrap();
        assert_eq!(video, [1, 2, 3, 4]);
        assert_eq!(audio, [9, 8]);
        assert!(demux(&[0, 0, 1, 0xBA]).is_err());
        assert!(demux(&[0, 0, 1, 0xE0, 0]).is_err());
    }

    /// One of the reference's lists: a packet's bytes and the frames it gave, line by line.
    fn listed(path: &Path) -> Vec<(usize, usize)> {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
            .lines()
            .map(|line| {
                let mut numbers = line.split(' ').map(|n| n.parse().unwrap());
                (numbers.next().unwrap(), numbers.next().unwrap())
            })
            .collect()
    }

    /// The two MPEG films of an installation decoded and compared, packet by packet, picture
    /// by picture and sample by sample, with what FFmpeg's decoders gave: `RC2K_FILM_FILES`
    /// names the folder with `INTRO.MPG` and `MFLOGO.MPG`, `RC2K_FILM_DUMPS` the folder with
    /// the reference made from them by `tools/film_reference/make.sh`. Without the two
    /// variables the test does nothing.
    #[test]
    fn the_real_films_decode_as_ffmpeg_decodes_them() {
        let (Some(files), Some(dumps)) = (
            std::env::var_os("RC2K_FILM_FILES"),
            std::env::var_os("RC2K_FILM_DUMPS"),
        ) else {
            return;
        };
        for name in ["MFLOGO", "INTRO"] {
            let bytes = std::fs::read(Path::new(&files).join(format!("{name}.MPG"))).unwrap();
            let dump = |ending: &str| Path::new(&dumps).join(format!("{name}.{ending}"));
            let (video, audio) = demux(&bytes).unwrap();

            // the sound
            let wanted_packets = listed(&dump("asizes"));
            let packets = mp2::cut(&audio);
            assert_eq!(
                packets
                    .iter()
                    .map(ExactSizeIterator::len)
                    .collect::<Vec<_>>(),
                wanted_packets.iter().map(|p| p.0).collect::<Vec<_>>(),
                "{name}: the sound's packets"
            );
            let started = std::time::Instant::now();
            let sound = decoded_sound(&audio).unwrap();
            let sound_time = started.elapsed().as_secs_f64();
            let wanted: Vec<i16> = std::fs::read(dump("s16"))
                .unwrap()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| i16::from_le_bytes(*b))
                .collect();
            assert_eq!(sound.pcm.len(), wanted.len(), "{name}: the samples' count");
            let different = sound
                .pcm
                .iter()
                .zip(&wanted)
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(different, 0, "{name}: samples that differ");

            // the pictures
            let wanted_packets = listed(&dump("vsizes"));
            let packets = mpeg1_video::cut(&video);
            assert_eq!(
                packets
                    .iter()
                    .map(ExactSizeIterator::len)
                    .collect::<Vec<_>>(),
                wanted_packets.iter().map(|p| p.0).collect::<Vec<_>>(),
                "{name}: the video's packets"
            );
            let mut reference = std::io::BufReader::new(std::fs::File::open(dump("yuv")).unwrap());
            let mut decoder = VideoDecoder::new();
            let mut pictures = 0usize;
            let mut wanted = Vec::new();
            let mut decoding = std::time::Duration::ZERO;
            for (number, packet) in packets
                .iter()
                .map(|p| &video[p.clone()])
                .chain(std::iter::once(&[][..]))
                .enumerate()
            {
                let started = std::time::Instant::now();
                let picture = decoder.decode(packet).unwrap();
                decoding += started.elapsed();
                if let Some(&(_, frames)) = wanted_packets.get(number) {
                    assert_eq!(
                        picture.is_some() as usize,
                        frames,
                        "{name}: whether packet {number} gives a picture"
                    );
                }
                let Some(picture) = picture else {
                    continue;
                };
                for plane in 0..3 {
                    let ours = picture.packed_plane(plane);
                    wanted.resize(ours.len(), 0);
                    reference.read_exact(&mut wanted).unwrap();
                    assert!(
                        ours == wanted,
                        "{name}: picture {pictures} (packet {number}), plane {plane} differs"
                    );
                }
                pictures += 1;
            }
            assert_eq!(
                reference.read(&mut [0]).unwrap(),
                0,
                "{name}: pictures missing"
            );
            assert_eq!(
                decoder.vectors_outside, 0,
                "{name}: vectors outside the picture"
            );
            let seconds =
                sound.pcm.len() as f64 / (sound.sample_rate as f64 * sound.channels as f64);
            println!(
                "{name}: {pictures} pictures and {} samples identical; {:.0} pictures a second, the sound {:.0} times as fast as it plays",
                sound.pcm.len(),
                pictures as f64 / decoding.as_secs_f64(),
                seconds / sound_time
            );
        }
    }
}
