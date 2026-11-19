//! The module's exports for the page: plain C functions on the module's memory (no
//! wasm-bindgen). The page copies its input in with `alloc`, calls, and reads the answer from
//! `result_ptr` / `result_len`; every call that can fail returns 0 for success, else the
//! result is the message. One thread: the module lives in a worker or the page's own thread.

use crate::files::Files;
use crate::{Read, Scan};
use rc2k_core::saves::code::parts::Density;
use std::cell::RefCell;

thread_local! {
    static RESULT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static SCAN: RefCell<Scan> = RefCell::new(Scan::default());
}

fn set_result(bytes: Vec<u8>) {
    RESULT.with(|r| *r.borrow_mut() = bytes);
}

fn fail(message: impl Into<String>) -> i32 {
    set_result(message.into().into_bytes());
    1
}

/// # Safety
/// `ptr` and `len` name bytes the caller got from `alloc` and filled.
unsafe fn input<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    if len == 0 {
        return &[];
    }
    // SAFETY: as the caller promises.
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut v = vec![0u8; len.max(1)];
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    ptr
}

/// # Safety
/// `ptr` came from `alloc(len)` and is not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    // SAFETY: `alloc` made it a vector of this length (at least one).
    drop(unsafe { Vec::from_raw_parts(ptr, len.max(1), len.max(1)) });
}

#[unsafe(no_mangle)]
pub extern "C" fn result_ptr() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn result_len() -> usize {
    RESULT.with(|r| r.borrow().len())
}

/// The QR codes' texts of the files (the `Files` bytes), at `now` seconds from 1970, the
/// codes `large` or small; the result is the texts, one to a line.
///
/// # Safety
/// `ptr` / `len`: bytes from `alloc`, filled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn encode(ptr: *const u8, len: usize, now: u32, large: i32) -> i32 {
    // SAFETY: as promised.
    let bytes = unsafe { input(ptr, len) };
    let Some(files) = Files::from_bytes(bytes) else {
        return fail("The saves could not be read.");
    };
    let density = if large != 0 {
        Density::Dense
    } else {
        Density::Small
    };
    match crate::parts(&files, now, density) {
        Ok(parts) => {
            set_result(parts.join("\n").into_bytes());
            0
        }
        Err(e) => fail(e),
    }
}

/// The modules of the QR code of a text: the side (four bytes) and `side * side` bytes, 1 for dark.
///
/// # Safety
/// `ptr` / `len`: bytes from `alloc`, filled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qr_modules(ptr: *const u8, len: usize) -> i32 {
    // SAFETY: as promised.
    let text = String::from_utf8_lossy(unsafe { input(ptr, len) }).into_owned();
    let Some(m) = crate::modules(&text) else {
        return fail("The text does not fit a QR code.");
    };
    let mut out = (m.side as u32).to_le_bytes().to_vec();
    out.extend(m.dark.iter().map(|&d| u8::from(d)));
    set_result(out);
    0
}

/// Reads a camera frame (RGBA, `width` x `height`) for a QR code and takes it into the scan: 0
/// nothing read, 1 a new part (or a whole code), 2 a part already held, 3 a part of another
/// code, 4 a code that is not the game's. The result is "held,count".
///
/// # Safety
/// `ptr`: `width * height * 4` bytes from `alloc`, filled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scan_frame(ptr: *const u8, width: usize, height: usize) -> i32 {
    // SAFETY: as promised.
    let rgba = unsafe { input(ptr, width * height * 4) };
    let Some(text) = crate::read_frame(width, height, rgba) else {
        return 0;
    };
    scan_text_inner(&text)
}

/// Takes a text (read from a QR code by other means) into the scan; the codes as `scan_frame`.
///
/// # Safety
/// `ptr` / `len`: bytes from `alloc`, filled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scan_text(ptr: *const u8, len: usize) -> i32 {
    // SAFETY: as promised.
    let text = String::from_utf8_lossy(unsafe { input(ptr, len) }).into_owned();
    scan_text_inner(&text)
}

fn scan_text_inner(text: &str) -> i32 {
    SCAN.with(|s| {
        let mut s = s.borrow_mut();
        let read = s.take(text);
        let (held, count) = s.progress();
        set_result(format!("{held},{count}").into_bytes());
        match read {
            Read::New => 1,
            Read::Again => 2,
            Read::Other => 3,
            Read::Not => 4,
        }
    })
}

/// Forgets the parts collected.
#[unsafe(no_mangle)]
pub extern "C" fn scan_reset() {
    SCAN.with(|s| *s.borrow_mut() = Scan::default());
}

/// Whether the scan holds every part: 0 not yet, 1 complete (the result is what the code holds,
/// one to a line), -1 the parts do not fit (the result is why).
#[unsafe(no_mangle)]
pub extern "C" fn scan_whole() -> i32 {
    SCAN.with(|s| match s.borrow().whole() {
        None => 0,
        Some(Err(e)) => {
            set_result(e.into_bytes());
            -1
        }
        Some(Ok(whole)) => match crate::describe(&whole) {
            Ok(lines) => {
                set_result(lines.join("\n").into_bytes());
                1
            }
            Err(e) => {
                set_result(e.into_bytes());
                -1
            }
        },
    })
}

/// The files the complete scan writes, given the browser's files now (the `Files` bytes) and
/// the files' stamp: the result is the files to write (`Files` bytes), their report as the file
/// named ":report".
///
/// # Safety
/// `ptr` / `len`: bytes from `alloc`, filled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scan_import(ptr: *const u8, len: usize, packed_time: u32) -> i32 {
    // SAFETY: as promised.
    let bytes = unsafe { input(ptr, len) };
    let Some(files) = Files::from_bytes(bytes) else {
        return fail("The saves could not be read.");
    };
    let whole = SCAN.with(|s| s.borrow().whole());
    let Some(Ok(whole)) = whole else {
        return fail("The code is not complete.");
    };
    match crate::import(&whole, &files, packed_time) {
        Ok((mut out, report)) => {
            out.set(":report", report.join("\n").into_bytes());
            set_result(out.to_bytes());
            0
        }
        Err(e) => fail(e),
    }
}
