//! The films: `gfx::play_film`, here an AVI reader of its own and FFmpeg's decoder (`libavcodec`,
//! built by `tools/build_ffmpeg.sh` with the Indeo 4 decoder only).
//!
//! A film: nothing with the setting `nofmv`; otherwise the reports (`Reports.on`) are off while it
//! plays and a pending key is thrown away (`input::take_key`). The path is resolved
//! (`files::resolve_path`), looked for (`files::file_exists`) and opened (failing: straight to the
//! close). Then each frame: the messages (`system::pump_messages`), the stream's next frame at its
//! time (the end or a failure: to the close), stretched over the whole back surface, shown
//! (`gfx::show_frame`), and a key (`input::take_key`) ends it. The close puts the reports back.
//!
//! `play_film_now(path)` (its one caller `menu::play_films`, which gives no callback) is true at
//! the film's end, false when a key ended it or it did not open (`play_films` stops its round on
//! false).
//!
//! An AVI (`ALLOGO.AVI`: Indeo 4) is read by its own reader, its first video stream's chunks in
//! order (a chunk of no bytes shows the frame before again), at the AVI's time a frame; an MPEG-1
//! program stream (`MFLOGO.MPG`, `INTRO.MPG`) by its own demuxer, its video (stream 0xE0, MPEG-1
//! video, cut into frames by FFmpeg's parser) at its sequence header's rate and its sound (stream
//! 0xC0, MPEG audio layer 2) decoded first and played through an SDL audio stream from the first
//! frame on (stopped by the key or the end). Each frame is converted from YUV (4:1:0 or 4:2:0,
//! BT.601) to the back surface's format and stretched over it (nearest). Anything else counts as a
//! film that did not open, and says so in the log.

use crate::ffi::ffmpeg::{Decoder, Frame, MP2, MPEG1VIDEO};
use crate::framework::log;
use crate::gfx::owner::Gfx;
use crate::system::owner::System;
use std::ffi::CStr;

/// 1 = no films.
pub static NO_FMV: crate::session::GameThread<crate::session::PlainDword> =
    crate::session::GameThread::new(crate::session::PlainDword::new(0));

// ---- the AVI ----

/// What the player needs of an AVI: the picture's size, the time a frame (microseconds),
/// the decoder's four characters and the video chunks (their bytes' place and length).
pub struct Avi {
    pub width: u32,
    pub height: u32,
    pub frame_us: u32,
    pub codec: [u8; 4],
    pub frames: Vec<(usize, usize)>,
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}

/// The AVI's header and its first video stream's chunks, in their order in `movi`.
pub fn parse_avi(b: &[u8]) -> Result<Avi, String> {
    if b.get(0..4) != Some(b"RIFF") || b.get(8..12) != Some(b"AVI ") {
        return Err("not an AVI (RIFF ... AVI)".into());
    }
    let mut avi = Avi {
        width: 0,
        height: 0,
        frame_us: 0,
        codec: [0; 4],
        frames: Vec::new(),
    };
    let mut video_stream: Option<u32> = None;
    let mut stream = 0u32;
    let mut scale_rate: Option<(u32, u32)> = None;
    walk(b, 12, b.len(), 0, &mut |id, data, at, len| match id {
        b"avih" => {
            avi.frame_us = le32(data, 0).unwrap_or(0);
            avi.width = le32(data, 32).unwrap_or(0);
            avi.height = le32(data, 36).unwrap_or(0);
        }
        b"strh" => {
            if data.get(0..4) == Some(b"vids") && video_stream.is_none() {
                video_stream = Some(stream);
                if let Some(h) = data.get(4..8) {
                    avi.codec.copy_from_slice(h);
                }
                if let (Some(scale), Some(rate)) = (le32(data, 20), le32(data, 24)) {
                    scale_rate = Some((scale, rate));
                }
            }
            stream += 1;
        }
        _ => {
            // `##dc` / `##db`: stream ##'s video
            if let Some(wanted) = video_stream
                && (&id[2..4] == b"dc" || &id[2..4] == b"db")
            {
                let number = std::str::from_utf8(&id[0..2])
                    .ok()
                    .and_then(|s| u32::from_str_radix(s, 16).ok());
                if number == Some(wanted) {
                    avi.frames.push((at, len));
                }
            }
        }
    })?;
    if avi.frame_us == 0
        && let Some((scale, rate)) = scale_rate
        && rate != 0
    {
        avi.frame_us = (scale as u64 * 1_000_000 / rate as u64) as u32;
    }
    if video_stream.is_none() || avi.frames.is_empty() {
        return Err("no video stream or no frames".into());
    }
    Ok(avi)
}

