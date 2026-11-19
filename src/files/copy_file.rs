//! A file copied, its time with it: the source opened to read, the destination created, a
//! megabyte at a time until a read gives 0 bytes, then the source's last write time put on the
//! destination (to two seconds, as DOS time). A failed read, a short or failed write or a
//! failed time ends it; both files are closed (the destination first). Refuses what
//! `open_by_mode` refuses.

use crate::errors::BadInput;

/// The bytes moved by one read and write: large enough that a big file is a few system calls,
/// small enough to stay out of the way.
const CHUNK: usize = 0x10_0000;

thread_local! {
    /// The copy's buffer, kept between files (a copy runs on one thread).
    static BUFFER: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Whether it was all done.
/// The paths without their NULs.
pub fn copy_file(from: &[u8], to: &[u8]) -> Result<bool, BadInput> {
    BUFFER.with_borrow_mut(|buffer| {
        buffer.resize(CHUNK, 0);
        Ok(rc2k_core::files::io::copy_file(
            &mut crate::files::handle::GameFiles::new(),
            from,
            to,
            buffer,
        )?)
    })
}
