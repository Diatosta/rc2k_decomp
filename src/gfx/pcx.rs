//! A PCX file here: a header of 0x80 bytes (+4 .. +0xA the picture's left, top, right and
//! bottom as words; +0x41 the number of planes, 1 or 3; +0x42 the bytes of one plane of one
//! row as a word), then the rows, run-length coded: a byte of 0xC0 or more is a count (less
//! 0xC0) for the byte after it, any other byte stands for itself. With one plane a byte is
//! a palette number and the palette, 0x300 bytes, is the end of the file.

use crate::files::stream::Stream;
use crate::session::{Bytes, Dword, with_runs};

/// IDA's address of the PCX decoding's variables.
pub const START: usize = 0x5195D4;
/// Their length.
pub const LEN: usize = 0x51B97C - START;

pub const HEADER_LEN: usize = 0x80;
pub const HEADER_PLANES: usize = 0x41;
pub const FILE_PALETTE_LEN: usize = 0x300;
/// The stream block's length (`files::stream::Stream`).
pub const STREAM_LEN: usize = core::mem::size_of::<Stream>();

/// The PCX file being decoded.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Pcx {
    /// `dword_5195D4`: the picture's width, as the header says.
    pub file_width: Dword,
    /// `dword_5195D8`: its height.
    pub file_height: Dword,
    /// `g_pcxLineBytes` (`dword_5195DC`).
    pub line_bytes: Dword,
    /// `g_pcxPlanes` (`dword_5195E0`).
    pub planes: Dword,
    /// `g_pictureStream` (`dword_5195E4`): the stream the readers opened the file as; not
    /// used since 2026-10-29 (`files::stream::with_stream`), kept for the block's layout.
    pub stream: Bytes<STREAM_LEN>,
    /// `g_pcxHeader` (`dword_51B5F4`): the header as read from the file, 0x80 bytes.
    pub header: Bytes<HEADER_LEN>,
    /// `unk_51B674`: the palette as read from the file, three bytes an entry.
    pub file_palette: Bytes<FILE_PALETTE_LEN>,
    /// `dword_51B974`: the widest picture that is taken.
    pub width_limit: Dword,
    /// `dword_51B978`: the highest picture that is taken.
    pub height_limit: Dword,
}

/// The PCX file's variables; the first values are the image's.
pub static PCX: Pcx = Pcx {
    file_width: Dword::new(0),
    file_height: Dword::new(0),
    line_bytes: Dword::new(0),
    planes: Dword::new(0),
    stream: Bytes::new(with_runs(&[(4, &[0xFF; 4])])),
    header: Bytes::new([0; HEADER_LEN]),
    file_palette: Bytes::new([0; FILE_PALETTE_LEN]),
    width_limit: Dword::new(0x320),
    height_limit: Dword::new(0x258),
};

/// The header zeroed and the file's first 0x80 bytes read into it (`files::read_file`):
/// whether the read worked.
pub fn read_header(path: &[u8]) -> Result<bool, crate::errors::BadInput> {
    let mut header = [0u8; HEADER_LEN];
    let ok = crate::files::read_file::read_file_into(path, &mut header, 0)?.ok;
    PCX.header.write(&header);
    Ok(ok)
}

/// The header read through `stream` (`files::read_stream_block`): whether it worked. The
/// bytes past a short read stay as they were.
pub fn read_stream_header(stream: &mut Stream) -> Result<bool, crate::errors::BadInput> {
    let mut header = PCX.header.read();
    let failed = crate::files::read_stream_block::read_stream_block(stream, &mut header)?.1;
    PCX.header.write(&header);
    Ok(!failed)
}

/// `path` opened as the stream (mode 0), `rows` run on it, the stream closed: whether all
/// three worked.
pub fn on_stream(
    path: &[u8],
    rows: impl FnOnce(&mut Stream) -> Result<bool, crate::errors::BadInput>,
) -> Result<bool, crate::errors::BadInput> {
    use crate::files::{close_stream::close_stream, open_stream::open_stream};
    crate::files::stream::with_stream(|stream| {
        if !open_stream(stream, path, 0)? {
            return Ok(false);
        }
        let rows = rows(stream);
        let closed = close_stream(stream)?;
        Ok(rows? && closed)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&PCX, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Pcx, stream), 0x5195E4 - START);
        assert_eq!(offset_of!(Pcx, header), 0x51B5F4 - START);
        assert_eq!(offset_of!(Pcx, file_palette), 0x51B674 - START);
        assert_eq!(offset_of!(Pcx, width_limit), 0x51B974 - START);
        assert_eq!(crate::moved::place(START), PCX.file_width.addr() as usize);
        assert_eq!(crate::moved::place(0x51B5F4), PCX.header.addr() as usize);
        assert_eq!(
            crate::moved::place(0x51B978),
            PCX.height_limit.addr() as usize
        );
    }
}