/// What `walk` calls for a chunk: its id, its bytes, their place and length.
type Visit<'a> = dyn FnMut(&[u8; 4], &[u8], usize, usize) + 'a;

/// The most lists inside one another an AVI may have (its own have 3).
const MOST_LIST_DEPTH: usize = 16;

/// Walks the chunks from `at` to `end`, `depth` lists down, going into `LIST`s; `f` gets each
/// chunk's id, its bytes, their place and length.
fn walk(b: &[u8], mut at: usize, end: usize, depth: usize, f: &mut Visit) -> Result<(), String> {
    if depth > MOST_LIST_DEPTH {
        return Err(format!("lists nested more than {MOST_LIST_DEPTH} deep"));
    }
    while at + 8 <= end {
        let id: [u8; 4] = b[at..at + 4].try_into().unwrap();
        let len = le32(b, at + 4).unwrap() as usize;
        let data_at = at + 8;
        let data_end = data_at
            .checked_add(len)
            .filter(|&e| e <= b.len())
            .ok_or_else(|| format!("chunk {} past the file's end", String::from_utf8_lossy(&id)))?;
        if &id == b"LIST" || &id == b"RIFF" {
            walk(b, data_at + 4, data_end, depth + 1, f)?;
        } else {
            f(&id, &b[data_at..data_end], data_at, len);
        }
        at = data_end + (len & 1);
    }
    Ok(())
}

// ---- the decoder (FFmpeg's, `ffi::ffmpeg`) ----

/// A decoded picture as 0xRRGGBB (BT.601, studio range), its width and height.
struct Picture {
    rgb: Vec<u32>,
    width: usize,
    height: usize,
}

/// A video frame converted.
fn picture(head: &Frame) -> Result<Picture, String> {
    let name = head.pixel_format_name();
    let (cx, cy) = match name.as_ref() {
        "yuv410p" => (4, 4),
        "yuv420p" => (2, 2),
        other => return Err(format!("a picture in {other}, which is not converted")),
    };
    let (width, height) = (head.width().max(0) as usize, head.height().max(0) as usize);
    let mut rgb = vec![0u32; width * height];
    let planes = [
        head.plane(0, height),
        head.plane(1, height.div_ceil(cy)),
        head.plane(2, height.div_ceil(cy)),
    ];
    let [Some(_), Some(_), Some(_)] = planes else {
        return Err("a picture without its three planes".into());
    };
    let at = |plane: usize, px: usize, py: usize| -> i32 {
        let line = head.line_size(plane) as usize;
        planes[plane].map_or(0, |b| b.get(py * line + px).copied().unwrap_or(0) as i32)
    };
    for y in 0..height {
        for x in 0..width {
            let l = at(0, x, y) - 16;
            let u = at(1, x / cx, y / cy) - 128;
            let v = at(2, x / cx, y / cy) - 128;
            let c = |n: i32| ((n + 128) >> 8).clamp(0, 255) as u32;
            let r = c(298 * l + 409 * v);
            let g = c(298 * l - 100 * u - 208 * v);
            let b = c(298 * l + 516 * u);
            rgb[y * width + x] = r << 16 | g << 8 | b;
        }
    }
    Ok(Picture { rgb, width, height })
}

