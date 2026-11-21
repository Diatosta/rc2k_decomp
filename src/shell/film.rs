//! The films: `gfx::play_film`, here an AVI reader of its own and the decoders of
//! `rc2k_core::film` (Indeo 4, MPEG-1 video, MPEG audio layer II).
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
//! video, cut into pictures by `mpeg1_video::cut`) at its sequence header's rate and its sound (stream
//! 0xC0, MPEG audio layer 2) decoded first and played through an SDL audio stream from the moment
//! the first picture is shown (stopped by the key or the end). The pictures follow the sound's
//! position (what the device has taken from the stream; the wall clock for a film without sound or
//! a device that does not start): a picture waits for its time, one more than two frames late is
//! not converted or shown. Each
//! frame is converted from YUV (4:1:0 or 4:2:0, BT.601) to the back surface's format and stretched
//! over it (nearest). Anything else counts as a film that did not open, and says so in the log.

use crate::gfx::owner::Gfx;
use crate::program_log::log;
use crate::system::owner::System;
use rc2k_core::film::{indeo4, mpeg_film, mpeg1_video};

// ---- the AVI ----

/// What the player needs of an AVI: the picture's size, the time a frame (microseconds),
/// the decoder's four characters and the video chunks (their bytes' place and length).
struct Avi {
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
fn parse_avi(b: &[u8]) -> Result<Avi, String> {
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
        // A corrupt header's huge scale must not wrap round to a tiny (or zero) time a frame.
        avi.frame_us =
            u32::try_from(u64::from(scale) * 1_000_000 / u64::from(rate)).unwrap_or(u32::MAX);
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

// ---- the decoder (`rc2k_core::film`) ----

/// A decoded picture as 0xRRGGBB (BT.601, studio range), its width and height.
struct Picture {
    rgb: Vec<u32>,
    width: usize,
    height: usize,
}

/// A decoded picture's planes as the player reads them: the size, how many pixels share one
/// chroma sample across and down, and each plane's bytes with the bytes from one row to the next.
struct Yuv<'a> {
    width: usize,
    height: usize,
    shared: (usize, usize),
    planes: [(&'a [u8], usize); 3],
}

impl<'a> Yuv<'a> {
    fn of_mpeg(p: &'a mpeg1_video::Picture) -> Self {
        Self {
            width: p.width,
            height: p.height,
            shared: (2, 2),
            planes: [p.plane(0), p.plane(1), p.plane(2)],
        }
    }

    fn of_indeo(p: &'a indeo4::Picture) -> Self {
        let plane = |n: usize| (p.planes[n].as_slice(), p.line_size(n));
        Self {
            width: p.width(),
            height: p.height(),
            shared: (4, 4),
            planes: [plane(0), plane(1), plane(2)],
        }
    }
}

/// A video frame converted.
fn picture(yuv: &Yuv) -> Result<Picture, String> {
    let (cx, cy) = yuv.shared;
    let (width, height) = (yuv.width, yuv.height);
    let mut rgb = vec![0u32; width * height];
    let at = |plane: usize, px: usize, py: usize| -> i32 {
        let (bytes, line) = yuv.planes[plane];
        bytes.get(py * line + px).copied().unwrap_or(0) as i32
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

/// A film's video decoder: MPEG-1 for the program streams, Indeo 4 for the AVI.
enum FilmDecoder {
    Mpeg(Box<mpeg1_video::VideoDecoder>),
    Indeo(Box<indeo4::Decoder>),
}

impl FilmDecoder {
    /// A packet in (None at the film's end), `each` called for every picture that comes out.
    fn decode(
        &mut self,
        packet: Option<&[u8]>,
        each: &mut dyn FnMut(&Yuv) -> Result<(), String>,
    ) -> Result<(), String> {
        match self {
            Self::Mpeg(decoder) => match decoder.decode(packet.unwrap_or(&[])).map_err(|e| e.0)? {
                Some(p) => each(&Yuv::of_mpeg(p)),
                None => Ok(()),
            },
            Self::Indeo(decoder) => decoder.decode_each(packet, &mut |p| each(&Yuv::of_indeo(p))),
        }
    }
}

// ---- the back surface ----

/// A channel of 0 .. 255 into its mask's bits.
fn pack(value: u32, mask: u32) -> u32 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let most = u64::from(mask >> shift);
    // In 64 bits: a mask of more than 24 bits would overflow 32.
    ((((u64::from(value) * most + 127) / 255) as u32) << shift) & mask
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
    decoder: FilmDecoder,
    sound: Option<(Vec<i16>, u32, usize)>,
    what: String,
    /// The file's path, for the notice of a film that breaks.
    file: String,
}

fn open(path: &str) -> Result<Film, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if bytes.starts_with(b"RIFF") {
        let avi = parse_avi(&bytes)?;
        match &avi.codec.map(|c| c.to_ascii_uppercase()) {
            b"IV41" => {}
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
                .map(|&(at, len)| bytes.get(at..at.checked_add(len)?).map(<[u8]>::to_vec))
                .collect::<Option<_>>()
                .ok_or("a frame past the end of the file")?,
            file: path.to_string(),
            frame: std::time::Duration::from_micros(avi.frame_us.max(1) as u64),
            decoder: FilmDecoder::Indeo(Box::new(indeo4::Decoder::new())),
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
        let (video, audio) = mpeg_film::demux(&bytes).map_err(|e| e.0)?;
        let rate = mpeg_film::frame_rate(&video);
        let packets: Vec<Vec<u8>> = mpeg1_video::cut(&video)
            .into_iter()
            .map(|range| video[range].to_vec())
            .collect();
        let sound = if audio.is_empty() {
            None
        } else {
            match mpeg_film::decoded_sound(&audio) {
                Some(s) => Some((s.pcm, s.sample_rate, s.channels)),
                None => {
                    log(&format!(
                        "gfx::play_film: {path}: played without sound: nothing decoded"
                    ));
                    None
                }
            }
        };
        return Ok(Film {
            file: path.to_string(),
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
            decoder: FilmDecoder::Mpeg(Box::new(mpeg1_video::VideoDecoder::new())),
            sound,
        });
    }
    Err("neither an AVI nor an MPEG program stream".into())
}

/// A film that is there and cannot be played is skipped with a line in the log only: the films
/// are the start-up ones, and a notice at every start for a copy without them helps no one.
fn bad_film(name: &str) {
    log(&format!("gfx::play_film: {name} skipped"));
}

/// Plays the film at `path` (see the header): true at the film's end, false for a key or a film
/// that does not play.
fn play_film(
    o: crate::owners::Owners<'static>,
    path: &dyn crate::memory::session::TextPlace,
) -> bool {
    // A film's frames are menu-sized but shown with black bars, not the menus' wash.
    struct Shown;
    impl Drop for Shown {
        fn drop(&mut self) {
            crate::render::wash::set_film_showing(false);
        }
    }
    crate::render::wash::set_film_showing(true);
    let _shown = Shown;
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
            Err(why) => {
                log(&format!("gfx::play_film: {name} not played: {why}"));
                bad_film(&name);
            }
            Ok(mut film) => {
                log(&format!("gfx::play_film: {name}: {}", film.what));
                ended = run(&mut film, o) == 0;
            }
        }
    }
    gfx_reports.on.set(reports);
    ended
}

/// A picture more than this many frames behind the film's clock is not converted or shown.
const DROP_AFTER_FRAMES: u32 = 2;

/// How long a film waits for its sound's device to start pulling before it runs on the wall clock.
const SOUND_START_WAIT: std::time::Duration = std::time::Duration::from_secs(1);

/// The film's clock: zero until the first picture is on the screen (the sound starts with it, so
/// a slow first decode delays both); then the sound's position once the device plays it (so
/// pictures follow what is heard), else the wall clock.
struct Clock<'a> {
    sound: Option<&'a crate::platform::ffi::sdl::PlayedSound>,
    /// When the first picture was shown.
    start: std::cell::Cell<Option<std::time::Instant>>,
}

impl Clock<'_> {
    /// Starts the clock and the sound (the first picture is on the screen).
    fn begin(&self) {
        if self.start.get().is_none() {
            self.start.set(Some(std::time::Instant::now()));
            if let Some(sound) = self.sound {
                sound.resume();
            }
        }
    }

