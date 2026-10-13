//! `files::read_whole_file` (0x421B54): reads a file's start into a buffer and says how
//! much it got.
//!
//! In:  eax = the path, ebx = the buffer, ecx = how many bytes at most
//! Out: carry set = the file could not be opened, read or closed (zero clear after a
//!      failed read); `dword_60B2CC` = the bytes read, 0 after a failed open or close;
//!      eax = what `CloseHandle` gave, or -1 if the file did not open; bl = 1 after a
//!      good read, 0 after a failed one; everything else preserved
//!
//! `files::open_by_mode` with mode 0, `files::read`, `files::close`.
//! 29 callers: the models, the stage's files.
//! Refuses what `files::resolve_path` refuses.
//! Shadow mode reads twice and compares.
//! Self-test: the game's settings file, whole and in part, and a file that is not
//! there.

use crate::errors::{BadInput, Refused};
use crate::files::handle::{NO_HANDLE, close, open_by_mode, read};
use crate::framework::{
    EAX, EBX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::read_whole_file", rva(0x421B54), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const BYTES_READ_RVA: usize = rva(0x60B2CC); // dword_60B2CC

// ---- Rust version ----

/// The bytes read (0 after any failure but a failed read's own count).
pub unsafe fn read_whole_file(
    path: *const u8,
    buffer: *mut u8,
    count: u32,
    ebx: u32,
) -> Result<Left, BadInput> {
    unsafe {
        let Some(file) = open_by_mode(path, 0)? else {
            game_write(BYTES_READ_RVA, 0u32);
            return Ok(Left::carry(true).with(EAX, NO_HANDLE));
        };
        let (bytes, read_ok) = read(file, buffer, count);
        game_write(BYTES_READ_RVA, bytes);
        let closed = close(file);
        if !closed {
            game_write(BYTES_READ_RVA, 0u32);
        }
        Ok(Left::carry(!closed || !read_ok)
            .with(EAX, closed as u32)
            .with(EBX, (ebx & !0xFF) | read_ok as u32))
    }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { read_whole_file(r.eax as *const u8, r.ebx as *mut u8, r.ecx, r.ebx) }?)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let left = match rust(r) {
        Ok(left) => left,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        left.apply(r);
        return;
    }
    let (buffer, count) = (r.ebx as *const u8, r.ecx);
    let mine: u32 = unsafe { game_read(BYTES_READ_RVA) };
    let my_bytes = unsafe { std::slice::from_raw_parts(buffer, mine.min(count) as usize) }.to_vec();
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs: u32 = unsafe { game_read(BYTES_READ_RVA) };
    let same = theirs == mine
        && unsafe { std::slice::from_raw_parts(buffer, mine.min(count) as usize) } == my_bytes;
    // eax after a close is whatever the system call left: only that it is not 0.
    let mut seen = o;
    if seen.eax != NO_HANDLE && seen.eax != 0 {
        seen.eax = 1;
    }
    if !same || !left.matches(r, &seen) {
        HOOK.mismatch(|| {
            format!(
                "path={:?} count={count:#x}: read {theirs:#x} / {mine:#x}, same bytes {same}; original eax {:#x} ebx {:#x} carry {}; rust {left:x?}",
                unsafe { std::ffi::CStr::from_ptr(r.eax as *const _) },
                o.eax,
                o.ebx,
                o.carry()
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: u32 = unsafe { game_read(BYTES_READ_RVA) };
    for (path, count) in [
        (c"var\\ral.cfg", 0x4000u32),
        (c"var\\ral.cfg", 7),
        (c"var\\ral.cfg", 0),
        (c"var\\rc2k_hook_no_such_file.bin", 0x100),
    ] {
        let run = |rust_version: bool| unsafe {
            let mut buffer = vec![0xAAu8; 0x4000];
            game_write(BYTES_READ_RVA, 0xAAAA_AAAAu32);
            let before = Regs {
                eax: path.as_ptr() as u32,
                ebx: buffer.as_mut_ptr() as u32,
                ecx: count,
                ..Regs::SENTINEL
            };
            let mut after = before;
            let ok = if rust_version {
                rust(&before).is_ok_and(|left| {
                    left.apply(&mut after);
                    true
                })
            } else {
                call_original(&mut after);
                if after.eax != NO_HANDLE && after.eax != 0 {
                    after.eax = 1;
                }
                true
            };
            (
                ok,
                after.eax,
                // The buffers of the two runs differ: only what became of bl.
                (after.ebx != before.ebx).then_some(after.ebx & 0xFF),
                after.ebx >> 8 == before.ebx >> 8,
                after.carry(),
                after.preserved_except(&before, EAX | EBX),
                game_read::<u32>(BYTES_READ_RVA),
                buffer,
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine, || {
            format!(
                "{path:?}, {count:#x} bytes: original {:x?}; rust {:x?}; same bytes {}",
                (
                    theirs.0, theirs.1, theirs.2, theirs.3, theirs.4, theirs.5, theirs.6
                ),
                (mine.0, mine.1, mine.2, mine.3, mine.4, mine.5, mine.6),
                theirs.7 == mine.7
            )
        });
    }
    unsafe { game_write(BYTES_READ_RVA, saved) };
    t.finish()
}