/// An audio frame as 16-bit samples, interleaved, appended to `pcm` (`channels` of them).
fn samples(head: &Frame, channels: usize, pcm: &mut Vec<i16>) -> Result<(), String> {
    let name = head.sample_format_name();
    let n = head.sample_count().max(0) as usize;
    let missing = || format!("sound in {name} without its samples");
    match name.as_ref() {
        "s16" => pcm.extend_from_slice(head.values::<i16>(0, n * channels).ok_or_else(missing)?),
        "s16p" | "fltp" => {
            let planes = (0..channels)
                .map(|c| match name.as_ref() {
                    "s16p" => head.values::<i16>(c, n).map(<[i16]>::to_vec),
                    _ => head.values::<f32>(c, n).map(|p| {
                        p.iter()
                            .map(|&v| (v * 32767.0).clamp(-32768.0, 32767.0) as i16)
                            .collect()
                    }),
                })
                .collect::<Option<Vec<Vec<i16>>>>()
                .ok_or_else(missing)?;
            for i in 0..n {
                pcm.extend(planes.iter().map(|p| p[i]));
            }
        }
        other => return Err(format!("sound in {other}, which is not converted")),
    }
    Ok(())
}

// ---- the MPEG program stream (MPEG-1 system stream) ----

/// Its video stream (0xE0) and its audio stream (0xC0), each as one run of bytes.
pub fn demux_mpeg(b: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let (mut video, mut audio) = (Vec::new(), Vec::new());
    let mut at = 0usize;
    let word = |at: usize| {
        b.get(at..at + 4)
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
                // the pack header: 12 bytes in MPEG-1 ('0010'), else MPEG-2's 14 + stuffing
                let mpeg1 = b.get(at + 4).is_some_and(|&x| x >> 4 == 2);
                at += if mpeg1 {
                    12
                } else {
                    14 + (b.get(at + 13).copied().unwrap_or(0) & 7) as usize
                };
            }
            id @ 0xBB..=0xFF => {
                let len = b
                    .get(at + 4..at + 6)
                    .map(|x| u16::from_be_bytes([x[0], x[1]]) as usize)
                    .ok_or("a packet past the file's end")?;
                let end = (at + 6 + len).min(b.len());
                if matches!(id, 0xC0 | 0xE0) {
                    // MPEG-1's packet header: stuffing, the buffer's size, the times
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
        return Err("an MPEG stream without video".into());
    }
    Ok((video, audio))
}

/// The frames a second of an MPEG-1 video stream (its sequence header's code).
fn mpeg_frame_rate(video: &[u8]) -> f64 {
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

/// An MPEG audio stream's rate and channels (its first frame's header).
fn mpeg_audio_format(audio: &[u8]) -> Option<(u32, usize)> {
    let at = audio
        .windows(2)
        .position(|w| w[0] == 0xFF && w[1] & 0xE0 == 0xE0)?;
    let h = audio.get(at..at + 4)?;
    let mpeg1 = h[1] & 0x08 != 0;
    let rate = match (h[2] >> 2) & 3 {
        0 => 44100,
        1 => 48000,
        2 => 32000,
        _ => return None,
    };
    let rate = if mpeg1 { rate } else { rate / 2 };
    let channels = if h[3] >> 6 == 3 { 1 } else { 2 };
    Some((rate, channels))
}

// ---- the back surface ----

/// A channel of 0 .. 255 into its mask's bits.
fn pack(value: u32, mask: u32) -> u32 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let most = mask >> shift;
    ((value * most + 127) / 255) << shift
}

/// The picture stretched over the locked surface, a checked row at a time.
fn blit(p: &Picture, gfx: &Gfx, system: &System) {
    let (rgb, w, h) = (&p.rgb, p.width, p.height);
    let s = &gfx.drawing.draw_surface;
    let (sw, sh) = (s.width.get() as usize, s.height.get() as usize);
    let bytes = s.bytes_per_pixel.get() as usize;
    let pitch = s.pitch.get();
    let pixels = s.pixels.get();
    // masks: blue, green, red, alpha
    let mask = |n: u32| system.session.screen.get().format[1 + n as usize].get();
    let (mb, mg, mr) = (mask(0), mask(1), mask(2));
    if pixels == 0 || sw == 0 || sh == 0 || w == 0 || h == 0 || !matches!(bytes, 2 | 4) {
        return;
    }
    let rows = crate::gfx::rows::Rows::new(pixels, pitch, sh as u32);
    let mut row = vec![0u8; sw * bytes];
    for y in 0..sh {
        let from = (y * h / sh) * w;
        for x in 0..sw {
            let c = rgb[from + x * w / sw];
            let p = pack(c >> 16 & 0xFF, mr) | pack(c >> 8 & 0xFF, mg) | pack(c & 0xFF, mb);
            row[x * bytes..(x + 1) * bytes].copy_from_slice(&p.to_le_bytes()[..bytes]);
        }
        rows.write(y as u32, 0, &row, "the film's picture")
            .unwrap_or_else(|e| crate::fatal::raise::end("gfx::play_film", e));
    }
}

// ---- the player ----

/// A film opened: its video's packets in order (an empty one: the frame before again),
/// the time a frame, its decoder; its sound decoded (16-bit, interleaved) with its rate
/// and channels.
struct Film {
    packets: Vec<Vec<u8>>,
    frame: std::time::Duration,
    decoder: Decoder,
    sound: Option<(Vec<i16>, u32, usize)>,
    what: String,
}

fn open(path: &str) -> Result<Film, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if bytes.starts_with(b"RIFF") {
        let avi = parse_avi(&bytes)?;
        let name: &CStr = match &avi.codec.map(|c| c.to_ascii_uppercase()) {
            b"IV41" => c"indeo4",
            other => {
                return Err(format!(
                    "no decoder built in for {}",
                    String::from_utf8_lossy(other)
                ));
            }
        };
        return Ok(Film {
            packets: avi
                .frames
                .iter()
                .map(|&(at, len)| bytes[at..at + len].to_vec())
                .collect(),
            frame: std::time::Duration::from_micros(avi.frame_us.max(1) as u64),
            decoder: Decoder::open(name, None)?,
            sound: None,
            what: format!(
                "AVI {} x {}, {}, {} us a frame",
                avi.width,
                avi.height,
                String::from_utf8_lossy(&avi.codec),
                avi.frame_us
            ),
        });
    }
    if bytes.starts_with(&[0, 0, 1, 0xBA]) {
        let (video, audio) = demux_mpeg(&bytes)?;
        let rate = mpeg_frame_rate(&video);
        let mut decoder = Decoder::open(c"mpeg1video", Some(MPEG1VIDEO))?;
        let packets = decoder.cut(&video);
        let sound = match mpeg_audio_format(&audio) {
            Some((hz, channels)) if !audio.is_empty() => match decoded_sound(&audio, channels) {
                Ok(pcm) => Some((pcm, hz, channels)),
                Err(why) => {
                    log(&format!(
                        "gfx::play_film: {path}: played without sound: {why}"
                    ));
                    None
                }
            },
            _ => None,
        };
        return Ok(Film {
            what: format!(
                "MPEG-1, {} packets, {rate:.3} a second, sound {}",
                packets.len(),
                match &sound {
                    Some((pcm, hz, ch)) => format!(
                        "{hz} Hz {ch} channels, {:.1} s",
                        pcm.len() as f64 / (*hz as f64 * *ch as f64)
                    ),
                    None => "none".into(),
                }
            ),
            packets,
            frame: std::time::Duration::from_secs_f64(1.0 / rate),
            decoder,
            sound,
        });
    }
    Err("neither an AVI nor an MPEG program stream".into())
}

