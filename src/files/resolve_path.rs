//! `files::resolve_path` (0x403206): turns a game-relative path into a full path under one of
//! the two base directories.
//!
//! In:  eax = path
//! Out: eax = resolved path (a buffer), or the input pointer if the path is used as it is;
//!      everything else preserved
//! The output buffer has no bounds check. A result longer than 0x100 bytes is refused, and the
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{game_ptr, game_read, rva};
use crate::threading::find_thread_slot::find_thread_slot;
use std::ffi::{CStr, c_char};
use std::ptr::copy_nonoverlapping;

// ---- game data ----

const MEDIA_MODE_RVA: usize = rva(0x5189A4); // g_mediaMode
const BASE_A_RVA: usize = rva(0x5189A8); // base directory A (string)
const BASE_B_RVA: usize = rva(0x5295B4); // base directory B (string)
const PCT_RVA: usize = rva(0x518AA8); // string that replaces each '%'
const GLOBAL_BUF_RVA: usize = rva(0x94E9B0); // output buffer for threads without a slot

const BUF_LEN: usize = 0x100; // ThreadSlot + 0x10 .. + 0x110

unsafe fn cstr<'a>(ptr: *const u8) -> &'a [u8] {
    unsafe { CStr::from_ptr(ptr as *const c_char) }.to_bytes()
}

struct Globals<'a> {
    mode: u32,
    a: &'a [u8],
    b: &'a [u8],
    pct: &'a [u8],
}
unsafe fn globals<'a>() -> Globals<'a> {
    unsafe {
        Globals {
            mode: game_read(MEDIA_MODE_RVA),
            a: cstr(game_ptr(BASE_A_RVA)),
            b: cstr(game_ptr(BASE_B_RVA)),
            pct: cstr(game_ptr(PCT_RVA)),
        }
    }
}

/// slot + 0x10 if the current thread has a ThreadSlot, else the global buffer.
fn out_buffer() -> *mut u8 {
    let slot = unsafe { find_thread_slot() };
    if slot.is_null() {
        game_ptr(GLOBAL_BUF_RVA)
    } else {
        unsafe { (slot as *mut u8).add(0x10) }
    }
}

// ---- Rust version ----

/// Ok(None) = the path is used as it is. `path` excludes the NUL; the result includes one.
/// A result that does not fit the 0x100-byte buffer is refused: the original writes past it.
pub fn resolve_path(
    path: &[u8],
    media_mode: u32,
    base_a: &[u8],
    base_b: &[u8],
    pct: &[u8],
) -> Result<Option<Vec<u8>>, BadInput> {
    // Converted: `rc2k_core::files::paths::resolve`.
    let bases = rc2k_core::files::paths::Bases {
        media_mode,
        a: base_a,
        b: base_b,
        percent: pct,
    };
    let Some(out) = rc2k_core::files::paths::resolve(path, &bases) else {
        return Ok(None);
    };
    if out.len() > BUF_LEN {
        return Err(BadInput(
            "resolved path longer than the buffer: the original writes past it",
        ));
    }
    Ok(Some(out))
}

/// The path as the game would open it, for Rust callers: the pointer given if the path is used
/// as it is, else the resolved path written to this thread's buffer.
pub unsafe fn resolve_path_in_game(path: *const u8) -> Result<*const u8, BadInput> {
    unsafe {
        let g = globals();
        let Some(resolved) = resolve_path(cstr(path), g.mode, g.a, g.b, g.pct)? else {
            return Ok(path);
        };
        let buf = out_buffer();
        copy_nonoverlapping(resolved.as_ptr(), buf, resolved.len());
        Ok(buf)
    }
}
