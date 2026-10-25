//! The films in the standalone program: `gfx::play_film` (0x41AE39), the platform's in
//! the game (DirectShow's multimedia stream, `IAMMultiMediaStream`, into a DirectDraw
//! surface), here an AVI reader of its own and FFmpeg's decoder (`libavcodec`, built by
//! `tools/build_ffmpeg.sh` with the Indeo 4 decoder only; owner, 2026-10-19).
//!
//! What the original does (`sub_41AE39`, its three helpers `sub_41AE92` open,
//! `sub_41B0A4` a frame, `sub_41B0DF` close):
//! - with `g_noFmv` set: nothing (the flags of `cmp g_noFmv, 0`);
//! - the reports (`dword_4F52AC`) off while it plays, a pending key thrown away
//!   (`input::take_key`);
//! - the file (eax) resolved (`files::resolve_path`) and looked for
//!   (`files::file_exists`), opened as a stream (failing: straight to the close);
//! - then each frame: the messages (`system::pump_messages`), the stream's next frame
//!   (its sample's `Update`, which waits for the frame's time; the end or a failure:
//!   to the close), stretched over the whole back surface (`Blt`), the caller's function
//!   (ebx, if any: every caller gives 0), shown (`gfx::show_frame`), and a key
//!   (`input::take_key`) ends it;
//! - the close, `test eax, eax` (eax: the key when one ended it, 0 at the film's end, the
//!   path when it did not open), the reports back.
//!
//! Here `play_film_now(path)` (its one caller `menu::play_films`, which gives no callback):
//! true at the film's end, false when a key ended it or it did not open (the original's
//! zero flag; `play_films` stops its round on false).
//!
//! Here: an AVI (`ALLOGO.AVI`: Indeo 4) by its own reader, its first video stream's chunks
//! in order (a chunk of no bytes shows the frame before again), at the AVI's time a frame;
//! or an MPEG-1 program stream (`MFLOGO.MPG`, `INTRO.MPG`) by its own demuxer, its video
//! (stream 0xE0, MPEG-1 video, cut into frames by FFmpeg's parser) at its sequence
//! header's rate and its sound (stream 0xC0, MPEG audio layer 2) decoded first and played
//! through an SDL audio stream from the first frame on (stopped by the key or the end).
//! Each frame converted from YUV (4:1:0 or 4:2:0, BT.601) to the back surface's format and
//! stretched over it (nearest, as `Blt`). Anything else counts as a film that did not open,
//! and says so in the log.

use crate::framework::{log, rva};
use crate::gfx::reports::REPORTS;
use crate::gfx::screen_state::DRAW_SURFACE;
use std::ffi::{CStr, c_char, c_void};

pub static HOOK: crate::framework::RawHook =
    crate::framework::RawHook::new("gfx::play_film", rva(0x41AE39));

/// IDA's address of the `nofmv` switch.
pub const NO_FMV_START: usize = 0x5BE870;

/// `g_noFmv` (`dword_5BE870`, `nofmv`, a `var\ral.cfg` setting): 1 = no films.
pub static NO_FMV: crate::session::Dword = crate::session::Dword::new(0);

/// Where the switch is (`moved::MOVED`'s home for IDA 0x5BE870).
pub fn no_fmv_home() -> usize {
    NO_FMV.addr()
}

// ---- FFmpeg (cdecl; only functions, and the frame's first fields, which are stable) ----

#[link(name = "avcodec")]
unsafe extern "C" {
    fn avcodec_find_decoder_by_name(name: *const c_char) -> *const c_void;
    fn avcodec_alloc_context3(codec: *const c_void) -> *mut c_void;
    fn avcodec_open2(context: *mut c_void, codec: *const c_void, options: *mut c_void) -> i32;
    fn avcodec_free_context(context: *mut *mut c_void);
    fn av_packet_alloc() -> *mut c_void;
    fn av_packet_free(packet: *mut *mut c_void);
    fn av_packet_from_data(packet: *mut c_void, data: *mut u8, size: i32) -> i32;
    fn av_packet_unref(packet: *mut c_void);
    fn avcodec_send_packet(context: *mut c_void, packet: *const c_void) -> i32;
    fn avcodec_receive_frame(context: *mut c_void, frame: *mut c_void) -> i32;
    fn av_parser_init(codec_id: i32) -> *mut c_void;
    #[allow(clippy::too_many_arguments)] // FFmpeg's
    fn av_parser_parse2(
        parser: *mut c_void,
        context: *mut c_void,
        out: *mut *mut u8,
        out_size: *mut i32,
        data: *const u8,
        size: i32,
        pts: i64,
        dts: i64,
        pos: i64,
    ) -> i32;
    fn av_parser_close(parser: *mut c_void);
}