/// An MPEG audio stream of `channels` decoded, 16-bit, interleaved.
fn decoded_sound(audio: &[u8], channels: usize) -> Result<Vec<i16>, String> {
    let mut sound = Decoder::open(c"mp2", Some(MP2))?;
    let mut pcm = Vec::new();
    for packet in sound.cut(audio) {
        sound.decode(Some(&packet), &mut |f| samples(f, channels, &mut pcm))?;
    }
    sound.decode(None, &mut |f| samples(f, channels, &mut pcm))?;
    Ok(pcm)
}

/// Plays the film at `path` (see the header): true at the film's end, false for a key or a film
/// that does not play.
pub fn play_film(o: crate::owners::Owners<'static>, path: &dyn crate::session::TextPlace) -> bool {
    let gfx_reports = &o.gfx.reports;
    let reports: u32 = gfx_reports.on.get();
    gfx_reports.on.set(0);
    // A key pending before the film is dropped (`input::take_key`).
    crate::input::take_key::take_key(o.input);
    // the open: the path resolved (`files::resolve_path`), there (`files::file_exists`)
    let found = path.text("a film's path").and_then(|given| {
        let resolved = crate::files::resolve_path::resolve(&given)?;
        let there = crate::files::file_exists::file_exists(&resolved)?;
        Ok((resolved, there))
    });
    let name = match found {
        Ok((text, there)) => {
            let name = crate::files::host_path(&text)
                .to_string_lossy()
                .into_owned();
            if !there {
                log(&format!("gfx::play_film: {name} is not there"));
            }
            there.then_some(name)
        }
        Err(why) => {
            log(&format!("gfx::play_film: the path refused: {}", why.0));
            None
        }
    };
    // Whether the film ended by itself (false: a key ended it, or it did not open).
    let mut ended = false;
    if let Some(name) = name {
        match open(&name) {
            Err(why) => log(&format!("gfx::play_film: {name} not played: {why}")),
            Ok(mut film) => {
                log(&format!("gfx::play_film: {name}: {}", film.what));
                ended = run(&mut film, o) == 0;
            }
        }
    }
    gfx_reports.on.set(reports);
    ended
}

