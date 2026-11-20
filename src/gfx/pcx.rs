//! A PCX file: a header of 0x80 bytes (+4 .. +0xA the picture's left, top, right and bottom as
//! words; +0x41 the number of planes, 1 or 3; +0x42 the bytes of one plane of one row as a word),
//! then the rows, run-length coded: a byte of 0xC0 or more is a count (less 0xC0) for the byte
//! after it, any other byte stands for itself. With one plane a byte is a palette number and the
//! palette, 0x300 bytes, is the end of the file.

use crate::files::stream::Stream;
use crate::memory::session::{PlainBytes, PlainDword, with_runs};

pub const HEADER_LEN: usize = 0x80;
pub(super) const HEADER_PLANES: usize = 0x41;
pub(super) const FILE_PALETTE_LEN: usize = 0x300;
/// The stream block's length (`files::stream::Stream`).
const STREAM_LEN: usize = core::mem::size_of::<Stream>();

/// The PCX file being decoded.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct Pcx {
    /// The picture's width, as the header says.
    pub file_width: PlainDword,
    /// Its height.
    pub file_height: PlainDword,
    pub line_bytes: PlainDword,
    pub planes: PlainDword,
    /// The stream the readers opened the file as; not used (`files::stream::with_stream`), kept for
    /// the block's layout.
    pub stream: PlainBytes<STREAM_LEN>,
    /// The header as read from the file, 0x80 bytes.
    pub header: PlainBytes<HEADER_LEN>,
    /// The palette as read from the file, three bytes an entry.
    pub file_palette: PlainBytes<FILE_PALETTE_LEN>,
    /// The widest picture that is taken.
    pub width_limit: PlainDword,
    /// The highest picture that is taken.
    pub height_limit: PlainDword,
}

impl Pcx {
    pub const fn new() -> Self {
        Pcx {
            file_width: PlainDword::new(0),
            file_height: PlainDword::new(0),
            line_bytes: PlainDword::new(0),
            planes: PlainDword::new(0),
            stream: PlainBytes::new(with_runs(&[(4, &[0xFF; 4])])),
            header: PlainBytes::new([0; HEADER_LEN]),
            file_palette: PlainBytes::new([0; FILE_PALETTE_LEN]),
            width_limit: PlainDword::new(0x320),
            height_limit: PlainDword::new(0x258),
        }
    }
}

/// The header zeroed and the file's first 0x80 bytes read into it (`files::read_file`): whether the
/// read worked.
pub fn read_header(
    path: &[u8],
    gfx: &crate::gfx::owner::Gfx,
) -> Result<bool, crate::errors::BadInput> {
    let mut header = [0u8; HEADER_LEN];
    let ok = crate::files::read_file::read_file_into(path, &mut header, 0)?.ok;
    gfx.pictures.pcx.header.write(&header);
    Ok(ok)
}

/// The header read through `stream` (`files::read_stream_block`): whether it worked. The bytes past
/// A short read stay as they were.
pub(super) fn read_stream_header(
    stream: &mut Stream,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<bool, crate::errors::BadInput> {
    let pictures = &gfx.pictures;
    let mut header = pictures.pcx.header.read();
    let failed = crate::files::read_stream_block::read_stream_block(stream, &mut header)?.1;
    pictures.pcx.header.write(&header);
    Ok(!failed)
}

/// `path` opened as the stream (mode 0), `rows` run on it, the stream closed: whether all three
/// worked.
pub(super) fn on_stream(
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
