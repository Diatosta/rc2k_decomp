//! FFmpeg (libavcodec 61 / libavutil 59, built by `tools/build_ffmpeg.sh` with the Indeo 4
//! decoder only) behind a safe interface, for the film player (`film.rs`).
//!
//! `Decoder` owns a codec context, a packet, a frame and optionally a parser, all made by
//! `open` and freed once in `drop`; every pointer FFmpeg hands back is checked for null
//! before use. `Frame` is the decoded frame FFmpeg lends for the length of one callback:
//! it can be had only from `Decoder::decode`, and its planes are slices that cannot
//! outlive that borrow. Nothing here keeps a raw pointer in a public signature.

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};

// FFmpeg's functions (cdecl; the frame's first fields, which are stable, are `Frame`).
// The declarations below are FFmpeg's (libavcodec 61 / libavutil 59), every pointer
// argument a C pointer of the type named in its header.

#[cfg(not(target_os = "emscripten"))]
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

#[cfg(not(target_os = "emscripten"))]
#[link(name = "avutil")]
unsafe extern "C" {
    fn av_frame_alloc() -> *mut c_void;
    fn av_frame_free(frame: *mut *mut c_void);
    fn av_malloc(size: usize) -> *mut c_void;
    fn av_free(data: *mut c_void);
    fn av_get_pix_fmt_name(format: i32) -> *const c_char;
    fn av_get_sample_fmt_name(format: i32) -> *const c_char;
}

// The web build has no FFmpeg (the films are left out of its first cut): no decoder is found,
// so `Decoder::open` fails and the film player skips the film, and nothing past it is reached.
#[cfg(target_os = "emscripten")]
mod none {
    #![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]
    use std::ffi::{c_char, c_void};
    use std::ptr::null_mut;
    pub unsafe fn avcodec_find_decoder_by_name(_: *const c_char) -> *const c_void {
        null_mut()
    }
    pub unsafe fn avcodec_alloc_context3(_: *const c_void) -> *mut c_void {
        null_mut()
    }
    pub unsafe fn avcodec_open2(_: *mut c_void, _: *const c_void, _: *mut c_void) -> i32 {
        -1
    }
    pub unsafe fn avcodec_free_context(_: *mut *mut c_void) {}
    pub unsafe fn av_packet_alloc() -> *mut c_void {
        null_mut()
    }
    pub unsafe fn av_packet_free(_: *mut *mut c_void) {}
    pub unsafe fn av_packet_from_data(_: *mut c_void, _: *mut u8, _: i32) -> i32 {
        -1
    }
    pub unsafe fn av_packet_unref(_: *mut c_void) {}
    pub unsafe fn avcodec_send_packet(_: *mut c_void, _: *const c_void) -> i32 {
        -1
    }
    pub unsafe fn avcodec_receive_frame(_: *mut c_void, _: *mut c_void) -> i32 {
        -1
    }
    pub unsafe fn av_parser_init(_: i32) -> *mut c_void {
        null_mut()
    }
    pub unsafe fn av_parser_parse2(
        _: *mut c_void,
        _: *mut c_void,
        _: *mut *mut u8,
        _: *mut i32,
        _: *const u8,
        _: i32,
        _: i64,
        _: i64,
        _: i64,
    ) -> i32 {
        -1
    }
    pub unsafe fn av_parser_close(_: *mut c_void) {}
    pub unsafe fn av_frame_alloc() -> *mut c_void {
        null_mut()
    }
    pub unsafe fn av_frame_free(_: *mut *mut c_void) {}
    pub unsafe fn av_malloc(_: usize) -> *mut c_void {
        null_mut()
    }
    pub unsafe fn av_free(_: *mut c_void) {}
    pub unsafe fn av_get_pix_fmt_name(_: i32) -> *const c_char {
        std::ptr::null()
    }
    pub unsafe fn av_get_sample_fmt_name(_: i32) -> *const c_char {
        std::ptr::null()
    }
}
#[cfg(target_os = "emscripten")]
use none::*;