#[link(name = "avutil")]
unsafe extern "C" {
    fn av_frame_alloc() -> *mut c_void;
    fn av_frame_free(frame: *mut *mut c_void);
    fn av_malloc(size: usize) -> *mut c_void;
    fn av_free(data: *mut c_void);
    fn av_get_pix_fmt_name(format: i32) -> *const c_char;
    fn av_get_sample_fmt_name(format: i32) -> *const c_char;
}

/// `AV_NOPTS_VALUE`.
const NO_TIME: i64 = i64::MIN;
/// `AV_CODEC_ID_MPEG1VIDEO`, `AV_CODEC_ID_MP2`.
const MPEG1VIDEO: i32 = 1;
const MP2: i32 = 0x15000;

/// `AV_INPUT_BUFFER_PADDING_SIZE`: zeroed bytes a packet's data must have after it.
const PADDING: usize = 64;

/// `AVFrame`'s first fields on a 32-bit program: `data[8]`, `linesize[8]`,
/// `extended_data`, `width`, `height`, `nb_samples`, `format`.
#[repr(C)]
struct FrameHead {
    data: [*const u8; 8],
    linesize: [i32; 8],
    extended_data: *const c_void,
    width: i32,
    height: i32,
    nb_samples: i32,
    format: i32,
}

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
    walk(b, 12, b.len(), &mut |id, data, at, len| match id {
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

/// Walks the chunks from `at` to `end`, going into `LIST`s; `f` gets each chunk's id, its
/// bytes, their place and length.
fn walk(b: &[u8], mut at: usize, end: usize, f: &mut Visit) -> Result<(), String> {
    while at + 8 <= end {
        let id: [u8; 4] = b[at..at + 4].try_into().unwrap();
        let len = le32(b, at + 4).unwrap() as usize;
        let data_at = at + 8;
        let data_end = data_at
            .checked_add(len)
            .filter(|&e| e <= b.len())
            .ok_or_else(|| format!("chunk {} past the file's end", String::from_utf8_lossy(&id)))?;
        if &id == b"LIST" || &id == b"RIFF" {
            walk(b, data_at + 4, data_end, f)?;
        } else {
            f(&id, &b[data_at..data_end], data_at, len);
        }
        at = data_end + (len & 1);
    }
    Ok(())
}

// ---- the decoder ----

/// A name FFmpeg gives a format, or "?".
unsafe fn name_of(name: *const c_char) -> String {
    if name.is_null() {
        "?".into()
    } else {
        unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() }
    }
}

/// A decoder of FFmpeg's, with its parser when the stream is not cut into frames.
struct Decoder {
    context: *mut c_void,
    packet: *mut c_void,
    frame: *mut c_void,
    parser: *mut c_void,
}

impl Decoder {
    fn open(name: &CStr, parser: Option<i32>) -> Result<Self, String> {
        unsafe {
            let decoder = avcodec_find_decoder_by_name(name.as_ptr());
            if decoder.is_null() {
                return Err(format!("FFmpeg has no {name:?}"));
            }
            let context = avcodec_alloc_context3(decoder);
            if context.is_null() || avcodec_open2(context, decoder, std::ptr::null_mut()) < 0 {
                let mut c = context;
                avcodec_free_context(&mut c);
                return Err(format!("{name:?} does not open"));
            }
            let parser = match parser {
                Some(id) => {
                    let p = av_parser_init(id);
                    if p.is_null() {
                        return Err(format!("FFmpeg has no parser for {name:?}"));
                    }
                    p
                }
                None => std::ptr::null_mut(),
            };
            Ok(Self {
                context,
                packet: av_packet_alloc(),
                frame: av_frame_alloc(),
                parser,
            })
        }
    }

    /// The stream cut into the decoder's packets (copies) by the parser.
    fn cut(&mut self, stream: &[u8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut at = 0;
        loop {
            let rest = &stream[at.min(stream.len())..];
            let (mut data, mut size) = (std::ptr::null_mut::<u8>(), 0i32);
            let used = unsafe {
                av_parser_parse2(
                    self.parser,
                    self.context,
                    &mut data,
                    &mut size,
                    if rest.is_empty() {
                        std::ptr::null()
                    } else {
                        rest.as_ptr()
                    },
                    rest.len() as i32,
                    NO_TIME,
                    NO_TIME,
                    0,
                )
            };
            if size > 0 && !data.is_null() {
                out.push(unsafe { std::slice::from_raw_parts(data, size as usize) }.to_vec());
            }
            if rest.is_empty() && size <= 0 {
                break;
            }
            at += used.max(0) as usize;
            if used <= 0 && !rest.is_empty() && size <= 0 {
                break;
            }
        }
        out
    }

    /// `bytes` (None: the end, to have the frames held back) decoded: `each` gets every
    /// frame that comes out.
    fn decode(
        &mut self,
        bytes: Option<&[u8]>,
        each: &mut dyn FnMut(&FrameHead) -> Result<(), String>,
    ) -> Result<(), String> {
        unsafe {
            let sent = match bytes {
                Some(bytes) => {
                    let data = av_malloc(bytes.len() + PADDING) as *mut u8;
                    if data.is_null() {
                        return Err("out of memory".into());
                    }
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), data, bytes.len());
                    std::ptr::write_bytes(data.add(bytes.len()), 0, PADDING);
                    if av_packet_from_data(self.packet, data, bytes.len() as i32) < 0 {
                        av_free(data as *mut c_void);
                        return Err("a packet that cannot be made".into());
                    }
                    let sent = avcodec_send_packet(self.context, self.packet);
                    av_packet_unref(self.packet);
                    sent
                }
                None => avcodec_send_packet(self.context, std::ptr::null()),
            };
            // a damaged packet is passed over, as a player does
            if sent < 0 && bytes.is_none() {
                return Ok(());
            }
            while avcodec_receive_frame(self.context, self.frame) >= 0 {
                each(&*(self.frame as *const FrameHead))?;
            }
            Ok(())
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            if !self.parser.is_null() {
                av_parser_close(self.parser);
            }
            av_frame_free(&mut self.frame);
            av_packet_free(&mut self.packet);
            avcodec_free_context(&mut self.context);
        }
    }
}

