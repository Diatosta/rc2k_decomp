//! `files::numbered_path` (0x403292): copies a path template to the buffer at 0x94E8B0 and
//! writes a number over its last `!!`, or if there is none its last `00`.
//!
//! In:  eax = template, ebx = number
//! Out: eax = the buffer; everything else preserved
//! The number is written with `format_u32` flags 0x12: two digits, zero-padded (a larger
//! number's last two). A template with
//! neither marker is fatal error 0x80F19001 in the original. The search never looks at the
//! template's first character. Refuses an empty template (the original reads before the
//! buffer) and anything that does not fit the buffer's 0x100 bytes.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_ptr, hook_asm, rva, shadow};
use crate::strings::utils::bytes_str;
use std::ffi::{CStr, c_char};
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::numbered_path", rva(0x403292), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const BUFFER_RVA: usize = rva(0x94E8B0); // byte_94E8B0
const BUFFER_LEN: usize = 0x100; // up to the path buffer at 0x94E9B0

pub fn buffer() -> *mut u8 {
    game_ptr(BUFFER_RVA)
}

// ---- Rust version ----

/// What the buffer holds afterwards, from its start to the last byte written. `template`
/// excludes the NUL.
pub fn numbered_path(template: &[u8], number: u32) -> Result<Vec<u8>, Refused> {
    // Converted: `rc2k_core::paths::numbered`.
    let Some(out) = rc2k_core::paths::numbered(template, number)? else {
        return Err(FatalError {
            code: 0x80F1_9001,
            message: "",
        }
        .into());
    };
    if out.len() > BUFFER_LEN {
        return Err(BadInput("longer than the buffer: the original writes past it").into());
    }
    Ok(out)
}

/// For Rust callers: writes the buffer and returns it.
pub unsafe fn numbered_path_in_game(
    template: *const u8,
    number: u32,
) -> Result<*const u8, Refused> {
    unsafe {
        let path = numbered_path(cstr(template), number)?;
        copy_nonoverlapping(path.as_ptr(), buffer(), path.len());
    }
    Ok(buffer())
}

unsafe fn cstr<'a>(ptr: *const u8) -> &'a [u8] {
    unsafe { CStr::from_ptr(ptr as *const c_char) }.to_bytes()
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let template = unsafe { cstr(r.eax as *const u8) };
    let mine = match numbered_path(template, r.ebx) {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };

    if !shadow() {
        unsafe { copy_nonoverlapping(mine.as_ptr(), buffer(), mine.len()) };
        r.eax = buffer() as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = unsafe { from_raw_parts(buffer(), mine.len()) };
    if theirs != &mine[..] || o.eax != buffer() as u32 || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "template={} number={}: original {} rust {}",
                bytes_str(template),
                r.ebx,
                bytes_str(theirs),
                bytes_str(&mine)
            )
        });
    }
    r.eax = o.eax;
}

// ---- self-test: the whole buffer is compared and put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let whole = || unsafe { from_raw_parts(buffer(), BUFFER_LEN) }.to_vec();
    let saved = whole();
    let fill = || unsafe { buffer().write_bytes(0xAA, BUFFER_LEN) };

    // Every template has a marker the search can find: without one the original reports a
    // fatal error.
    let templates: [&[u8]; 7] = [
        b"savedata\\carsetup\\carset00\0",
        b"a!!b\0",
        b"x00y!!z\0",
        b"a00\0",
        b"a0000\0",
        b"!!!\0",
        b"p!!q00r!!s\0",
    ];
    for template in templates {
        for number in [0u32, 5, 42, 99, 100, 12345] {
            fill();
            let before = Regs {
                eax: template.as_ptr() as u32,
                ebx: number,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let theirs = whole();

            fill();
            let mine = unsafe { numbered_path_in_game(template.as_ptr(), number) };
            let ours = whole();

            let ok = mine.is_ok_and(|p| p as u32 == ro.eax)
                && theirs == ours
                && ro.preserved_except(&before, EAX);
            t.check(ok, || {
                format!(
                    "template={} number={number}: original eax={:#x} {}; rust {mine:?} {}",
                    bytes_str(&template[..template.len() - 1]),
                    ro.eax,
                    bytes_str(&theirs[..0x30]),
                    bytes_str(&ours[..0x30])
                )
            });
        }
    }
    unsafe { copy_nonoverlapping(saved.as_ptr(), buffer(), BUFFER_LEN) };
    t.finish()
}
