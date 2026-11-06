//! The picture buffer: where a picture file is decoded to, as 32-bit pixels of four 8-bit
//! components, before it is converted to the surface's format and drawn. The background of
//! every menu screen goes through it. Its variables are `PICTURE`.
//!
//! Phase 12 (2026-10-27): the buffers are Rust's (`Buffer`: the game's, the overlay's of
//! `load_picture_with_overlay`, the map key sheet's), counted on their lists as the game's
//! blocks were and given back with them. `PICTURE.buffer` and `DECODER.pixels` hold a
//! buffer's number (0 none), and its users work on its bytes (`with_buffer`, `read_bytes`,
//! `write_bytes`). The drawing layer reaches the picture as `gfx::surface::Target::Picture`
//! (no address).

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::memory::owned::Owned;

/// The buffer holds a whole screen: 800 x 600 pixels of four bytes.
pub const PICTURE_DWORDS: usize = 0x75300;

/// The picture's width, height and row length set (`rc2k_core::gfx::picture_size`).
pub fn set_size(size: rc2k_core::gfx::picture_size::PictureSize) {
    let pictures = &gfx().pictures;
    let [width, height, row_bytes] = size.dwords();
    pictures.picture.width.set(width);
    pictures.picture.height.set(height);
    pictures.picture.row_bytes.set(row_bytes);
}

/// The picture buffers, by the number `PICTURE.buffer` holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buffer {
    /// `gfx::allocate_picture_buffer`'s, on `PICTURE.list`.
    Main = 1,
    /// `gfx::load_picture_with_overlay`'s, on the lexicon's list while it runs.
    Overlay = 2,
    /// `race::load_game_map`'s key sheet, on `MAP.key_list` while it runs.
    Key = 3,
}

static BUFFERS: [Owned<Vec<u8>>; 3] = [const { Owned::new("no picture buffer") }; 3];

fn owned(handle: u32) -> Result<&'static Owned<Vec<u8>>, BadInput> {
    (handle as usize)
        .checked_sub(1)
        .and_then(|n| BUFFERS.get(n))
        .ok_or(BadInput("no picture buffer"))
}

/// Buffer `which` taken, zeroed, `bytes` counted on `list` (sized as the game's block was:
/// whole pages less its header). Its number.
pub fn take(which: Buffer, list: crate::pointers::GamePtr, bytes: u32) -> Result<u32, Refused> {
    let memory = vec![0u8; crate::memory::owned::block_len(bytes)];
    BUFFERS[which as usize - 1].take(list, bytes, memory)?;
    Ok(which as u32)
}

/// Buffer `which` given back (its list's `free_list` counts it down).
pub fn give_back(which: Buffer) {
    BUFFERS[which as usize - 1].give_back();
}

/// `f` on the bytes of the buffer `handle` names; refused for none or one not held.
pub fn with_buffer<R>(handle: u32, f: impl FnOnce(&mut [u8]) -> R) -> Result<R, BadInput> {
    owned(handle)?.with(|b| f(b))
}

/// `f` on the picture buffer's bytes (`PICTURE.buffer`).
pub fn with_picture<R>(f: impl FnOnce(&mut [u8]) -> R) -> Result<R, BadInput> {
    with_buffer(gfx().pictures.picture.buffer.get(), f)
}

/// Whether a picture buffer is set (`PICTURE.buffer` not 0).
pub fn has_picture() -> bool {
    gfx().pictures.picture.buffer.get() != 0
}

/// `len` bytes of buffer `handle` from byte `at`; refused past its end.
pub fn read_bytes(handle: u32, at: usize, len: usize) -> Result<Vec<u8>, BadInput> {
    with_buffer(handle, |b| {
        b.get(at..at.checked_add(len)?).map(<[u8]>::to_vec)
    })?
    .ok_or(BadInput("a read past the picture buffer"))
}

/// `bytes` into buffer `handle` at byte `at`; refused past its end.
pub fn write_bytes(handle: u32, at: usize, bytes: &[u8]) -> Result<(), BadInput> {
    with_buffer(handle, |b| {
        b.get_mut(at..at.checked_add(bytes.len())?)
            .map(|to| to.copy_from_slice(bytes))
    })?
    .ok_or(BadInput("a write past the picture buffer"))
}