/// A decoded picture as 0xRRGGBB (BT.601, studio range), its width and height.
struct Picture {
    rgb: Vec<u32>,
    width: usize,
    height: usize,
}

/// A video frame converted.
unsafe fn picture(head: &FrameHead) -> Result<Picture, String> {
    let name = unsafe { name_of(av_get_pix_fmt_name(head.format)) };
    let (cx, cy) = match name.as_ref() {
        "yuv410p" => (4, 4),
        "yuv420p" => (2, 2),
        other => return Err(format!("a picture in {other}, which is not converted")),
    };
    let (width, height) = (head.width.max(0) as usize, head.height.max(0) as usize);
    let mut rgb = vec![0u32; width * height];
    let at = |plane: usize, px: usize, py: usize| unsafe {
        *head.data[plane].offset(py as isize * head.linesize[plane] as isize + px as isize) as i32
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
unsafe fn samples(head: &FrameHead, channels: usize, pcm: &mut Vec<i16>) -> Result<(), String> {
    let name = unsafe { name_of(av_get_sample_fmt_name(head.format)) };
    let n = head.nb_samples.max(0) as usize;
    for i in 0..n {
        for c in 0..channels {
            let v = unsafe {
                match name.as_ref() {
                    "s16p" => *(head.data[c] as *const i16).add(i),
                    "s16" => *(head.data[0] as *const i16).add(i * channels + c),
                    "fltp" => (*(head.data[c] as *const f32).add(i) * 32767.0)
                        .clamp(-32768.0, 32767.0) as i16,
                    other => return Err(format!("sound in {other}, which is not converted")),
                }
            };
            pcm.push(v);
        }
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

/// The picture stretched over the locked surface (`g_surface`).
unsafe fn blit(p: &Picture) {
    let (rgb, w, h) = (&p.rgb, p.width, p.height);
    unsafe {
        let surface = DRAW_SURFACE.x.addr() as *const u32;
        let at = |n: usize| surface.add(n).read_unaligned();
        let (sw, sh) = (at(2) as usize, at(3) as usize);
        let bytes = at(5) as usize;
        let pitch = at(9) as usize;
        let pixels = at(10) as *mut u8;
        let format = (crate::session::SCREEN.format_addr() + 4) as *const u32;
        // masks: blue, green, red, alpha
        let (mb, mg, mr) = (
            format.read_unaligned(),
            format.add(1).read_unaligned(),
            format.add(2).read_unaligned(),
        );
        if pixels.is_null() || sw == 0 || sh == 0 || w == 0 || h == 0 {
            return;
        }
        for y in 0..sh {
            let from = (y * h / sh) * w;
            let row = pixels.add(y * pitch);
            for x in 0..sw {
                let c = rgb[from + x * w / sw];
                let p = pack(c >> 16 & 0xFF, mr) | pack(c >> 8 & 0xFF, mg) | pack(c & 0xFF, mb);
                match bytes {
                    2 => (row.add(x * 2) as *mut u16).write_unaligned(p as u16),
                    4 => (row.add(x * 4) as *mut u32).write_unaligned(p),
                    _ => {}
                }
            }
        }
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
            Some((hz, channels)) if !audio.is_empty() => {
                let mut sound = Decoder::open(c"mp2", Some(MP2))?;
                let mut pcm = Vec::new();
                for packet in sound.cut(&audio) {
                    sound.decode(Some(&packet), &mut |f| unsafe {
                        samples(f, channels, &mut pcm)
                    })?;
                }
                sound.decode(None, &mut |f| unsafe { samples(f, channels, &mut pcm) })?;
                Some((pcm, hz, channels))
            }
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

/// The Rust version (see the header) for the path at `path`: the zero flag it leaves
/// (true: the film's end, false for a key or a film that does not play).
pub unsafe fn play_film(path: u32) -> bool {
    unsafe {
        let reports: u32 = REPORTS.on.get();
        REPORTS.on.set(0);
        // A key pending before the film is dropped (`input::take_key`).
        crate::input::take_key::take_key();
        // the open: the path resolved (`files::resolve_path`), there (`files::file_exists`)
        let found = crate::files::resolve_path::resolve_path_in_game(path as *const u8)
            .and_then(|at| Ok((at, crate::files::file_exists::file_exists(at)?)));
        let name = match found {
            Ok((at, there)) => {
                let name = CStr::from_ptr(at as *const c_char)
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
        // The original's eax: the path (the zero flag clear when it does not open), else
        // the key that ended the film (0 at its end).
        let mut eax = path;
        if let Some(name) = name {
            match open(&name) {
                Err(why) => log(&format!("gfx::play_film: {name} not played: {why}")),
                Ok(mut film) => {
                    log(&format!("gfx::play_film: {name}: {}", film.what));
                    eax = u32::from(run(&mut film));
                }
            }
        }
        REPORTS.on.set(reports);
        eax == 0
    }
}

/// The film's sound playing through SDL (stopped when dropped).
struct Sound(*mut sdl3_sys::everything::SDL_AudioStream);

impl Sound {
    fn start(pcm: &[i16], hz: u32, channels: usize) -> Option<Self> {
        use sdl3_sys::everything::*;
        unsafe {
            if !SDL_InitSubSystem(SDL_INIT_AUDIO) {
                return None;
            }
            let spec = SDL_AudioSpec {
                format: SDL_AUDIO_S16LE,
                channels: channels as i32,
                freq: hz as i32,
            };
            let stream = SDL_OpenAudioDeviceStream(
                SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK,
                &spec,
                None,
                std::ptr::null_mut(),
            );
            if stream.is_null() {
                return None;
            }
            SDL_PutAudioStreamData(stream, pcm.as_ptr().cast(), (pcm.len() * 2) as i32);
            SDL_FlushAudioStream(stream);
            SDL_ResumeAudioStreamDevice(stream);
            Some(Self(stream))
        }
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        unsafe { sdl3_sys::everything::SDL_DestroyAudioStream(self.0) };
    }
}

/// The frames until the end or a key: the key, or 0 at the end.
unsafe fn run(film: &mut Film) -> u8 {
    unsafe {
        let sound = film
            .sound
            .as_ref()
            .and_then(|(pcm, hz, ch)| Sound::start(pcm, *hz, *ch));
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
                        let result = film.decoder.decode(packet.map(|p| p.as_slice()), &mut |f| {
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
            crate::platform::pump_messages_now();
            // the frame's time, as the stream's Update waits for it
            let due = film.frame * shown;
            if let Some(wait) = due.checked_sub(start.elapsed()) {
                std::thread::sleep(wait);
            }
            shown += 1;
            // (`gfx::lock_surface` leaves the carry clear for a locked surface, 1.)
            if crate::platform::lock_surface() >= 1 {
                blit(&next);
                crate::platform::unlock_surface();
            }
            last = Some(next);
            crate::platform::show_frame_now();
            let key = crate::input::take_key::take_key();
            if key != 0 {
                drop(sound);
                return key;
            }
        }
        drop(sound);
        0
    }
}

/// `gfx::play_film` for its callers (the films' loop): nothing with the setting `nofmv`
/// (the zero flag clear); else the film at `path`, true at its end.
pub fn play_film_now(path: u32) -> bool {
    HOOK.count_call();
    if NO_FMV.get() != 0 {
        return false;
    }
    unsafe { play_film(path) }
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
}