/// The frames until the end or a key: the key, or 0 at the end.
fn run(film: &mut Film, o: crate::owners::Owners<'static>) -> u8 {
    let (gfx, system, input) = (o.gfx, o.system, o.input);
    let sound = film
        .sound
        .as_ref()
        .and_then(|(pcm, hz, ch)| crate::ffi::sdl::PlayedSound::start(pcm, *hz, *ch));
    let start = std::time::Instant::now();
    let mut shown = 0u32;
    let mut last: Option<Picture> = None;
    let mut queue: std::collections::VecDeque<Picture> = Default::default();
    let packets = std::mem::take(&mut film.packets);
    let mut packets = packets.iter().map(Some).chain(std::iter::once(None));
    let mut ended = false;
    loop {
        // the next picture: decoded until one comes out (an empty packet: the one
        // before again)
        while queue.is_empty() && !ended {
            match packets.next() {
                None => ended = true,
                Some(Some(packet)) if packet.is_empty() => {
                    if let Some(p) = &last {
                        queue.push_back(Picture {
                            rgb: p.rgb.clone(),
                            width: p.width,
                            height: p.height,
                        });
                    }
                }
                Some(packet) => {
                    let result =
                        film.decoder
                            .decode(packet.map(std::vec::Vec::as_slice), &mut |f| {
                                queue.push_back(picture(f)?);
                                Ok(())
                            });
                    if let Err(why) = result {
                        log(&format!("gfx::play_film: frame {shown}: {why}"));
                        ended = true;
                    }
                }
            }
        }
        let Some(next) = queue.pop_front() else {
            break;
        };
        crate::platform::pump_messages_now(o);
        // The window closed: the film ends (fixed bug: "Closing the window leaves the game
        // running").
        if system.session.flags.quitting.is_set() {
            break;
        }
        // the frame's time, as the stream's Update waits for it
        let due = film.frame * shown;
        if let Some(wait) = due.checked_sub(start.elapsed()) {
            std::thread::sleep(wait);
        }
        shown += 1;
        // (`gfx::lock_surface` leaves the carry clear for a locked surface, 1.)
        if crate::platform::lock_surface(gfx, system) >= 1 {
            blit(&next, gfx, system);
            crate::platform::unlock_surface(gfx);
        }
        last = Some(next);
        crate::platform::show_frame_now(o);
        let key = crate::input::take_key::take_key(input);
        if key != 0 {
            drop(sound);
            return key;
        }
    }
    drop(sound);
    0
}

