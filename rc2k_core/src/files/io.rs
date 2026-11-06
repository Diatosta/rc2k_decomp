//! The game's file functions over a file system (converted 2026-10-21 from `files::read_file`
//! 0x402BB6, `write_file` 0x402BFB, `copy_file` 0x402C28, `file_exists` 0x402EBE,
//! `file_size` 0x402ED4 and the buffered streams: `open_stream` 0x402CDD, `close_stream`
//! 0x402CFA, `read_stream_byte` 0x402D16, `write_stream_byte` 0x402D53, `read_stream_block`
//! 0x402DA3, `write_stream_block` 0x402DC9). The file system is the platform's
//! (`Files`): the game's own primitives in the hooked game, the same Win32 calls in the
//! standalone.

use crate::Invalid;

/// The game's file modes (its table at 0x4E06EA, 16 bytes a mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// An existing file, to read.
    Read = 0,
    /// Created, or replaced.
    Create = 1,
    /// Created, not replacing one.
    CreateNew = 2,
    /// An existing file, to read and write.
    ReadWrite = 3,
    /// Opened or created, to write.
    Append = 4,
}

impl Mode {
    /// A mode by its number; refused past the table.
    pub fn from_number(n: u32) -> Result<Mode, Invalid> {
        Ok(match n {
            0 => Mode::Read,
            1 => Mode::Create,
            2 => Mode::CreateNew,
            3 => Mode::ReadWrite,
            4 => Mode::Append,
            _ => return Err(Invalid("a file mode past the table")),
        })
    }
}

/// Where a seek counts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum From {
    Start = 0,
    Here = 1,
    End = 2,
}

/// A file system as the game's primitives see it.
pub trait Files {
    /// A path as the caller gives it.
    type Path: Copy;
    /// An open file.
    type Handle: Copy;
    /// A path the file system refuses (the game's path resolving), before anything.
    fn check(&mut self, path: Self::Path) -> Result<(), Invalid>;
    /// A file opened, or `None` when it cannot be.
    fn open(&mut self, path: Self::Path, mode: Mode) -> Result<Option<Self::Handle>, Invalid>;
    /// Bytes read into `into`: how many, and whether the read worked (a short read works).
    fn read(&mut self, file: Self::Handle, into: &mut [u8]) -> (u32, bool);
    /// Bytes written: how many, and whether all of them were.
    fn write(&mut self, file: Self::Handle, from: &[u8]) -> (u32, bool);
    /// The new position, or `None`.
    fn seek(&mut self, file: Self::Handle, distance: i32, from: From) -> Option<u32>;
    /// The size, or `None`.
    fn size(&mut self, file: Self::Handle) -> Option<u32>;
    /// Whether it closed.
    fn close(&mut self, file: Self::Handle) -> bool;
    /// Its last write time as DOS date and time, or `None`.
    fn time(&mut self, file: Self::Handle) -> Option<u32>;
    /// Its last write time set; whether it was.
    fn set_time(&mut self, file: Self::Handle, dos: u32) -> bool;
}

/// What a read of a whole file leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadResult {
    pub ok: bool,
    /// `None` when the read was not reached (the open or the seek failed).
    pub bytes_read: Option<u32>,
}

/// A file read into `into` from `offset` (no seek at 0); ok when the read worked and the
/// file closed.
pub fn read_file<F: Files>(
    f: &mut F,
    path: F::Path,
    into: &mut [u8],
    offset: u32,
) -> Result<ReadResult, Invalid> {
    let Some(file) = f.open(path, Mode::Read)? else {
        return Ok(ReadResult {
            ok: false,
            bytes_read: None,
        });
    };
    let mut result = ReadResult {
        ok: false,
        bytes_read: None,
    };
    if offset == 0 || f.seek(file, offset as i32, From::Start).is_some() {
        let (n, ok) = f.read(file, into);
        result = ReadResult {
            ok,
            bytes_read: Some(n),
        };
    }
    let closed = f.close(file);
    result.ok &= closed;
    Ok(result)
}