/// `AV_NOPTS_VALUE`.
const NO_TIME: i64 = i64::MIN;
/// `AV_CODEC_ID_MPEG1VIDEO`, `AV_CODEC_ID_MP2`: the parsers `Decoder::open` can start.
pub const MPEG1VIDEO: i32 = 1;
pub const MP2: i32 = 0x15000;

/// `AV_INPUT_BUFFER_PADDING_SIZE`: zeroed bytes a packet's data must have after it.
const PADDING: usize = 64;

/// `AVFrame`'s first fields on a 32-bit program: `data[8]`, `linesize[8]`,
/// `extended_data`, `width`, `height`, `nb_samples`, `format`. Only `Decoder::decode` hands
/// one out, as a borrow of FFmpeg's own frame.
#[repr(C)]
pub struct Frame {
    data: [*const u8; 8],
    linesize: [i32; 8],
    extended_data: *const c_void,
    width: i32,
    height: i32,
    nb_samples: i32,
    format: i32,
}

/// A name FFmpeg gives a format (its static text), or "?".
fn name_of(name: *const c_char) -> String {
    if name.is_null() {
        "?".into()
    } else {
        // SAFETY: FFmpeg's format names are static NUL-ended texts.
        unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() }
    }
}

impl Frame {
    /// The picture's width in pixels (as FFmpeg gave it, possibly negative or 0).
    pub fn width(&self) -> i32 {
        self.width
    }

    /// The picture's height in pixels.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// The sound frame's samples a channel.
    pub fn sample_count(&self) -> i32 {
        self.nb_samples
    }

    /// Bytes a row of plane `p` takes in the frame's buffer.
    pub fn line_size(&self, p: usize) -> i32 {
        self.linesize[p]
    }

    /// The picture format's name.
    pub fn pixel_format_name(&self) -> String {
        // SAFETY: a lookup by number; FFmpeg answers null for an unknown one.
        name_of(unsafe { av_get_pix_fmt_name(self.format) })
    }

    /// The sound format's name.
    pub fn sample_format_name(&self) -> String {
        // SAFETY: as `pixel_format_name`.
        name_of(unsafe { av_get_sample_fmt_name(self.format) })
    }

    /// Plane `p`'s `rows` rows of `linesize` bytes; None for a plane without rows (or one
    /// asking for more rows than the picture is high).
    pub fn plane(&self, p: usize, rows: usize) -> Option<&[u8]> {
        let line = usize::try_from(self.linesize[p]).ok()?;
        if self.data[p].is_null() || line == 0 || rows > usize::try_from(self.height).ok()? {
            return None;
        }
        // SAFETY: a frame `Decoder::decode` hands on is FFmpeg's, decoded: each plane holds
        // its rows of `linesize` bytes (at most `height` of them, checked above), for as
        // long as the frame is borrowed.
        Some(unsafe { std::slice::from_raw_parts(self.data[p], line * rows) })
    }

    /// Plane `p` as `count` values of `T` (sound samples); None for an empty or
    /// misaligned plane, or one smaller than that.
    pub fn values<T: Copy>(&self, p: usize, count: usize) -> Option<&[T]> {
        let at = self.data[p].cast::<T>();
        if at.is_null() || !at.is_aligned() || !sound_plane_holds::<T>(self.linesize[0], count) {
            return None;
        }
        // SAFETY: as `plane`: each sound plane holds `linesize[0]` bytes, checked above.
        Some(unsafe { std::slice::from_raw_parts(at, count) })
    }
}

/// Whether a sound plane of `linesize` bytes (FFmpeg gives every sound plane the size of the
/// first) holds `count` values of `T`.
fn sound_plane_holds<T>(linesize: i32, count: usize) -> bool {
    let room = usize::try_from(linesize).unwrap_or(0);
    count
        .checked_mul(size_of::<T>())
        .is_some_and(|bytes| bytes <= room)
}

/// A decoder of FFmpeg's, with its parser when the stream is not cut into frames.
pub struct Decoder {
    context: *mut c_void,
    packet: *mut c_void,
    frame: *mut c_void,
    parser: *mut c_void,
}

