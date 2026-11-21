//! The picture buffer: where a picture file is decoded to, as 32-bit pixels of four 8-bit
//! components, before it is converted to the surface's format and drawn. The background of every
//! menu screen goes through it. Its variables are `PICTURE`.
//! The buffers are Rust's (`Buffer`: the game's, the overlay's of `load_picture_with_overlay`, the
//! map key sheet's), counted on their lists as the game's blocks were and given back with them.
//! `PICTURE.buffer` and `DECODER.pixels` hold a buffer's number (0 none), and its users work on its
//! bytes (`with_buffer`, `read_bytes`, `write_bytes`). The drawing layer reaches the picture as
//! `gfx::surface::Target::Picture` (no address).

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::Pictures;
use crate::memory::owned::Owned;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// The buffer holds a whole screen: 800 x 600 pixels of four bytes.
pub(super) const PICTURE_DWORDS: usize = 0x75300;

/// The picture's width, height and row length set (`rc2k_core::gfx::picture_size`).
pub fn set_size(pictures: &Pictures, size: rc2k_core::gfx::picture_size::PictureSize) {
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

/// The three picture buffers and their change counters.
pub struct PictureBuffers {
    buffers: [Owned<Vec<u8>>; 3],
    /// Each buffer's change counter: it moves on every access that may write the buffer, so equal
    /// counters mean equal bytes (`generation`).
    generations: [AtomicU64; 3],
}

impl PictureBuffers {
    pub const fn new() -> Self {
        PictureBuffers {
            buffers: [const { Owned::new("no picture buffer") }; 3],
            generations: [const { AtomicU64::new(0) }; 3],
        }
    }
}

/// The change counter of buffer `handle`: the bytes are the same as when it last had this value.
pub fn generation(buffers: &PictureBuffers, handle: u32) -> Option<u64> {
    let counter = buffers.generations.get((handle as usize).checked_sub(1)?)?;
    Some(counter.load(Relaxed))
}

/// The bytes of buffer `handle` as they are now, held for `f`: no write, so no change counted.
pub fn with_buffer_unchanged<R>(
    buffers: &PictureBuffers,
    handle: u32,
    f: impl FnOnce(&[u8]) -> R,
) -> Result<R, BadInput> {
    owned(buffers, handle)?.with(|b| f(b))
}

/// Buffer `handle` is about to be written (or taken, or given back): counted as changed, after
/// the retained background has taken the bytes it still needs (`render::background`).
fn changing(buffers: &PictureBuffers, handle: u32, bytes: Option<&[u8]>) {
    if let Some(n) = (handle as usize).checked_sub(1)
        && let Some(counter) = buffers.generations.get(n)
    {
        if let Some(bytes) = bytes {
            crate::render::background::before_change(handle, bytes);
        }
        counter.fetch_add(1, Relaxed);
    }
}

fn owned(buffers: &PictureBuffers, handle: u32) -> Result<&Owned<Vec<u8>>, BadInput> {
    (handle as usize)
        .checked_sub(1)
        .and_then(|n| buffers.buffers.get(n))
        .ok_or(BadInput("no picture buffer"))
}

/// Buffer `which` taken, zeroed, `bytes` counted on `list` (sized as the game's block was: whole
/// pages less its header). Its number.
pub fn take(
    buffers: &PictureBuffers,
    which: Buffer,
    list: &impl crate::memory::HeldList,
    bytes: u32,
) -> Result<u32, Refused> {
    let memory = vec![0u8; crate::memory::owned::block_len(bytes)];
    changing(buffers, which as u32, None);
    buffers.buffers[which as usize - 1].take(list, bytes, memory)?;
    Ok(which as u32)
}

/// Buffer `which` given back (its list's `free_list` counts it down).
pub fn give_back(buffers: &PictureBuffers, which: Buffer) {
    // A retained background still drawing from it keeps a copy first.
    let _ = with_buffer_unchanged(buffers, which as u32, |b| {
        crate::render::background::before_change(which as u32, b);
    });
    changing(buffers, which as u32, None);
    buffers.buffers[which as usize - 1].give_back();
}

/// `f` on the bytes of the buffer `handle` names; refused for none or one not held. Counted as a
/// change of the buffer.
pub fn with_buffer<R>(
    buffers: &PictureBuffers,
    handle: u32,
    f: impl FnOnce(&mut [u8]) -> R,
) -> Result<R, BadInput> {
    owned(buffers, handle)?.with(|b| {
        changing(buffers, handle, Some(b));
        f(b)
    })
}

/// `f` on the picture buffer's bytes (`PICTURE.buffer`).
pub fn with_picture<R>(pictures: &Pictures, f: impl FnOnce(&mut [u8]) -> R) -> Result<R, BadInput> {
    with_buffer(&pictures.buffers, pictures.picture.buffer.get(), f)
}

/// Whether a picture buffer is set (`PICTURE.buffer` not 0).
pub(super) fn has_picture(pictures: &Pictures) -> bool {
    pictures.picture.buffer.get() != 0
}

/// `len` bytes of buffer `handle` from byte `at`; refused past its end.
pub fn read_bytes(
    buffers: &PictureBuffers,
    handle: u32,
    at: usize,
    len: usize,
) -> Result<Vec<u8>, BadInput> {
    with_buffer_unchanged(buffers, handle, |b| {
        b.get(at..at.checked_add(len)?).map(<[u8]>::to_vec)
    })?
    .ok_or(BadInput("a read past the picture buffer"))
}

/// `bytes` into buffer `handle` at byte `at`; refused past its end.
pub fn write_bytes(
    buffers: &PictureBuffers,
    handle: u32,
    at: usize,
    bytes: &[u8],
) -> Result<(), BadInput> {
    with_buffer(buffers, handle, |b| {
        b.get_mut(at..at.checked_add(bytes.len())?)
            .map(|to| to.copy_from_slice(bytes))
    })?
    .ok_or(BadInput("a write past the picture buffer"))
}

/// `bytes` into a buffer's bytes at `at`; false (nothing written) past its end.
pub fn put(buffer: &mut [u8], at: usize, bytes: &[u8]) -> bool {
    match at
        .checked_add(bytes.len())
        .and_then(|end| buffer.get_mut(at..end))
    {
        Some(to) => {
            to.copy_from_slice(bytes);
            true
        }
        None => false,
    }
}

/// `bytes` into a buffer's bytes at `at`, a decoder's write: past the end it ends the game.
pub(super) fn put_or_end(buffer: &mut [u8], at: usize, bytes: &[u8]) {
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
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let memory = crate::memory::PlainList::empty();
        let list = &memory;
        let buffers = &PictureBuffers::new();
        let n = take(buffers, Buffer::Key, list, 0x100).unwrap();
        assert_eq!(n, 3);
        assert_eq!(memory.get().total, 0x100);
        write_bytes(buffers, n, 4, &[1, 2, 3]).unwrap();
        assert_eq!(read_bytes(buffers, n, 3, 5).unwrap(), [0, 1, 2, 3, 0]);
        let len = 0x1000 - crate::memory::HEADER_LEN as usize;
        assert!(read_bytes(buffers, n, len - 1, 2).is_err());
        assert!(read_bytes(buffers, 0, 0, 1).is_err());
        give_back(buffers, Buffer::Key);
        assert!(read_bytes(buffers, n, 0, 1).is_err());
        crate::memory::with_figures(|f| *f = figures);
    }

    #[test]
    fn the_change_counter_moves_on_writes_and_not_on_reads() {
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let memory = crate::memory::PlainList::empty();
        let buffers = &PictureBuffers::new();
        let n = take(buffers, Buffer::Key, &memory, 0x100).unwrap();
        let before = generation(buffers, n).unwrap();
        read_bytes(buffers, n, 0, 4).unwrap();
        with_buffer_unchanged(buffers, n, <[u8]>::len).unwrap();
        assert_eq!(generation(buffers, n), Some(before), "reads leave it");
        write_bytes(buffers, n, 0, &[1]).unwrap();
        let written = generation(buffers, n).unwrap();
        assert!(written > before, "a write moves it");
        with_buffer(buffers, n, |_| ()).unwrap();
        assert!(
            generation(buffers, n).unwrap() > written,
            "a mutable access moves it"
        );
        give_back(buffers, Buffer::Key);
        assert_eq!(generation(buffers, 0), None);
        crate::memory::with_figures(|f| *f = figures);
    }

    /// The picture records' values and the registry's picture stores, kept to be put back: the
    /// records are plain fields, so the registry does not walk them.
    struct KeptPictureState {
        decoder: Vec<u32>,
        pcx: Vec<u32>,
        picture: Vec<u32>,
        png_file: Vec<u32>,
        png_header: Vec<u32>,
        planes: Vec<u8>,
        registry: crate::test_support::state::Snapshot,
    }

    impl KeptPictureState {
        fn take() -> Self {
            let gfx = crate::test_support::own::gfx();
            KeptPictureState {
                decoder: gfx.pictures.decoder.values(),
                pcx: gfx.pictures.pcx.values(),
                picture: gfx.pictures.picture.values(),
                png_file: gfx.png.file.values(),
                png_header: gfx.png.header.values(),
                planes: gfx.pictures.planes.read().to_vec(),
                registry: crate::test_support::state::Snapshot::of(&[
                    "owners.gfx.pictures.showing",
                    "owners.gfx.png.setting",
                ]),
            }
        }

        fn restore(&self) {
            let gfx = crate::test_support::own::gfx();
            gfx.pictures.decoder.set_values(&self.decoder);
            gfx.pictures.pcx.set_values(&self.pcx);
            gfx.pictures.picture.set_values(&self.picture);
            gfx.png.file.set_values(&self.png_file);
            gfx.png.header.set_values(&self.png_header);
            gfx.pictures
                .planes
                .write_at(0, &self.planes, "kept planes")
                .unwrap();
            self.registry.restore();
        }
    }

    #[test]
    fn a_pcx_file_is_decoded_into_the_picture_buffer() {
        let pictures = &crate::test_support::own::gfx().pictures;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = KeptPictureState::take();
        let memory = crate::memory::PlainList::empty();
        let list = &memory;
        pictures
            .picture
            .buffer
            .set(take(&pictures.buffers, Buffer::Main, list, 0x1D_4C00 + 0xC80).unwrap());
        set_size(
            &crate::test_support::own::gfx().pictures,
            rc2k_core::gfx::picture_size::READ,
        );
        // A 4 x 2 picture of palette numbers 1 .. 8 (each byte below 0xC0 is itself), the palette
        // after it: number k is (k, 2k, 3k).
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
        let mut path = [0u8; 64];
        path[..name.len()].copy_from_slice(name.as_bytes());
        let read = crate::gfx::read_picture_file::read_picture_file(
            &path,
            crate::test_support::own::gfx(),
            crate::test_support::own::system(),
        );
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
            let pixel =
                read_bytes(&pictures.buffers, pictures.picture.buffer.get(), at, 4).unwrap();
            assert_eq!(
                u32::from_le_bytes(pixel.try_into().unwrap()),
                palette[number],
                "pixel {n}"
            );
        }
        assert_ne!(palette[1], palette[2]);
        give_back(&pictures.buffers, Buffer::Main);
        kept.restore();
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
        let gfx = crate::test_support::own::gfx();
        let name = format!("target/rc2k_png_test_{}.png", std::process::id());
        std::fs::write(&name, file).unwrap();
        let read = crate::gfx::read_png::read_png(
            &gfx.png,
            name.as_bytes(),
            crate::test_support::own::gfx(),
            crate::test_support::own::system(),
        );
        std::fs::remove_file(&name).unwrap();
        assert!(read.unwrap(), "the picture was read");
        read_bytes(
            &gfx.pictures.buffers,
            gfx.pictures.picture.buffer.get(),
            0,
            6 * 4,
        )
        .unwrap()
    }

    #[test]
    fn a_png_file_is_decoded_its_first_row_against_zeros() {
        let pictures = &crate::test_support::own::gfx().pictures;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = KeptPictureState::take();
        let memory = crate::memory::PlainList::empty();
        let list = &memory;
        pictures
            .picture
            .buffer
            .set(take(&pictures.buffers, Buffer::Main, list, 0x1D_4C00 + 0xC80).unwrap());
        set_size(
            &crate::test_support::own::gfx().pictures,
            rc2k_core::gfx::picture_size::READ,
        );
        let plain = read_png_file(&png(0));
        assert_eq!(pictures.picture.width.get(), 3);
        assert_eq!(
            &plain[..3],
            &[30, 20, 10],
            "the first pixel's components, blue first"
        );
        let up = read_png_file(&png(2));
        assert_eq!(plain, up, "Up against the zero row is the row itself");
        give_back(&pictures.buffers, Buffer::Main);
        kept.restore();
        crate::memory::with_figures(|f| *f = figures);
    }

    #[test]
    fn the_png_info_holds_the_size_and_the_palette_only_when_there_is_one() {
        use crate::gfx::png::{PALETTE_LEN, PngInfo};
        let pictures = &crate::test_support::own::gfx().pictures;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = KeptPictureState::take();
        let memory = crate::memory::PlainList::empty();
        let list = &memory;
        pictures
            .picture
            .buffer
            .set(take(&pictures.buffers, Buffer::Main, list, 0x1D_4C00 + 0xC80).unwrap());
        set_size(
            &crate::test_support::own::gfx().pictures,
            rc2k_core::gfx::picture_size::READ,
        );
        let leftover = || PngInfo {
            has_palette: true,
            width: 77,
            height: 88,
            palette: [0x55; PALETTE_LEN],
        };
        let mut plain = leftover();
        let file = png(0);
        let code = crate::gfx::decode_png::decode_png(
            &crate::test_support::own::gfx().png,
            &pictures.buffers,
            &file,
            pictures.picture.buffer.get(),
            &mut plain,
        );
        assert_eq!(code.unwrap(), 0);
        assert!(!plain.has_palette);
        assert_eq!((plain.width, plain.height), (3, 2));
        assert_eq!(
            plain.palette, [0x55; PALETTE_LEN],
            "no PLTE: the palette stays"
        );
        // The same file with a PLTE chunk before its IDAT.
        let idat = file.windows(4).position(|w| w == b"IDAT").unwrap() - 4;
        let colours: Vec<u8> = (0..PALETTE_LEN).map(|n| n as u8).collect();
        let mut chunk = (PALETTE_LEN as u32).to_be_bytes().to_vec();
        chunk.extend(b"PLTE");
        chunk.extend(&colours);
        chunk.extend([0; 4]);
        let mut with = file[..idat].to_vec();
        with.extend(chunk);
        with.extend(&file[idat..]);
        let mut info = leftover();
        let code = crate::gfx::decode_png::decode_png(
            &crate::test_support::own::gfx().png,
            &pictures.buffers,
            &with,
            pictures.picture.buffer.get(),
            &mut info,
        );
        assert_eq!(code.unwrap(), 0);
        assert!(info.has_palette);
        assert_eq!((info.width, info.height), (3, 2));
        assert_eq!(info.palette.as_slice(), colours.as_slice());
        give_back(&pictures.buffers, Buffer::Main);
        kept.restore();
        crate::memory::with_figures(|f| *f = figures);
    }
}