/// What a write of a whole file leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteResult {
    pub ok: bool,
    /// `None` when the write was not reached (the open failed).
    pub bytes_written: Option<u32>,
}

/// A file created (or replaced) holding `data`; ok when all was written and it closed.
pub fn write_file<F: Files>(f: &mut F, path: F::Path, data: &[u8]) -> Result<WriteResult, Invalid> {
    let Some(file) = f.open(path, Mode::Create)? else {
        return Ok(WriteResult {
            ok: false,
            bytes_written: None,
        });
    };
    let (n, all) = f.write(file, data);
    let closed = f.close(file);
    Ok(WriteResult {
        ok: closed && all,
        bytes_written: Some(n),
    })
}

/// Whether a file opens to read (and closes again).
pub fn file_exists<F: Files>(f: &mut F, path: F::Path) -> Result<bool, Invalid> {
    Ok(match f.open(path, Mode::Read)? {
        Some(file) => f.close(file),
        None => false,
    })
}

/// A file's size (0xFFFFFFFF when not known) and whether it was found and closed.
pub fn file_size<F: Files>(f: &mut F, path: F::Path) -> Result<(u32, bool), Invalid> {
    let Some(file) = f.open(path, Mode::Read)? else {
        return Ok((u32::MAX, false));
    };
    let size = f.size(file);
    let closed = f.close(file);
    Ok((size.unwrap_or(u32::MAX), closed && size.is_some()))
}

/// A file copied through `buffer` (the game's scratch block), its time with it: done when
/// the whole source was read and the time set. Both paths checked before anything.
pub fn copy_file<F: Files>(
    f: &mut F,
    from: F::Path,
    to: F::Path,
    buffer: &mut [u8],
) -> Result<bool, Invalid> {
    f.check(to)?;
    let Some(source) = f.open(from, Mode::Read)? else {
        return Ok(false);
    };
    let Some(target) = f.open(to, Mode::Create)? else {
        f.close(source);
        return Ok(false);
    };
    let mut done = false;
    loop {
        let (n, ok) = f.read(source, buffer);
        if !ok {
            break;
        }
        if n == 0 {
            if let Some(time) = f.time(source) {
                done = f.set_time(target, time);
            }
            break;
        }
        if !f.write(target, &buffer[..n as usize]).1 {
            break;
        }
    }
    f.close(target);
    f.close(source);
    Ok(done)
}

/// What `files::read_whole_file` (0x421B54) comes to: whether the file opened, the bytes
/// read and whether that read worked, whether it closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Whole {
    pub opened: bool,
    pub bytes: u32,
    pub read_ok: bool,
    pub closed: bool,
}

/// A file's start read into `into`.
pub fn read_whole_file<F: Files>(
    f: &mut F,
    path: F::Path,
    into: &mut [u8],
) -> Result<Whole, Invalid> {
    let Some(file) = f.open(path, Mode::Read)? else {
        return Ok(Whole {
            opened: false,
            bytes: 0,
            read_ok: false,
            closed: false,
        });
    };
    let (bytes, read_ok) = f.read(file, into);
    let closed = f.close(file);
    Ok(Whole {
        opened: true,
        bytes,
        read_ok,
        closed,
    })
}

/// The path `files::read_first_match` (0x402F82) reads: the pattern up to and with its
/// last `/`, `\` or `:` (one at its very start does not count), then the name found; with
/// its NUL.
pub fn match_path(pattern: &[u8], name: &[u8]) -> Vec<u8> {
    let directory = (1..pattern.len())
        .rev()
        .find(|&n| matches!(pattern[n], b'/' | b'\\' | b':'))
        .map_or(0, |n| n + 1);
    let mut out = pattern[..directory].to_vec();
    out.extend_from_slice(name);
    out.push(0);
    out
}