/// `gfx::play_film` for its callers (the films' loop): nothing with the setting `nofmv`; else the
/// film at `path`, true at its end.
pub fn play_film_now(
    o: crate::owners::Owners<'static>,
    path: &dyn crate::session::TextPlace,
) -> bool {
    if NO_FMV.get().get() != 0 {
        return false;
    }
    play_film(o, path)
}

#[cfg(test)]
mod tests {
    #[test]
    fn walks_a_small_avi() {
        // RIFF AVI with hdrl/avih, strl/strh (vids IV41), movi with two 00dc chunks
        let mut b = Vec::new();
        let chunk = |id: &[u8], data: &[u8]| {
            let mut c = id.to_vec();
            c.extend((data.len() as u32).to_le_bytes());
            c.extend(data);
            if data.len() % 2 == 1 {
                c.push(0);
            }
            c
        };
        let list = |kind: &[u8], inner: Vec<u8>| {
            let mut d = kind.to_vec();
            d.extend(inner);
            chunk(b"LIST", &d)
        };
        let mut avih = vec![0u8; 56];
        avih[0..4].copy_from_slice(&66_666u32.to_le_bytes());
        avih[32..36].copy_from_slice(&320u32.to_le_bytes());
        avih[36..40].copy_from_slice(&240u32.to_le_bytes());
        let mut strh = vec![0u8; 56];
        strh[0..4].copy_from_slice(b"vids");
        strh[4..8].copy_from_slice(b"IV41");
        let hdrl = list(
            b"hdrl",
            [chunk(b"avih", &avih), list(b"strl", chunk(b"strh", &strh))].concat(),
        );
        let movi = list(
            b"movi",
            [chunk(b"00dc", &[1, 2, 3]), chunk(b"00dc", &[])].concat(),
        );
        let mut riff = b"AVI ".to_vec();
        riff.extend(hdrl);
        riff.extend(movi);
        b.extend(chunk(b"RIFF", &riff));
        let avi = super::parse_avi(&b).unwrap();
        assert_eq!((avi.width, avi.height, avi.frame_us), (320, 240, 66_666));
        assert_eq!(&avi.codec, b"IV41");
        assert_eq!(avi.frames.len(), 2);
        assert_eq!(avi.frames[1].1, 0);
    }

    /// `LIST`s inside one another `depth` times, around nothing.
    fn nested_lists(depth: usize) -> Vec<u8> {
        let mut inner = Vec::new();
        for _ in 0..depth {
            let mut list = b"LIST".to_vec();
            list.extend((inner.len() as u32 + 4).to_le_bytes());
            list.extend(b"rec ");
            list.extend(inner);
            inner = list;
        }
        let mut riff = b"RIFF".to_vec();
        riff.extend((inner.len() as u32 + 4).to_le_bytes());
        riff.extend(b"AVI ");
        riff.extend(inner);
        riff
    }

    #[test]
    fn refuses_lists_nested_too_deep() {
        let deep = nested_lists(super::MOST_LIST_DEPTH + 1);
        let why = super::parse_avi(&deep).err().unwrap();
        assert!(why.contains("nested"), "{why}");
        // At the limit the walk goes on (and finds no video).
        let at_limit = nested_lists(super::MOST_LIST_DEPTH);
        let why = super::parse_avi(&at_limit).err().unwrap();
        assert!(why.contains("no video"), "{why}");
        // Far too deep: refused, not a stack overflow.
        assert!(super::parse_avi(&nested_lists(1_000)).is_err());
    }
}