    fn now(&self) -> std::time::Duration {
        let Some(start) = self.start.get() else {
            return std::time::Duration::ZERO;
        };
        let wall = start.elapsed();
        match self.sound {
            None => wall,
            Some(sound) => {
                let heard = sound.played();
                if !heard.is_zero() {
                    heard
                } else if wall < SOUND_START_WAIT {
                    std::time::Duration::ZERO
                } else {
                    wall
                }
            }
        }
    }
}

/// The frames until the end or a key: the key, or 0 at the end.
fn run(film: &mut Film, o: crate::owners::Owners<'static>) -> u8 {
    let (gfx, system, input) = (o.gfx, o.system, o.input);
    let sound = film.sound.as_ref().and_then(|(pcm, hz, ch)| {
        let volume = crate::platform::sound::FILM_VOLUME
            .load(std::sync::atomic::Ordering::Relaxed)
            .min(crate::platform::sound::FULL_VOLUME);
        let full = crate::platform::sound::FULL_VOLUME as i32;
        let scaled: Vec<i16> = pcm
            .iter()
            .map(|&s| (i32::from(s) * volume as i32 / full) as i16)
            .collect();
        crate::platform::ffi::sdl::PlayedSound::start(&scaled, *hz, *ch)
    });
    let clock = Clock {
        sound: sound.as_ref(),
        start: std::cell::Cell::new(None),
    };
    let frame = film.frame;
    // The number the next decoded picture has, and the next to show.
    let mut decoded = 0u32;
    let mut shown = 0u32;
    let mut last: Option<Picture> = None;
    // None: a picture that was late and not converted.
    let mut queue: std::collections::VecDeque<Option<Picture>> = Default::default();
    let packets = std::mem::take(&mut film.packets);
    let mut packets = packets.iter().map(Some).chain(std::iter::once(None));
    let mut ended = false;
    let late = |number: u32| clock.now() > frame * (number + DROP_AFTER_FRAMES);
    let key = loop {
        // the next picture: decoded until one comes out (an empty packet: the one
        // before again)
        while queue.is_empty() && !ended {
            match packets.next() {
                None => ended = true,
                Some(Some(packet)) if packet.is_empty() => {
                    if let Some(p) = &last {
                        let again = (!late(decoded)).then(|| Picture {
                            rgb: p.rgb.clone(),
                            width: p.width,
                            height: p.height,
                        });
                        queue.push_back(again);
                        decoded += 1;
                    }
                }
                Some(packet) => {
                    let result =
                        film.decoder
                            .decode(packet.map(std::vec::Vec::as_slice), &mut |f| {
                                let skip = late(decoded);
                                decoded += 1;
                                queue.push_back(if skip { None } else { Some(picture(f)?) });
                                Ok(())
                            });
                    if let Err(why) = result {
                        log(&format!("gfx::play_film: frame {shown}: {why}"));
                        bad_film(&film.file);
                        ended = true;
                    }
                }
            }
        }
        let Some(next) = queue.pop_front() else {
            break 0;
        };
        crate::shell::frame::pump_messages_now(o);
        // The window closed: the film ends (fixed bug: "Closing the window leaves the game
        // running").
        if system.session.flags.quitting.is_set() {
            break 0;
        }
        // the frame's time on the film's clock
        let due = frame * shown;
        if next.is_some() {
            while let Some(wait) = due.checked_sub(clock.now()) {
                if wait.is_zero() {
                    break;
                }
                std::thread::sleep(wait.min(std::time::Duration::from_millis(4)));
            }
        }
        shown += 1;
        let Some(next) = next else {
            continue;
        };
        // Only a surface that was locked (1 or more) is drawn on.
        if let Some(_locked) = crate::gfx::with_locked_surface::lock(gfx, system) {
            blit(&next, gfx, system);
        }
        last = Some(next);
        crate::shell::frame::show_frame_now(o);
        clock.begin();
        let key = crate::input::take_key::take_key(input);
        if key != 0 {
            break key;
        }
    };
    drop(sound);
    key
}

