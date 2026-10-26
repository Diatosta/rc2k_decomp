//! `files::read_packed_file` (0x41BA71): a file packed by the game's "SQSL" scheme read and
//! unpacked in place.
//!
//! In:  eax = path, ebx = buffer, ecx = its bytes
//! Out: eax = the bytes unpacked, carry clear, zero flag = none; on a failed read or a
//!      file not "SQSL", eax = 0, carry set, zero clear; everything else preserved
//!
//! The file (up to ecx bytes, `files::read_file`) begins "SQSL", two dwords, and the count
//! of dwords it unpacks to. Its dwords are moved to the buffer's end (backwards, so a file
//! filling the buffer stays in place); from +0x10 of them a stream of bits (each dword's
//! highest first) is read and the dwords written from the buffer's start (both + the
//! size's odd bytes: the move shifts the file by them), each by a code: 0 a zero; 10 and 8 bits n, the
//! dword n + 1 back; 1100 and 12 bits, likewise; 1101 and 17 bits, that number; 1110, a
//! sign bit and 27 bits, that number (28 bits, signed); 1111 and 32 bits, that number. Read
//! from the course data (`files::read_level_file`). Refuses a count of 0 (the original
//! counts round), and a stream that reads or writes past the buffer or refers back past its
//! start (the original would carry on in other memory).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;

use rc2k_core::files::packed::MAGIC;

/// What the original leaves: eax and two flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub eax: u32,
    pub carry: bool,
    pub zero: bool,
}

impl Outcome {
    pub fn apply(&self, r: &mut Regs) {
        r.eax = self.eax;
        r.set_carry(self.carry);
        r.set_zero(self.zero);
    }

    pub fn of(r: &Regs) -> Self {
        Self {
            eax: r.eax,
            carry: r.carry(),
            zero: crate::audio::buffer_play::zero(r),
        }
    }
}

/// `files::read_file`'s call (eax the path, ebx the buffer, ecx the bytes, edx 0): eax,
/// carry and zero as it leaves them.
pub type Read<'a> = &'a mut dyn FnMut(&mut Regs) -> Result<(), Refused>;

/// In the game: `files::read_file` in Rust.
pub fn game_read(r: &mut Regs) -> Result<(), Refused> {
    let got = unsafe { crate::files::read_file::read_file(r.eax as _, r.ebx as _, r.ecx, r.edx) }?;
    if let Some(n) = got.bytes_read {
        r.eax = n;
    }
    r.set_carry(!got.ok);
    r.set_zero(got.ok);
    Ok(())
}

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The unpacking of a file of `size` bytes read into the buffer of `room`: the bytes
/// written (from the buffer's start). Converted: `rc2k_core::files::packed::unpack`.
pub unsafe fn unpack(buffer: u32, room: u32, size: u32) -> Result<u32, Refused> {
    let bytes = unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, room as usize) };
    Ok(rc2k_core::files::packed::unpack(bytes, size as usize)? as u32)
}

/// `files::read_packed_file` (see the header): eax the path, ebx the buffer, ecx its bytes.
pub unsafe fn read_packed_file(entry: &Regs, read: Read) -> Result<Outcome, Refused> {
    let mut r = Regs { edx: 0, ..*entry };
    read(&mut r)?;
    let failed = |zero| Outcome {
        eax: 0,
        carry: true,
        zero,
    };
    if r.carry() {
        return Ok(failed(crate::audio::buffer_play::zero(&r)));
    }
    let (buffer, room) = (entry.ebx, entry.ecx);
    if unsafe { get(buffer) } != MAGIC {
        return Ok(failed(false));
    }
    let bytes = unsafe { unpack(buffer, room, r.eax) }?;
    Ok(Outcome {
        eax: bytes,
        carry: false,
        zero: bytes == 0,
    })
}
