//! `files::resolve_path` (0x403206): turns a game-relative path into a full path under one of
//! the two base directories.
//!
//! In:  eax = path
//! Out: eax = resolved path (a buffer), or the input pointer if the path is used as it is;
//!      everything else preserved
//! The output buffer has no bounds check. A result longer than 0x100 bytes is refused, and the
//! hook leaves it to the original.

use crate::errors::BadInput;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_ptr, game_read, hook_asm, rva, shadow};
use crate::strings::utils::bytes_str;
use crate::threading::find_thread_slot::find_thread_slot;
use std::ffi::{CStr, c_char};
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::resolve_path", rva(0x403206), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn at(p: &[u8], i: usize) -> u8 {
    p.get(i).copied().unwrap_or(0)
}

/// Ok(None) = the path is used as it is. `path` excludes the NUL; the result includes one.
/// A result that does not fit the 0x100-byte buffer is refused: the original writes past it.
pub fn resolve_path(
    path: &[u8],
    media_mode: u32,
    base_a: &[u8],
    base_b: &[u8],
    pct: &[u8],
) -> Result<Option<Vec<u8>>, BadInput> {
    if path.starts_with(b"var\\") || path.starts_with(b"save") || at(path, 1) == b':' {
        return Ok(None);
    }
    let (semi3, semi4) = (path.starts_with(b";3"), path.starts_with(b";4"));
    let use_b = !semi4 && (media_mode >= 3 || (!semi3 && media_mode == 2)); // unsigned compares
    let base = if use_b { base_b } else { base_a };
    let rel = if at(path, 0) == b';' {
        &path[2.min(path.len())..]
    } else {
        path
    };

    let mut out = base.to_vec();
    for &c in rel {
        if c == b'%' {
            out.extend_from_slice(pct)
        } else {
            out.push(c)
        }
    }
    out.push(0);
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

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let path = unsafe { cstr(r.eax as *const u8) };
    let g = unsafe { globals() };
    let Ok(mine) = resolve_path(path, g.mode, g.a, g.b, g.pct) else {
        unsafe { call_original(r) };
        return;
    };

    if !shadow() {
        if let Some(m) = mine {
            let buf = out_buffer();
            unsafe { copy_nonoverlapping(m.as_ptr(), buf, m.len()) };
            r.eax = buf as u32;
        }
        return; // None: eax unchanged
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let ok = match &mine {
        None => o.eax == r.eax,
        Some(m) => {
            o.eax == out_buffer() as u32
                && unsafe { from_raw_parts(o.eax as *const u8, m.len()) } == &m[..]
        }
    };
    if !ok {
        HOOK.mismatch(|| format!("path={}", bytes_str(path)));
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // No empty path and no lone ';': the original reads past the NUL for those.
    let paths: [&[u8]; 11] = [
        b"var\\logfile.txt\0",
        b"save1.dat\0",
        b"C:\\x.txt\0",
        b"data\\a.txt\0",
        b";3data\\a.txt\0",
        b";4data\\a.txt\0",
        b";5x\0",
        b"d\\%\\f.txt\0",
        b"%%\0",
        b";3%x\0",
        b"x\0",
    ];
    let g = unsafe { globals() };
    let buf = out_buffer();

    for p in paths {
        let snapshot: Vec<u8> = unsafe { from_raw_parts(buf, BUF_LEN) }.to_vec();
        let before = Regs {
            eax: p.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };

        let mine = resolve_path(&p[..p.len() - 1], g.mode, g.a, g.b, g.pct).unwrap_or(None);
        let now = unsafe { from_raw_parts(buf, BUF_LEN) };
        let mut want = snapshot.clone();
        let eax_want = match &mine {
            None => p.as_ptr() as u32,
            Some(m) => {
                want[..m.len()].copy_from_slice(m);
                buf as u32
            }
        };
        let preserved = ro.preserved_except(&before, EAX);

        t.check(ro.eax == eax_want && now == &want[..] && preserved, || {
            format!(
                "path={} eax={:#x} want {eax_want:#x} preserved={preserved}",
                bytes_str(&p[..p.len() - 1]),
                ro.eax
            )
        });
        unsafe { copy_nonoverlapping(snapshot.as_ptr(), buf, BUF_LEN) }; // restore game memory
    }
    t.finish()
}
