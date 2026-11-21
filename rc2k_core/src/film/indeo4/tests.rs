//! Tests of the Indeo 4 decoder.

use super::test_streams::STREAMS;
use super::*;

/// The picture packets of an AVI file, in order (the chunks `00dc` / `00db` of its `movi`
/// list), empty ones included.
fn avi_packets(file: &[u8]) -> Vec<&[u8]> {
    fn walk<'a>(bytes: &'a [u8], packets: &mut Vec<&'a [u8]>) {
        let mut at = 0;
        while let Some(head) = bytes.get(at..at + 8) {
            let size = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
            let Some(data) = bytes.get(at + 8..at + 8 + size) else {
                return;
            };
            match &head[..4] {
                b"RIFF" | b"LIST" => walk(data.get(4..).unwrap_or(&[]), packets),
                b"00dc" | b"00db" => packets.push(data),
                _ => {}
            }
            at += 8 + size + (size & 1);
        }
    }
    let mut packets = Vec::new();
    walk(file, &mut packets);
    packets
}

fn picture_bytes(picture: &Picture) -> Vec<u8> {
    picture.planes.concat()
}

/// ALLOGO.AVI under the folder `RC2K_FILM_FILES` names (a game's `files/fmv`) decoded and
/// compared byte for byte with FFmpeg's pictures under the folder `RC2K_FILM_DUMPS`:
/// `allogo_frames.yuv` (`ffmpeg -i ALLOGO.AVI -fps_mode passthrough -f rawvideo`: the
/// pictures the decoder gives, one after another) and `allogo_shown.yuv`
/// (`ffmpeg -i ALLOGO.AVI -f rawvideo`: a picture for every packet's time, the one before
/// again where a packet gives none). Skipped without both names.
#[test]
fn the_logo_film_decodes_as_ffmpeg_does() {
    let (Some(files), Some(dumps)) = (
        std::env::var_os("RC2K_FILM_FILES"),
        std::env::var_os("RC2K_FILM_DUMPS"),
    ) else {
        return;
    };
    let read = |folder: &std::ffi::OsString, name: &str| {
        let path = std::path::Path::new(folder).join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    };
    let film = read(&files, "ALLOGO.AVI");
    let frames = read(&dumps, "allogo_frames.yuv");
    let shown = read(&dumps, "allogo_shown.yuv");
    let packets = avi_packets(&film);
    assert!(!packets.is_empty());

    let mut decoder = Decoder::new();
    let mut given = Vec::new();
    let mut shown_ours = Vec::new();
    let mut last: Option<Vec<u8>> = None;
    for (number, packet) in packets.iter().enumerate() {
        let picture = if packet.is_empty() {
            None
        } else {
            decoder
                .decode(packet)
                .unwrap_or_else(|why| panic!("packet {number}: {why}"))
        };
        if let Some(picture) = &picture {
            assert_eq!((picture.width, picture.height), (320, 240));
            assert_eq!((picture.chroma_width, picture.chroma_height), (80, 60));
            given.push(picture_bytes(picture));
            last = Some(picture_bytes(picture));
        }
        if let Some(last) = &last {
            shown_ours.push(last.clone());
        }
    }
    let size = 320 * 240 + 2 * 80 * 60;
    assert_eq!(frames.len() % size, 0);
    assert_eq!(given.len(), frames.len() / size, "pictures given");
    let mut identical = 0;
    for (number, (ours, theirs)) in given.iter().zip(frames.chunks_exact(size)).enumerate() {
        assert!(ours == theirs, "picture {number} differs from FFmpeg's");
        identical += 1;
    }
    assert_eq!(shown_ours.len(), shown.len() / size, "pictures shown");
    for (number, (ours, theirs)) in shown_ours.iter().zip(shown.chunks_exact(size)).enumerate() {
        assert!(
            ours == theirs,
            "shown picture {number} differs from FFmpeg's"
        );
    }
    println!(
        "ALLOGO.AVI: {} packets, {identical} pictures identical to FFmpeg's, {} shown identical",
        packets.len(),
        shown_ours.len()
    );
}

fn fnv(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        *hash = (*hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01B3);
    }
}

/// A packet's first bits: the start code, the frame kind, and three clear flags.
fn header_start(kind: u32) -> Vec<u8> {
    (0x3FFF8u32 | kind << 18).to_le_bytes().to_vec()
}

#[test]
fn made_up_streams_decode_as_ffmpeg_does() {
    for (number, stream) in STREAMS.iter().enumerate() {
        let mut decoder = Decoder::new();
        let mut hash = 0xCBF2_9CE4_8422_2325u64;
        let mut pictures = 0;
        for packet in stream.packets {
            if let Ok(Some(picture)) = decoder.decode(packet) {
                assert_eq!(
                    (picture.width, picture.height),
                    (stream.width, stream.height)
                );
                for plane in &picture.planes {
                    fnv(&mut hash, plane);
                }
                pictures += 1;
            }
        }
        assert_eq!(pictures, stream.pictures, "stream {number}: pictures given");
        assert_eq!(hash, stream.hash, "stream {number}: the pictures' hash");
    }
}