/// A level file's packed copy (`files::read_level_file` 0x41BA2E): the path put at +5,
/// ".sq" after it, then "files\\leve" over the first ten bytes, so that `srclvls\\`
/// becomes `files\\levels\\`; with its NUL. Refused: a path whose copy ends inside the
/// ten bytes (the original's text then goes on past them).
pub fn packed_level_path(path: &[u8]) -> Result<Vec<u8>, Invalid> {
    let mut out = vec![0u8; 5];
    out.extend_from_slice(path);
    out.extend_from_slice(b".sq");
    if out.len() < 10 {
        return Err(Invalid(
            "a level path too short for its packed copy's prefix",
        ));
    }
    out[..10].copy_from_slice(b"files\\leve");
    out.push(0);
    Ok(out)
}

/// The kind `GetDriveTypeA` gives a CD-ROM.
pub const CD_ROM: u32 = 5;

/// `files::is_cd_drive` (0x403310): the flags of comparing a drive's kind with a CD-ROM's
/// (carry: below, zero: a CD-ROM).
pub fn is_cd(kind: u32) -> (bool, bool) {
    (kind < CD_ROM, kind == CD_ROM)
}

/// A stream's buffer.
pub const BUFFER_LEN: usize = 0x2000;

/// A buffered file, as the game keeps one (the block the stream functions take in esi).
#[repr(C)]
pub struct Stream<H> {
    /// The mode it was opened with (1: written to, closing writes the buffer out).
    pub mode: u32,
    pub file: H,
    /// The next byte of the buffer to hand out, or to fill.
    pub position: u32,
    /// The bytes in the buffer.
    pub filled: u32,
    pub buffer: [u8; BUFFER_LEN],
}

/// A stream opened: the mode kept (also when the file does not open), the position and
/// fill 0. Whether it opened.
pub fn open_stream<F: Files>(
    f: &mut F,
    s: &mut Stream<F::Handle>,
    path: F::Path,
    mode: u32,
) -> Result<bool, Invalid> {
    let m = Mode::from_number(mode)?;
    let file = f.open(path, m)?;
    s.mode = mode;
    let Some(file) = file else {
        return Ok(false);
    };
    s.file = file;
    s.position = 0;
    s.filled = 0;
    Ok(true)
}

/// A stream closed (a written one's buffer written out first), its handle `none` after.
pub fn close_stream<F: Files>(
    f: &mut F,
    s: &mut Stream<F::Handle>,
    none: F::Handle,
) -> Result<bool, Invalid> {
    if s.mode == 1 && s.filled != 0 {
        let filled = buffered(s.filled)?;
        f.write(s.file, &s.buffer[..filled]);
    }
    let closed = f.close(s.file);
    s.file = none;
    Ok(closed)
}

fn buffered(n: u32) -> Result<usize, Invalid> {
    if n as usize > BUFFER_LEN {
        return Err(Invalid("a stream's place past its buffer"));
    }
    Ok(n as usize)
}

/// What reading a byte from a stream gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteRead {
    Byte(u8),
    /// The end of the file.
    End,
    /// A read failed. `count_low` is the low byte of the count that read reported: the
    /// original leaves it where the byte would be, and the picture readers take it as one.
    Failed {
        count_low: u8,
    },
}

impl ByteRead {
    /// The byte, if there was one.
    pub fn byte(self) -> Option<u8> {
        match self {
            Self::Byte(byte) => Some(byte),
            Self::End | Self::Failed { .. } => None,
        }
    }
    /// Whether it was a read error and not the end of the file.
    pub fn failed(self) -> bool {
        matches!(self, Self::Failed { .. })
    }
    /// The byte as the picture readers take it, whatever happened: 0 at the end of the file,
    /// `count_low` after a failed read.
    pub fn value(self) -> u8 {
        match self {
            Self::Byte(byte) => byte,
            Self::End => 0,
            Self::Failed { count_low } => count_low,
        }
    }
}

