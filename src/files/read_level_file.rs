//! `files::read_level_file` (0x41BA2E): a file read as it is, or, when it is not there,
//! its packed copy under `files\levels`.
//!
//! In:  eax = path (`srclvls\...`), ebx = buffer, ecx = its bytes
//! Out: with the file there, as `files::read_file` leaves them (edx kept); else as
//!      `files::read_packed_file` leaves them, on the path `files\levels\...` + ".sq"
//!      (built in `g_packedPath`, 0x5BE9BD: the path copied to +5, ".sq" after it, and
//!      "files\leve" over its first ten bytes, so that `srclvls\` becomes `files\levels\`)
//!
//! One caller, `sub_4455FC` (loadlevel, the course data). Shadow mode runs the original
//! alone (it reads the file). Self-test: a file that is there (stage 1's `localdat`
//! through `files\`), one that is not (stage 1's `srclvls` course data, read packed), and
//! one with neither, read by both; the path buffer put back afterwards.

use crate::errors::Refused;
use crate::files::read_packed_file::{Outcome, Read, game_read, read_packed_file};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("files::read_level_file", rva(0x41BA2E), thunk, &ORIGINAL)
    .private_copy(0x43)
    .relocate_calls(&[0x0, 0xE, 0x33, 0x3C])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const PACKED_PATH_RVA: usize = rva(0x5BE9BD); // g_packedPath
const PREFIX: &[u8; 10] = b"files\\leve";

/// Whether a file can be opened (`files::file_exists`).
pub type Exists<'a> = &'a mut dyn FnMut(u32) -> Result<bool, Refused>;

pub fn game_exists(path: u32) -> Result<bool, Refused> {
    Ok(unsafe { crate::files::file_exists::file_exists(path as *const u8) }?)
}

/// `files::read_level_file` (see the header): eax, carry and zero.
pub unsafe fn read_level_file(
    entry: &Regs,
    exists: Exists,
    read: Read,
) -> Result<Outcome, Refused> {
    if exists(entry.eax)? {
        let mut r = Regs { edx: 0, ..*entry };
        read(&mut r)?;
        return Ok(Outcome::of(&r));
    }
    unsafe {
        let path = game_addr(PACKED_PATH_RVA) as *mut u8;
        let cursor = crate::strings::str_append::str_append(entry.eax as *const u8, path.add(5));
        (cursor as *mut [u8; 4]).write_unaligned(*b".sq\0");
        (path as *mut [u8; 10]).write_unaligned(*PREFIX);
        read_packed_file(
            &Regs {
                eax: path as u32,
                ..*entry
            },
            read,
        )
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { read_level_file(r, &mut game_exists, &mut game_read) } {
        Ok(o) => o.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let paths: Region = (game_addr(PACKED_PATH_RVA) as u32, 0x80);
    let saved = capture(&[paths]);
    let mut buffer = vec![0u8; 0x10_0000];
    let b = buffer.as_mut_ptr() as u32;
    let regs = |r: &Regs| [r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (what, path) in [
        ("a file there", &b"files\\levels\\level01\\localdat\0"[..]),
        ("a packed one", b"srclvls\\level01\\cdat00c\0"),
        ("neither", b"srclvls\\level99\\cdat00c\0"),
    ] {
        let before = Regs {
            eax: path.as_ptr() as u32,
            ebx: b,
            ecx: buffer.len() as u32,
            ..Regs::SENTINEL
        };
        let reset = || unsafe {
            restore(&[paths], &saved);
            std::ptr::write_bytes(b as *mut u8, 0xAA, 0x10_0000);
        };
        reset();
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = (
            Outcome::of(&ro),
            regs(&ro),
            buffer.clone(),
            capture(&[paths]),
        );
        reset();
        let mine = unsafe { read_level_file(&before, &mut game_exists, &mut game_read) };
        let mine = (mine, regs(&before), buffer.clone(), capture(&[paths]));
        t.check(
            mine.0.as_ref().ok() == Some(&theirs.0)
                && mine.1 == theirs.1
                && mine.2 == theirs.2
                && mine.3 == theirs.3,
            || {
                format!(
                    "{what}: original {:x?}; rust {:x?}; same buffer {}, same path {}",
                    theirs.0,
                    mine.0,
                    mine.2 == theirs.2,
                    mine.3 == theirs.3
                )
            },
        );
    }
    restore(&[paths], &saved);
    drop(buffer);
    t.finish()
}