/// `gfx::play_film` for its callers (the films' loop): nothing with the setting `nofmv`; else the
/// film at `path`, true at its end.
pub fn play_film_now(
    o: crate::owners::Owners<'static>,
    path: &dyn crate::memory::session::TextPlace,
) -> bool {
    if o.system.machine().no_fmv.get() != 0 {
        return false;
    }
    play_film(o, path)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_channel_is_packed_into_its_mask_without_overflow() {
        assert_eq!(super::pack(255, 0xFF00), 0xFF00);
        assert_eq!(super::pack(0, 0xFF00), 0);
        assert_eq!(super::pack(255, 0xF800), 0xF800);
        assert_eq!(super::pack(128, 0x07E0), 0x0400);
        assert_eq!(super::pack(255, 0), 0);
        // A mask wider than any pixel format's (the product would pass 32 bits).
        assert_eq!(super::pack(255, 0xFFFF_FFFF), 0xFFFF_FFFF);
    }

    #[test]
    fn a_huge_frame_scale_is_not_wrapped_to_a_short_frame_time() {
        // No time a frame in avih; strh gives a scale of 0xFFFF_FFFF over a rate of 1.
        let mut avih = vec![0u8; 56];
        avih[32..36].copy_from_slice(&320u32.to_le_bytes());
        let mut strh = vec![0u8; 56];
        strh[0..4].copy_from_slice(b"vids");
        strh[4..8].copy_from_slice(b"IV41");
        strh[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
        strh[24..28].copy_from_slice(&1u32.to_le_bytes());
        let chunk = |id: &[u8], data: &[u8]| {
            let mut c = id.to_vec();
            c.extend((data.len() as u32).to_le_bytes());
            c.extend(data);
            c
        };
        let list = |kind: &[u8], inner: Vec<u8>| {
            let mut d = kind.to_vec();
            d.extend(inner);
            chunk(b"LIST", &d)
        };
        let hdrl = list(
            b"hdrl",
            [chunk(b"avih", &avih), list(b"strl", chunk(b"strh", &strh))].concat(),
        );
        let movi = list(b"movi", chunk(b"00dc", &[0, 0]));
        let mut riff = b"AVI ".to_vec();
        riff.extend(hdrl);
        riff.extend(movi);
        let file = chunk(b"RIFF", &riff);
        let avi = super::parse_avi(&file).unwrap();
        assert_eq!(avi.frame_us, u32::MAX);
    }

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