/// The stream's next byte, the buffer filled again when it is used up. Refused: a position
/// past the buffer (the original reads past it).
pub fn read_stream_byte<F: Files>(
    f: &mut F,
    s: &mut Stream<F::Handle>,
) -> Result<ByteRead, Invalid> {
    if s.position >= s.filled {
        let (count, ok) = f.read(s.file, &mut s.buffer);
        if !ok {
            return Ok(ByteRead::Failed {
                count_low: count as u8,
            });
        }
        s.filled = count;
        s.position = 0;
        if count == 0 {
            return Ok(ByteRead::End);
        }
    }
    let Some(&byte) = s.buffer.get(s.position as usize) else {
        return Err(Invalid("a stream's place past its buffer"));
    };
    s.position = s.position.wrapping_add(1);
    Ok(ByteRead::Byte(byte))
}

/// Bytes read into `into`: how many, and whether a read failed.
pub fn read_stream_block<F: Files>(
    f: &mut F,
    s: &mut Stream<F::Handle>,
    into: &mut [u8],
) -> Result<(u32, bool), Invalid> {
    for (n, b) in into.iter_mut().enumerate() {
        match read_stream_byte(f, s)? {
            ByteRead::Byte(byte) => *b = byte,
            end => return Ok((n as u32, end.failed())),
        }
    }
    Ok((into.len() as u32, false))
}

/// A byte into a stream opened to write, the full buffer written out first; false when
/// that write fails.
pub fn write_stream_byte<F: Files>(
    f: &mut F,
    s: &mut Stream<F::Handle>,
    byte: u8,
) -> Result<bool, Invalid> {
    if s.position as usize >= BUFFER_LEN {
        let filled = buffered(s.filled)?;
        if !f.write(s.file, &s.buffer[..filled]).1 {
            return Ok(false);
        }
        s.filled = 0;
        s.position = 0;
    }
    s.buffer[s.position as usize] = byte;
    s.position = s.position.wrapping_add(1);
    s.filled = s.filled.wrapping_add(1);
    Ok(true)
}