impl Decoder {
    pub fn open(name: &CStr, parser: Option<i32>) -> Result<Self, String> {
        // SAFETY: a NUL-ended name; each object FFmpeg hands back is checked for null before
        // use, and freed by its own function (here on failure, else in `drop`).
        unsafe {
            let decoder = avcodec_find_decoder_by_name(name.as_ptr());
            if decoder.is_null() {
                return Err(format!("FFmpeg has no {name:?}"));
            }
            let context = avcodec_alloc_context3(decoder);
            if context.is_null() || avcodec_open2(context, decoder, std::ptr::null_mut()) < 0 {
                let mut c = context;
                avcodec_free_context(&raw mut c);
                return Err(format!("{name:?} does not open"));
            }
            // Made whole before anything more can fail: `drop` frees what is there.
            let mut made = Self {
                context,
                packet: av_packet_alloc(),
                frame: av_frame_alloc(),
                parser: std::ptr::null_mut(),
            };
            if made.packet.is_null() || made.frame.is_null() {
                return Err(format!("{name:?}: out of memory"));
            }
            if let Some(id) = parser {
                made.parser = av_parser_init(id);
                if made.parser.is_null() {
                    return Err(format!("FFmpeg has no parser for {name:?}"));
                }
            }
            Ok(made)
        }
    }

    /// The stream cut into the decoder's packets (copies) by the parser.
    pub fn cut(&mut self, stream: &[u8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut at = 0;
        loop {
            let rest = &stream[at.min(stream.len())..];
            let (mut data, mut size) = (std::ptr::null_mut::<u8>(), 0i32);
            // SAFETY: the decoder's parser and context (open); `rest`'s bytes and length;
            // `data` / `size` written by FFmpeg.
            let used = unsafe {
                av_parser_parse2(
                    self.parser,
                    self.context,
                    &raw mut data,
                    &raw mut size,
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
                // SAFETY: the parser's `size` bytes at `data`, valid until its next call.
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
    pub fn decode(
        &mut self,
        bytes: Option<&[u8]>,
        each: &mut dyn FnMut(&Frame) -> Result<(), String>,
    ) -> Result<(), String> {
        // SAFETY: the packet's data is FFmpeg's own allocation, `PADDING` zero bytes past
        // the copy as it asks, owned by the packet once made; the frame FFmpeg fills is laid
        // out as `Frame` (its first fields) and lent to `each` only until the next one.
        unsafe {
            let sent = match bytes {
                Some(bytes) => {
                    let data = av_malloc(bytes.len() + PADDING).cast::<u8>();
                    if data.is_null() {
                        return Err("out of memory".into());
                    }
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), data, bytes.len());
                    std::ptr::write_bytes(data.add(bytes.len()), 0, PADDING);
                    if av_packet_from_data(self.packet, data, bytes.len() as i32) < 0 {
                        av_free(data.cast::<c_void>());
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
                // The frame's head as FFmpeg laid it out (`Frame`), borrowed for `each`.
                each(&*(self.frame as *const Frame))?;
            }
            Ok(())
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: the objects `open` made, each freed once by its own function (the frame's,
        // the packet's and the context's take a null one).
        unsafe {
            if !self.parser.is_null() {
                av_parser_close(self.parser);
            }
            av_frame_free(&raw mut self.frame);
            av_packet_free(&raw mut self.packet);
            avcodec_free_context(&raw mut self.context);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_sound_plane_must_hold_the_samples() {
        assert!(super::sound_plane_holds::<i16>(2304, 1152));
        assert!(!super::sound_plane_holds::<i16>(2304, 1153));
        assert!(super::sound_plane_holds::<f32>(4608, 1152));
        assert!(!super::sound_plane_holds::<f32>(2304, 1152));
        assert!(!super::sound_plane_holds::<i16>(-1, 1));
        assert!(!super::sound_plane_holds::<i16>(i32::MAX, usize::MAX));
        assert!(super::sound_plane_holds::<i16>(0, 0));
    }
}
