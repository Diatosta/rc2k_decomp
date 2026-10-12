//! A buffered file, as the picture readers use it: the block the four stream functions take
//! in esi (`open_stream`, `close_stream`, `read_stream_byte`, `read_stream_block`; the two
//! that write, `sub_402D53` and `sub_402DC9`, are not ported).

use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::framework::{game_read, game_write};

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

impl Stream {
    /// For the self-tests: every field a recognisable filler.
    pub fn filler() -> Box<Stream> {
        Box::new(Stream {
            mode: 0xAAAA_AAAA,
            file: 0xAAAA_AAAA,
            position: 0xAAAA_AAAA,
            filled: 0xAAAA_AAAA,
            buffer: [0xAA; BUFFER_LEN],
        })
    }
}

// ---- for the self-tests: a file of the test's own in the game's `var` folder ----

pub const TEST_FILE: &str = "var\\rc2k_hook_stream_test.bin";
pub const TEST_PATH: &[u8] = b"var\\rc2k_hook_stream_test.bin\0";
pub const MISSING_PATH: &[u8] = b"var\\rc2k_hook_no_such_file.bin\0";

/// More than one buffer and not a whole number of them.
pub const TEST_LEN: usize = 0x4567;

pub fn test_content() -> Vec<u8> {
    (0..TEST_LEN as u32)
        .map(|n| (n.wrapping_mul(31) >> 3) as u8 ^ n as u8)
        .collect()
}

/// Writes the test file; false if it cannot be written.
pub fn make_test_file() -> bool {
    std::fs::write(TEST_FILE, test_content()).is_ok()
}

pub fn remove_test_file() {
    let _ = std::fs::remove_file(TEST_FILE);
}

/// `g_lastFilePath`, which opening and closing change, kept over a test.
pub struct LastPath(u32);

impl LastPath {
    pub fn save() -> Self {
        Self(unsafe { game_read(LAST_FILE_PATH_RVA) })
    }
    pub fn now() -> u32 {
        unsafe { game_read(LAST_FILE_PATH_RVA) }
    }
    pub fn restore(self) {
        unsafe { game_write(LAST_FILE_PATH_RVA, self.0) };
    }
}
