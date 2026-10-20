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
//!
//! Shadow mode: the Rust version, then the original on the same buffer, compared (the
//! buffer, eax, carry, zero). Self-test: a copy whose `read_file` is a stand-in handing it
//! streams of the test's (every code, the refill edges, the odd sizes, a file filling the
//! buffer, bad magic, a failed read), and the stage 1 course data read by both.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("files::read_packed_file", rva(0x41BA71), thunk, &ORIGINAL)
    .private_copy(0x765)
    .relocate_calls(&[0x3])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

use rc2k_core::packed::MAGIC;
const READ_FILE: usize = 0x402BB6; // files::read_file

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
/// written (from the buffer's start). Converted: `rc2k_core::packed::unpack`.
pub unsafe fn unpack(buffer: u32, room: u32, size: u32) -> Result<u32, Refused> {
    let bytes = unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, room as usize) };
    Ok(rc2k_core::packed::unpack(bytes, size as usize)? as u32)
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        match unsafe { read_packed_file(r, &mut game_read) } {
            Ok(o) => o.apply(r),
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let region = [(r.ebx, r.ecx as usize)];
    let mine = match unsafe { read_packed_file(r, &mut game_read) } {
        Ok(o) => o,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    let my_buffer = crate::service::harness::capture(&region);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = Outcome::of(&o);
    let same = crate::service::harness::capture(&region) == my_buffer;
    if mine != theirs || !same || !o.preserved_except(r, crate::framework::EAX) {
        HOOK.mismatch(|| {
            format!(
                "{} bytes: original {theirs:x?}; rust {mine:x?}; same buffer {same}",
                r.ecx
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test ----

/// What the stand-in for `read_file` hands over: the file, or None for a failed read.
static FILE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

fn stand_in_read(_place: u32, r: &mut Regs) {
    match &*FILE.lock().unwrap() {
        Some(file) => {
            let n = file.len().min(r.ecx as usize);
            unsafe { std::ptr::copy_nonoverlapping(file.as_ptr(), r.ebx as *mut u8, n) };
            r.eax = n as u32;
            r.set_carry(false);
            r.set_zero(true);
        }
        None => {
            r.set_carry(true);
            r.set_zero(false);
        }
    }
}

fn rust_read(r: &mut Regs) -> Result<(), Refused> {
    stand_in_read(0, r);
    Ok(())
}

#[derive(Clone, Copy)]
enum Code {
    Zero,
    Back8(u32),
    Back12(u32),
    Short(u32),
    Signed(i32),
    Long(u32),
}

/// A packed file of the codes, with `extra` odd bytes between the header and the stream.
fn pack(codes: &[Code], extra: usize) -> Vec<u8> {
    let mut bits: Vec<bool> = Vec::new();
    let mut push = |v: u32, n: u32| {
        for k in (0..n).rev() {
            bits.push(v >> k & 1 != 0);
        }
    };
    for &c in codes {
        match c {
            Code::Zero => push(0, 1),
            Code::Back8(n) => push(0b10 << 8 | n, 10),
            Code::Back12(n) => push(0b1100 << 12 | n, 16),
            Code::Short(v) => {
                push(0b1101, 4);
                push(v, 17)
            }
            Code::Signed(v) => {
                push(0b1110, 4);
                push(v as u32 & 0x0FFF_FFFF, 28)
            }
            Code::Long(v) => {
                push(0b1111, 4);
                push(v, 32)
            }
        }
    }
    let mut file = Vec::new();
    for d in [MAGIC, 0, 0x5_0000, codes.len() as u32] {
        file.extend(d.to_le_bytes());
    }
    // The odd bytes go after the header: the original's move leaves the stream at
    // +0x10 + (size & 3) of the file.
    file.extend((0..extra).map(|n| 0x40 + n as u8));
    for word in bits.chunks(32) {
        let mut d = 0u32;
        for (k, &b) in word.iter().enumerate() {
            d |= (b as u32) << (31 - k);
        }
        file.extend(d.to_le_bytes());
    }
    file
}

fn mixed(seed: u32) -> Vec<Code> {
    let mut s = seed;
    let mut next = move || {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        s >> 3
    };
    let mut codes = vec![Code::Long(0x1234_5678), Code::Short(0x1_FFFF)];
    for n in 2..400u32 {
        codes.push(match next() % 7 {
            0 => Code::Zero,
            1 => Code::Back8(next() % n.min(0x100)),
            2 => Code::Back12(next() % n.min(0x1000)),
            3 => Code::Short(next() & 0x1_FFFF),
            4 => Code::Signed(((next() << 4) as i32) >> 4),
            5 => Code::Signed(-(next() as i32 & 0x7FF_FFFF)),
            _ => Code::Long(next()),
        });
    }
    codes
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    use crate::redirect::RedirectedCopy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(stand_in_read);
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x41BA71)),
            0x765,
            &[0x60, 0x33, 0xD2, 0xE8, 0x3D, 0x71, 0xFE, 0xFF],
            &[0x3],
            &|c| (c == rva(READ_FILE)).then(|| stand_in(0)),
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regs = |r: &Regs| [r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let literal: Vec<Code> = (0..100)
        .map(|n| Code::Long((n * 0x0101_0101) ^ 0xA5A5))
        .collect();
    let mut cases: Vec<(String, Option<Vec<u8>>, u32)> = Vec::new();
    for extra in 0..4 {
        cases.push((
            format!("mixed, {extra} odd bytes"),
            Some(pack(&mixed(7 + extra as u32), extra)),
            0x4000,
        ));
    }
    let tight = pack(&literal, 0);
    let tight_room = tight.len() as u32;
    cases.push((
        "literals filling the buffer".into(),
        Some(tight),
        tight_room,
    ));
    cases.push(("one zero".into(), Some(pack(&[Code::Zero], 0)), 0x40));
    let mut bad = pack(&mixed(3), 0);
    bad[0] = b'X';
    cases.push(("not SQSL".into(), Some(bad), 0x4000));
    cases.push(("a failed read".into(), None, 0x4000));
    let mut buffer = vec![0u8; 0x4000];
    let b = buffer.as_mut_ptr() as u32;
    for (what, file, room) in cases {
        *FILE.lock().unwrap() = file;
        let before = Regs {
            eax: 0x0BAD_0BAD,
            ebx: b,
            ecx: room,
            ..Regs::SENTINEL
        };
        let fill = || unsafe { std::ptr::write_bytes(b as *mut u8, 0xAA, 0x4000) };
        fill();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Outcome::of(&ro), regs(&ro), buffer.clone());
        fill();
        let mine = unsafe { read_packed_file(&before, &mut rust_read) };
        let mine = (mine, regs(&before), buffer.clone());
        let same =
            mine.0.as_ref().ok() == Some(&theirs.0) && mine.1 == theirs.1 && mine.2 == theirs.2;
        t.check(same, || {
            let at = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "{what}: original {:x?}; rust {:x?}; first difference at {at:x?}",
                theirs.0, mine.0
            )
        });
    }
    // No dwords: refused.
    *FILE.lock().unwrap() = Some(pack(&[], 0));
    let none = Regs {
        ebx: b,
        ecx: 0x100,
        ..Regs::SENTINEL
    };
    let refused = unsafe { read_packed_file(&none, &mut rust_read) }.is_err();
    t.check(refused, || "no dwords: not refused".into());
    *FILE.lock().unwrap() = None;
    drop(copy);
    // The stage 1 course data, read by both.
    let mut big = vec![0u8; 0x10_0000];
    let path = b"files\\levels\\level01\\cdat00c.sq\0";
    let before = Regs {
        eax: path.as_ptr() as u32,
        ebx: big.as_mut_ptr() as u32,
        ecx: big.len() as u32,
        ..Regs::SENTINEL
    };
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    if ro.carry() {
        drop(big);
        return t.skip("files\\levels\\level01\\cdat00c.sq could not be read");
    }
    let theirs = (Outcome::of(&ro), regs(&ro), big.clone());
    big.fill(0);
    let mine = unsafe { read_packed_file(&before, &mut game_read) };
    let mine = (mine, regs(&before), big.clone());
    t.check(
        mine.0.as_ref().ok() == Some(&theirs.0) && mine.1 == theirs.1 && mine.2 == theirs.2,
        || {
            format!(
                "stage 1's course data: original {:x?}; rust {:x?}",
                theirs.0, mine.0
            )
        },
    );
    t.finish()
}
