//! `files::read_stream_block` (0x402DA3): reads a number of bytes from a buffered stream.
//!
//! In:  ebx = where they go, ecx = how many, esi = the stream
//! Out: eax = how many were read; carry = a read failed; everything else preserved
//! One `read_stream_byte` per byte. The end of the file stops it without the carry flag:
//! the caller has to look at eax. Three callers, the picture readers (0x80 bytes of header).
//! Shadow mode runs the original alone (the file's position moves) and checks only what it
//! leaves in the registers. The name is a reading.

use crate::errors::BadInput;
use crate::files::handle::{close, open_by_mode};
use crate::files::stream::{
    LastPath, Stream, TEST_LEN, TEST_PATH, make_test_file, remove_test_file,
};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::read_stream_block", rva(0x402DA3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Returns (the bytes read, whether a read failed).
pub unsafe fn read_stream_block(
    stream: *mut Stream,
    to: *mut u8,
    count: u32,
) -> Result<(u32, bool), BadInput> {
    // Converted: `rc2k_core::files::io::read_stream_block`.
    unsafe {
        Ok(rc2k_core::files::io::read_stream_block(
            &mut crate::files::handle::GameFiles::new(),
            crate::files::stream::core_stream(stream),
            std::slice::from_raw_parts_mut(to, count as usize),
        )?)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        let (done, failed) = match unsafe {
            read_stream_block(r.esi as *mut Stream, r.ebx as *mut u8, r.ecx)
        } {
            Ok(left) => left,
            Err(e) => return unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        };
        r.eax = done;
        r.set_carry(failed);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax > r.ecx || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| format!("count={}: the original read {}", r.ecx, o.eax));
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test: blocks of several sizes read one after the other from the test file through
// both, each with a stream and a handle of its own; then a stream whose handle is 0 ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !make_test_file() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved = LastPath::save();
    let files = unsafe {
        (
            open_by_mode(TEST_PATH.as_ptr(), 0),
            open_by_mode(TEST_PATH.as_ptr(), 0),
        )
    };
    let (Ok(Some(file_o)), Ok(Some(file_r))) = files else {
        saved.restore();
        remove_test_file();
        return t.skip("cannot open the test file");
    };
    let mut theirs = Stream::filler();
    let mut ours = Stream::filler();
    // The last three: across the end of the file, and past it.
    let sizes = [
        0u32,
        1,
        0x80,
        0x1F7F,
        3,
        0x2000,
        0x300,
        TEST_LEN as u32,
        5,
        0,
    ];
    for (stream, file) in [(&mut theirs, file_o), (&mut ours, file_r)] {
        stream.mode = 0;
        stream.file = file;
        stream.position = 0;
        stream.filled = 0;
    }
    let mut check = |what: String, theirs: &mut Stream, ours: &mut Stream, count: u32| {
        let mut out_o = vec![0xAAu8; count as usize + 8];
        let mut out_r = out_o.clone();
        let before = Regs {
            ebx: out_o.as_mut_ptr() as u32,
            ecx: count,
            esi: theirs as *mut Stream as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { read_stream_block(ours, out_r.as_mut_ptr(), count) };
        let same_stream = (theirs.position, theirs.filled) == (ours.position, ours.filled)
            && theirs.buffer == ours.buffer;
        let preserved = ro.preserved_except(&before, EAX);
        let ok = mine == Ok((ro.eax, ro.carry())) && out_o == out_r && same_stream && preserved;
        t.check(ok, || {
            format!(
                "{what}: original read {} carry={} position={} filled={} preserved={preserved}; rust {mine:?} position={} filled={} same bytes={}",
                ro.eax,
                ro.carry(),
                theirs.position,
                theirs.filled,
                ours.position,
                ours.filled,
                out_o == out_r
            )
        });
    };
    for (n, count) in sizes.into_iter().enumerate() {
        check(
            format!("block {n} of {count} bytes"),
            &mut theirs,
            &mut ours,
            count,
        );
    }
    unsafe {
        close(file_o);
        close(file_r);
    }

    // A read that fails, at once and after bytes that were waiting.
    for waiting in [0u32, 4] {
        for stream in [&mut theirs, &mut ours] {
            stream.file = 0;
            stream.position = 10;
            stream.filled = 10 + waiting;
        }
        check(
            format!("a handle of 0 with {waiting} bytes waiting"),
            &mut theirs,
            &mut ours,
            16,
        );
    }
    saved.restore();
    remove_test_file();
    t.finish()
}
