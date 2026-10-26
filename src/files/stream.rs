//! A buffered file, as the picture readers use it: the block the four stream functions take
//! in esi (`open_stream`, `close_stream`, `read_stream_byte`, `read_stream_block`; the two
//! that write are `files::write_stream_byte` and `files::write_stream_block`).

pub const BUFFER_LEN: usize = 0x2000;

#[repr(C)]
pub struct Stream {
    /// The mode it was opened with (1 = written to: closing writes the buffer out).
    pub mode: u32,
    pub file: u32,
    /// The next byte of the buffer to hand out, or to fill.
    pub position: u32,
    /// The bytes in the buffer.
    pub filled: u32,
    pub buffer: [u8; BUFFER_LEN],
}

// The core's stream is the same block.
const _: () = assert!(
    size_of::<Stream>() == size_of::<rc2k_core::files::io::Stream<u32>>()
        && BUFFER_LEN == rc2k_core::files::io::BUFFER_LEN
);

/// The game's stream block as the core's.
///
/// # Safety
/// `stream` must point at a whole stream block that nothing else uses meanwhile.
pub unsafe fn core_stream<'a>(stream: *mut Stream) -> &'a mut rc2k_core::files::io::Stream<u32> {
    unsafe { &mut *(stream as *mut rc2k_core::files::io::Stream<u32>) }
}

impl Stream {}

// ---- for the self-tests: a file of the test's own in the game's `var` folder ----
