//! A buffered file, as the picture readers use it: the block the four stream functions take
//! in esi (`open_stream`, `close_stream`, `read_stream_byte`, `read_stream_block`; the two
//! that write are `files::write_stream_byte` and `files::write_stream_block`).

pub const BUFFER_LEN: usize = 0x2000;

/// The core's stream: the mode, the file, the position, the fill and the buffer.
pub type Stream = rc2k_core::files::io::Stream<u32>;

/// The one stream the picture readers and the screenshot use (the original's at
/// `g_pcx` + 0x18; Rust's since 2026-10-29, the block there no longer used).
static STREAM: std::sync::Mutex<Stream> = std::sync::Mutex::new(Stream {
    mode: 0,
    file: u32::MAX,
    position: 0,
    filled: 0,
    buffer: [0; BUFFER_LEN],
});

/// `f` on the stream (held meanwhile: the readers do not nest).
pub fn with_stream<R>(f: impl FnOnce(&mut Stream) -> R) -> R {
    f(&mut STREAM
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner))
}