#[test]
fn the_players_call_gives_each_picture_and_passes_over_damage() {
    let stream = &STREAMS[0];
    let mut decoder = Decoder::new();
    let mut pictures = 0;
    let mut count = |picture: &Picture| {
        assert_eq!(Picture::PIXEL_FORMAT_NAME, "yuv410p");
        assert_eq!(picture.line_size(0), picture.width());
        assert_eq!(picture.line_size(1), picture.width().div_ceil(4));
        assert_eq!(
            picture.plane(0, picture.height()).map(<[u8]>::len),
            Some(stream.width * stream.height)
        );
        let chroma_rows = picture.height().div_ceil(4);
        assert!(picture.plane(1, chroma_rows).is_some() && picture.plane(2, chroma_rows).is_some());
        assert!(picture.plane(1, chroma_rows + 1).is_none() && picture.plane(3, 1).is_none());
        pictures += 1;
        Ok(())
    };
    for packet in stream.packets {
        decoder.decode_each(Some(packet), &mut count).unwrap();
        // a damaged packet in between: no picture, no error
        decoder.decode_each(Some(&[0xFF; 9]), &mut count).unwrap();
    }
    decoder.decode_each(None, &mut count).unwrap();
    assert!(pictures > 0);
    // the callback's own failure is the call's
    let mut decoder = Decoder::new();
    let failed = decoder.decode_each(Some(stream.packets[0]), &mut |_| Err("no room".into()));
    assert_eq!(failed, Err("no room".to_string()));
}

#[test]
fn what_is_not_done_is_refused_by_name() {
    let mut decoder = Decoder::new();
    assert!(matches!(decoder.decode(&[]), Err(Refused::Damaged(_))));
    assert!(matches!(decoder.decode(&[0; 40]), Err(Refused::Damaged(_))));
    assert!(matches!(
        decoder.decode(&header_start(7)),
        Err(Refused::Damaged(_))
    ));
    assert!(matches!(
        decoder.decode(&header_start(3)),
        Err(Refused::NotImplemented(_))
    ));
    // an empty frame before any picture, of either kind
    assert!(matches!(
        decoder.decode(&header_start(5)),
        Err(Refused::Damaged(_))
    ));
    assert_eq!(decoder.decode(&header_start(6)), Ok(None));

    // Headers written out bit by bit: the start (24 bits, no key), a size by number or
    // in full, no tiles, the chroma format, then the split into bands.
    let header = |fields: &[(u32, u32)]| {
        let (mut bytes, mut at) = (header_start(0), 25usize);
        bytes.resize(40, 0);
        for &(value, count) in fields {
            for bit in 0..count {
                bytes[at >> 3] |= ((value >> bit & 1) as u8) << (at & 7);
                at += 1;
            }
        }
        bytes
    };
    // 320 x 240 with luma in four bands
    let four_bands = header(&[(1, 3), (0, 1), (0, 2), (2, 2), (0xFF, 8), (3, 2)]);
    assert_eq!(
        Decoder::new().decode(&four_bands),
        Err(Refused::NotImplemented("a picture in four luma bands"))
    );
    // a split that is none
    let no_split = header(&[(1, 3), (0, 1), (0, 2), (1, 2)]);
    assert!(matches!(
        Decoder::new().decode(&no_split),
        Err(Refused::Damaged(_))
    ));
    // 4096 x 4096, and a picture with no width
    let huge = header(&[
        (7, 3),
        (4096, 16),
        (4096, 16),
        (0, 1),
        (0, 2),
        (3, 2),
        (3, 2),
    ]);
    assert!(matches!(
        Decoder::new().decode(&huge),
        Err(Refused::NotImplemented(_))
    ));
    let flat = header(&[(7, 3), (16, 16), (0, 16), (0, 1), (0, 2), (3, 2), (3, 2)]);
    assert!(matches!(
        Decoder::new().decode(&flat),
        Err(Refused::Damaged(_))
    ));
    // another chroma format
    let chroma = header(&[(1, 3), (0, 1), (1, 2)]);
    assert!(matches!(
        Decoder::new().decode(&chroma),
        Err(Refused::Damaged(_))
    ));
    assert!(
        Refused::NotImplemented("a bidirectional frame")
            .to_string()
            .contains("not implemented")
    );
}

/// Every stream cut short at every length, and with bits flipped at random: whatever
/// comes out, nothing panics, and a decoder that met damage still decodes on.
#[test]
fn damaged_packets_are_refused_or_decoded_without_panic() {
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut random = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for stream in STREAMS {
        let mut decoder = Decoder::new();
        for packet in stream.packets {
            for length in (0..packet.len()).step_by(3) {
                let _ = decoder.decode(&packet[..length]);
            }
            let _ = decoder.decode(packet);
        }
        for _ in 0..40 {
            let mut decoder = Decoder::new();
            for packet in stream.packets {
                let mut damaged = packet.to_vec();
                for _ in 0..1 + random() % 4 {
                    let at = random() as usize % (damaged.len() * 8);
                    damaged[at >> 3] ^= 1 << (at & 7);
                }
                let _ = decoder.decode(&damaged);
            }
        }
    }
    // bytes at random behind a sound start code
    let mut decoder = Decoder::new();
    for _ in 0..2000 {
        let mut packet = header_start((random() % 5) as u32);
        packet[3] = random() as u8;
        packet.extend((0..random() % 200).map(|_| random() as u8));
        let _ = decoder.decode(&packet);
    }
}
