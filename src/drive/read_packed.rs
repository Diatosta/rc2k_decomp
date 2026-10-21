//! `drive::read_packed` (0x41EEB5): ecx bytes from the file eax into ebx: a count of 0 or
//! more is a plain read (its `jns` goes straight to `files::read`); below 0, -ecx packed
//! bytes read into `g_pngInfo` and unpacked into ebx (`drive::unpack_rle`), eax the bytes
//! unpacked.
//!
//! In:  eax the file, ebx the buffer, ecx the count (below 0: packed)
//! Out: eax the bytes read or unpacked, CF set when the read failed (ZF from `test eax`
//!      after an unpacking); a plain read keeps ebx, ecx, edx, a packed one every register
//!      but eax
//!
//! Caller: `drive::read_replay_frame`. It begins with a jump out (`jns files::read`):
//! hooked by a private copy (0x2F bytes, the jump and the two calls relocated) and a
//! hand-made patch.
//! Refuses what `unpack_rle` refuses (nothing written then but the packed bytes).
//! Shadow mode: the file is put back where it was after the Rust version's read.
//! Self-test: the original and the Rust version on a test file (written to the temp
//! folder, read from its start each time): plain and packed reads.

use crate::errors::Refused;
use crate::files::handle::read;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::a;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_ALWAYS, CreateFileA, DeleteFileA, FILE_ATTRIBUTE_NORMAL, FILE_BEGIN, FILE_CURRENT,
    GetTempPathA, SetFilePointer,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::read_packed", rva(0x41EEB5), thunk, &ORIGINAL)
    .private_copy(0x2F)
    .relocate_calls(&[0x2, 0x17, 0x20])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `g_pngInfo`: the packed bytes' buffer.
pub const PACKED: usize = 0x93E8B0;

/// The Rust version: (eax, CF, ZF when it unpacked).
// Converted: `rc2k_core::drive::race_steps::packed_bytes` (the unpacking:
// `rc2k_core::drive::rle`).
pub unsafe fn read_packed(
    file: u32,
    buffer: u32,
    count: u32,
) -> Result<(u32, bool, Option<bool>), Refused> {
    let Some(packed) = rc2k_core::drive::race_steps::packed_bytes(count) else {
        let (n, ok) = unsafe { read(file, buffer as *mut u8, count) };
        return Ok((n, !ok, None));
    };
    let (n, ok) = unsafe { read(file, a(PACKED) as *mut u8, packed) };
    if !ok {
        return Ok((n, true, None));
    }
    let out = unsafe { crate::drive::unpack_rle::unpack_rle(a(PACKED), buffer) }?;
    Ok((out, false, Some(out == 0)))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (eax, carry, _) = unsafe { read_packed(r.eax, r.ebx, r.ecx) }?;
    Ok(Left::carry(carry).with(EAX, eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    let n = if (r.ecx as i32) < 0 {
        0x10000
    } else {
        r.ecx as usize
    };
    // In shadow mode the Rust version reads first: the file goes back to where it was, so
    // that the original (whose bytes the game gets) reads the same ones.
    let file = r.eax;
    let place = unsafe { SetFilePointer(file as _, 0, std::ptr::null_mut(), FILE_CURRENT) };
    let rust_in_place = |r: &Regs| {
        let left = rust(r);
        if shadow() {
            unsafe { SetFilePointer(file as _, place as i32, std::ptr::null_mut(), FILE_BEGIN) };
        }
        left
    };
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(r.ebx, n), (a(PACKED), 0x400)],
        &rust_in_place,
    );
}

/// A file in the temp folder with `bytes`, removed when dropped (for the tests).
pub struct TestFile {
    pub handle: u32,
    path: Vec<u8>,
}

impl TestFile {
    pub fn new(name: &str, bytes: &[u8]) -> Option<Self> {
        let mut dir = [0u8; 260];
        let n = unsafe { GetTempPathA(dir.len() as u32, dir.as_mut_ptr()) } as usize;
        if n == 0 || n >= dir.len() {
            return None;
        }
        let mut path = dir[..n].to_vec();
        path.extend_from_slice(name.as_bytes());
        path.push(0);
        let handle = unsafe {
            CreateFileA(
                path.as_ptr(),
                0xC000_0000,
                0,
                std::ptr::null(),
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        } as u32;
        if handle == u32::MAX {
            return None;
        }
        let file = Self { handle, path };
        let (_, ok) =
            unsafe { crate::files::handle::write(handle, bytes.as_ptr(), bytes.len() as u32) };
        ok.then_some(file)
    }

    /// Back to the start.
    pub fn rewind(&self) {
        unsafe { SetFilePointer(self.handle as _, 0, std::ptr::null_mut(), 0) };
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        unsafe {
            crate::files::handle::close(self.handle);
            DeleteFileA(self.path.as_ptr());
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut bytes: Vec<u8> = vec![0xFD, 1, 2, 3, 0x05, 0x77, 0];
    bytes.extend((0..0x40).map(|x| x as u8 ^ 0x5A));
    let Some(file) = TestFile::new("rc2k_read_packed.tmp", &bytes) else {
        return t.skip("no test file in the temp folder");
    };
    let saved = crate::service::harness::capture(&[(a(PACKED), 0x40)]);
    let mut out = vec![0xAAu8; 0x100];
    let at = out.as_mut_ptr() as u32;
    for count in [7u32, 0x20, 0, (-7i32) as u32, (-6i32) as u32] {
        let set = || {
            file.rewind();
            unsafe {
                std::ptr::write_bytes(at as *mut u8, 0xAA, 0x100);
                std::ptr::write_bytes(a(PACKED) as *mut u8, 0, 0x40);
            }
        };
        check_state(
            &mut t,
            &format!("count {count:#x}"),
            call_original,
            Regs {
                eax: file.handle,
                ebx: at,
                ecx: count,
                ..Regs::SENTINEL
            },
            &set,
            &[(at, 0x100), (a(PACKED), 0x40)],
            &rust,
        );
    }
    crate::service::harness::restore(&[(a(PACKED), 0x40)], &saved);
    drop(file);
    t.finish()
}