/// Bytes into a stream: (how many, the carry: a write failed, the zero flag: none to write
/// or the last eax 0); `eax` that register at entry, its low byte each byte in turn.
pub fn write_stream_block<F: Files>(
    f: &mut F,
    s: &mut Stream<F::Handle>,
    bytes: &[u8],
    eax: u32,
) -> Result<(u32, bool, bool), Invalid> {
    if bytes.is_empty() {
        return Ok((0, false, true));
    }
    let mut eax = eax;
    for (n, &b) in bytes.iter().enumerate() {
        eax = eax & !0xFF | b as u32;
        if !write_stream_byte(f, s, b)? {
            return Ok((n as u32, true, false));
        }
    }
    Ok((bytes.len() as u32, false, eax == 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Files in memory: a handle is a file's name and a position.
    #[derive(Default)]
    struct Memory {
        files: HashMap<u32, Vec<u8>>,
        open: HashMap<u32, (u32, usize)>,
        next: u32,
        chunk: usize,
    }

    impl Files for Memory {
        type Path = u32;
        type Handle = u32;
        fn check(&mut self, _: u32) -> Result<(), Invalid> {
            Ok(())
        }
        fn open(&mut self, path: u32, mode: Mode) -> Result<Option<u32>, Invalid> {
            match mode {
                Mode::Read if !self.files.contains_key(&path) => return Ok(None),
                Mode::Create => {
                    self.files.insert(path, Vec::new());
                }
                _ => {}
            }
            self.next += 1;
            self.open.insert(self.next, (path, 0));
            Ok(Some(self.next))
        }
        fn read(&mut self, h: u32, into: &mut [u8]) -> (u32, bool) {
            let (path, at) = self.open[&h];
            let data = &self.files[&path];
            let n = into.len().min(self.chunk).min(data.len() - at);
            into[..n].copy_from_slice(&data[at..at + n]);
            self.open.get_mut(&h).unwrap().1 += n;
            (n as u32, true)
        }
        fn write(&mut self, h: u32, from: &[u8]) -> (u32, bool) {
            let path = self.open[&h].0;
            self.files.get_mut(&path).unwrap().extend_from_slice(from);
            (from.len() as u32, true)
        }
        fn seek(&mut self, h: u32, d: i32, _: From) -> Option<u32> {
            self.open.get_mut(&h).unwrap().1 = d as usize;
            Some(d as u32)
        }
        fn size(&mut self, h: u32) -> Option<u32> {
            Some(self.files[&self.open[&h].0].len() as u32)
        }
        fn close(&mut self, h: u32) -> bool {
            self.open.remove(&h).is_some()
        }
        fn time(&mut self, _: u32) -> Option<u32> {
            Some(7)
        }
        fn set_time(&mut self, _: u32, _: u32) -> bool {
            true
        }
    }

    fn stream() -> Box<Stream<u32>> {
        Box::new(Stream {
            mode: 0,
            file: 0,
            position: 0,
            filled: 0,
            buffer: [0; BUFFER_LEN],
        })
    }

    #[test]
    fn whole_files_and_streams() {
        let mut f = Memory {
            chunk: usize::MAX,
            ..Memory::default()
        };
        let data: Vec<u8> = (0..0x4567u32)
            .map(|n| ((n * 31) >> 3) as u8 ^ n as u8)
            .collect();
        assert!(write_file(&mut f, 1, &data).unwrap().ok);
        assert_eq!(file_size(&mut f, 1).unwrap(), (0x4567, true));
        assert_eq!(file_size(&mut f, 9).unwrap(), (u32::MAX, false));
        let mut back = vec![0; 0x10];
        let r = read_file(&mut f, 1, &mut back, 0x100).unwrap();
        assert_eq!(
            (r.ok, r.bytes_read, &back[..]),
            (true, Some(0x10), &data[0x100..0x110])
        );
        assert!(copy_file(&mut f, 1, 2, &mut [0; 0x1000]).unwrap());
        assert_eq!(f.files[&2], data);

        // A stream written byte by byte and in a block, then read back the same ways.
        let mut s = stream();
        assert!(open_stream(&mut f, &mut s, 3, 1).unwrap());
        for &b in &data[..0x2100] {
            assert!(write_stream_byte(&mut f, &mut s, b).unwrap());
        }
        assert_eq!(
            write_stream_block(&mut f, &mut s, &data[0x2100..], 0)
                .unwrap()
                .0,
            0x2467
        );
        assert!(close_stream(&mut f, &mut s, u32::MAX).unwrap());
        assert_eq!(f.files[&3], data);
        let mut f2 = Memory {
            chunk: 0x1234,
            files: f.files.clone(),
            ..Memory::default()
        };
        let mut s = stream();
        assert!(open_stream(&mut f2, &mut s, 3, 0).unwrap());
        let mut first = vec![0; 0x3000];
        assert_eq!(
            read_stream_block(&mut f2, &mut s, &mut first).unwrap(),
            (0x3000, false)
        );
        let mut rest = Vec::new();
        loop {
            let r = read_stream_byte(&mut f2, &mut s).unwrap();
            match r.byte() {
                Some(b) => rest.push(b),
                None => break assert!(!r.failed()),
            }
        }
        first.extend(rest);
        assert_eq!(first, data);
        assert!(open_stream(&mut f2, &mut s, 3, 5).is_err());
        let w = read_whole_file(&mut f2, 3, &mut [0; 0x10]).unwrap();
        assert_eq!(
            (w.opened, w.bytes, w.read_ok, w.closed),
            (true, 0x10, true, true)
        );
        assert!(!read_whole_file(&mut f2, 8, &mut [0; 4]).unwrap().opened);
    }

    #[test]
    fn paths() {
        assert_eq!(
            match_path(b"files\\d\\*.loc", b"eng.loc"),
            b"files\\d\\eng.loc\0"
        );
        assert_eq!(match_path(b"\\*.x", b"a.x"), b"a.x\0");
        assert_eq!(
            packed_level_path(b"srclvls\\s1\\x").unwrap(),
            b"files\\levels\\s1\\x.sq\0"
        );
        assert!(packed_level_path(b"a").is_err());
    }
}
