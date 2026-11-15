//! Creates a file (replacing one that exists), writes a buffer to it, closes it. It fails if the
//! open fails, if the write fails or writes less than asked, or if the close fails. It does not
//! create missing folders.

use crate::errors::BadInput;

use crate::files::handle::GameFiles;
pub use rc2k_core::files::io::WriteResult;

/// The file written from `data`. Refuses what `resolve_path` refuses.
pub fn write_file_from(path: &[u8], data: &[u8]) -> Result<WriteResult, BadInput> {
    Ok(rc2k_core::files::io::write_file(
        &mut GameFiles::new(),
        path,
        data,
    )?)
}

/// `write_file` from the first `count` bytes of a store's field.
pub fn write_file_from_store(
    path: &[u8],
    from: &dyn crate::memory::cells::Store,
    count: u32,
) -> Result<WriteResult, BadInput> {
    let mut data = vec![0u8; count as usize];
    from.load(0, &mut data)?;
    write_file_from(path, &data)
}
