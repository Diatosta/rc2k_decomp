//! A PCX file here: a header of 0x80 bytes (+4 .. +0xA the picture's left, top, right and
//! bottom as words; +0x41 the number of planes, 1 or 3; +0x42 the bytes of one plane of one
//! row as a word), then the rows, run-length coded: a byte of 0xC0 or more is a count (less
//! 0xC0) for the byte after it, any other byte stands for itself. With one plane a byte is
//! a palette number and the palette, 0x300 bytes, is the end of the file.

use crate::files::stream::Stream;
use crate::gfx::block::game_ptr;
use crate::gfx::decoder::Slot as DecSlot;

/// dword_51B5F4: the header as read from the file, 0x80 bytes.
pub const HEADER_RVA: DecSlot = DecSlot::at(0x51B5F4);
pub const HEADER_LEN: usize = 0x80;
pub const HEADER_PLANES: usize = 0x41;
/// unk_51B674: the palette as read from the file, three bytes an entry.
pub const FILE_PALETTE_RVA: DecSlot = DecSlot::at(0x51B674);
pub const FILE_PALETTE_LEN: usize = 0x300;
/// dword_5195E4: the stream the readers open the file as.
pub const STREAM_RVA: DecSlot = DecSlot::at(0x5195E4);

// The layout: what a decoded picture is, for the readers and for what comes after them.
pub const PLANES_RVA: DecSlot = DecSlot::at(0x5195E0); // dword_5195E0
pub const LINE_BYTES_RVA: DecSlot = DecSlot::at(0x5195DC); // dword_5195DC
pub const FILE_WIDTH_RVA: DecSlot = DecSlot::at(0x5195D4); // dword_5195D4: as the header says
pub const FILE_HEIGHT_RVA: DecSlot = DecSlot::at(0x5195D8); // dword_5195D8
pub const DECODED_WIDTH_RVA: DecSlot = DecSlot::at(0x519124); // dword_519124: cut to what fits
pub const DECODED_HEIGHT_RVA: DecSlot = DecSlot::at(0x519128); // dword_519128
pub const PIXEL_BITS_RVA: DecSlot = DecSlot::at(0x51912C); // dword_51912C: 8 or 32
pub const PIXEL_BYTES_RVA: DecSlot = DecSlot::at(0x519130); // dword_519130
pub const DECODED_ROW_BYTES_RVA: DecSlot = DecSlot::at(0x519134); // dword_519134
pub const DECODED_ROW_DWORDS_RVA: DecSlot = DecSlot::at(0x519138); // dword_519138
pub const STRIDE_RVA: DecSlot = DecSlot::at(0x51913C); // dword_51913C: from one row to the next
pub const PIXELS_RVA: DecSlot = DecSlot::at(0x519140); // dword_519140: where the rows go
pub const ROW_GAP_RVA: DecSlot = DecSlot::at(0x519144); // dword_519144: the stride less a row's bytes
pub const PALETTE_RVA: DecSlot = DecSlot::at(0x519154); // dword_519154: 256 dwords
/// dword_51B974, dword_51B978: the widest and highest picture that is taken.
pub const WIDTH_LIMIT_RVA: DecSlot = DecSlot::at(0x51B974);
pub const HEIGHT_LIMIT_RVA: DecSlot = DecSlot::at(0x51B978);

pub fn stream() -> *mut Stream {
    game_ptr(STREAM_RVA)
}