/// `bytes` into a buffer's bytes at `at`, a decoder's write: past the end it ends the game,
/// as the original's write past its block would have gone wrong.
pub fn put_or_end(buffer: &mut [u8], at: usize, bytes: &[u8]) {
    match at
        .checked_add(bytes.len())
        .and_then(|end| buffer.get_mut(at..end))
    {
        Some(to) => to.copy_from_slice(bytes),
        None => crate::fatal::raise::end(
            "gfx::picture",
            BadInput("a decoded pixel past the picture buffer"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buffer_is_taken_read_and_given_back() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let mut memory = vec![0u8; size_of::<crate::memory::MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        let list =
            crate::pointers::GamePtr::new(crate::pointers::game_address(memory.as_mut_ptr()));
        let n = take(Buffer::Key, list, 0x100).unwrap();
        assert_eq!(n, 3);
        assert_eq!(u32::from_le_bytes(memory[4..8].try_into().unwrap()), 0x100);
        write_bytes(n, 4, &[1, 2, 3]).unwrap();
        assert_eq!(read_bytes(n, 3, 5).unwrap(), [0, 1, 2, 3, 0]);
        let len = 0x1000 - crate::memory::HEADER_LEN as usize;
        assert!(read_bytes(n, len - 1, 2).is_err());
        assert!(read_bytes(0, 0, 1).is_err());
        give_back(Buffer::Key);
        assert!(read_bytes(n, 0, 1).is_err());
        crate::memory::with_figures(|f| *f = figures);
    }

    #[test]
    fn a_pcx_file_is_decoded_into_the_picture_buffer() {
        let pictures = &gfx().pictures;
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept = crate::moved::snapshot();
        let mut memory = vec![0u8; size_of::<crate::memory::MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        let list =
            crate::pointers::GamePtr::new(crate::pointers::game_address(memory.as_mut_ptr()));
        pictures
            .picture
            .buffer
            .set(take(Buffer::Main, list, 0x1D_4C00 + 0xC80).unwrap());
        set_size(rc2k_core::gfx::picture_size::READ);
        // A 4 x 2 picture of palette numbers 1 .. 8 (each byte below 0xC0 is itself), the
        // palette after it: number k is (k, 2k, 3k).
        let mut file = vec![0u8; 0x80];
        file[0] = 10;
        file[1] = 5;
        file[2] = 1;
        file[3] = 8;
        file[8..10].copy_from_slice(&3u16.to_le_bytes());
        file[0xA..0xC].copy_from_slice(&1u16.to_le_bytes());
        file[0x41] = 1;
        file[0x42..0x44].copy_from_slice(&4u16.to_le_bytes());
        file.extend(1..=8u8);
        file.push(0x0C);
        file.extend((0..256u32).flat_map(|k| [k as u8, (2 * k) as u8, (3 * k) as u8]));
        let name = format!("target/rc2k_picture_test_{}.pcx", std::process::id());
        std::fs::write(&name, &file).unwrap();
        let path: Vec<u8> = name.bytes().chain([0]).collect();
        let path_at = crate::pointers::Lent::new(&path);
        let read = crate::gfx::read_picture_file::read_picture_file(
            &crate::pointers::GamePtr::new(path_at.addr()),
        );
        drop(path_at);
        std::fs::remove_file(&name).unwrap();
        assert!(read.unwrap(), "the picture was read");
        let palette: [u32; 256] = pictures.decoder.palette();
        let (width, stride) = (
            pictures.decoder.width.get(),
            pictures.decoder.stride.get() as usize,
        );
        assert_eq!(width, 4);
        for (n, number) in (1..=8usize).enumerate() {
            let at = (n / 4) * stride + (n % 4) * 4;
            let pixel = read_bytes(pictures.picture.buffer.get(), at, 4).unwrap();
            assert_eq!(
                u32::from_le_bytes(pixel.try_into().unwrap()),
                palette[number],
                "pixel {n}"
            );
        }
        assert_ne!(palette[1], palette[2]);
        give_back(Buffer::Main);
        crate::moved::restore(&kept);
        crate::memory::with_figures(|f| *f = figures);
    }

    /// A 3 x 2 PNG of colour type 2, its first row filtered `first_filter` (0 None, 2 Up).
    fn png(first_filter: u8) -> Vec<u8> {
        let rows: [[u8; 9]; 2] = [
            [10, 20, 30, 40, 50, 60, 70, 80, 90],
            [1, 2, 3, 4, 5, 6, 7, 8, 9],
        ];
        let mut raw = vec![first_filter];
        raw.extend(rows[0]);
        raw.push(0);
        raw.extend(rows[1]);
        let chunk = |kind: &[u8], data: &[u8]| {
            let mut c = (data.len() as u32).to_be_bytes().to_vec();
            c.extend(kind);
            c.extend(data);
            c.extend([0; 4]);
            c
        };
        let mut header = 3u32.to_be_bytes().to_vec();
        header.extend(2u32.to_be_bytes());
        header.extend([8, 2, 0, 0, 0]);
        let mut file = b"\x89PNG\r\n\x1a\n".to_vec();
        file.extend(chunk(b"IHDR", &header));
        // The reader takes a head of 0x80 bytes first.
        file.extend(chunk(b"tEXt", &[b'x'; 0x60]));
        file.extend(chunk(
            b"IDAT",
            &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6),
        ));
        file.extend(chunk(b"IEND", &[]));
        file
    }

    /// The picture buffer's first 6 x 4 bytes after `read_png` of `file`.
    fn read_png_file(file: &[u8]) -> Vec<u8> {
        let gfx = gfx();
        let name = format!("target/rc2k_png_test_{}.png", std::process::id());
        std::fs::write(&name, file).unwrap();
        let read = crate::gfx::read_png::read_png(&gfx.png, name.as_bytes());
        std::fs::remove_file(&name).unwrap();
        assert!(read.unwrap(), "the picture was read");
        read_bytes(gfx.pictures.picture.buffer.get(), 0, 6 * 4).unwrap()
    }

    #[test]
    fn a_png_file_is_decoded_its_first_row_against_zeros() {
        let pictures = &gfx().pictures;
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept = crate::moved::snapshot();
        let mut memory = vec![0u8; size_of::<crate::memory::MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        let list =
            crate::pointers::GamePtr::new(crate::pointers::game_address(memory.as_mut_ptr()));
        pictures
            .picture
            .buffer
            .set(take(Buffer::Main, list, 0x1D_4C00 + 0xC80).unwrap());
        set_size(rc2k_core::gfx::picture_size::READ);
        let plain = read_png_file(&png(0));
        assert_eq!(pictures.picture.width.get(), 3);
        assert_eq!(
            &plain[..3],
            &[30, 20, 10],
            "the first pixel's components, blue first"
        );
        let up = read_png_file(&png(2));
        assert_eq!(plain, up, "Up against the zero row is the row itself");
        give_back(Buffer::Main);
        crate::moved::restore(&kept);
        crate::memory::with_figures(|f| *f = figures);
    }
}
