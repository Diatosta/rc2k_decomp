//! `settings::find_key` (0x41157D): finds a key in a settings text.
//!
//! In:  eax = the key (lower case, NUL-terminated), esi = the text (NUL-terminated)
//! Out: eax = where the key's value starts, carry clear; or eax = 0, carry set;
//!      everything else preserved
//! A line is looked at from its first character above a space. The key must be there in
//! any case (the text is lowered through `byte_4F0BC8`) and be followed by `=` or by a
//! character that is a space or less. The value starts at the first character above a
//! space after that, an `=` between them being skipped once; a line that ends first has
//! its value at the line feed (so: empty), and a text that ends first has no such key.
//! A text that ends with the key itself makes the original compare on past both ends:
//! refused. Seven callers. Private copy (0x78 bytes): the line loop comes back into the
//! first bytes.

use crate::errors::BadInput;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("settings::find_key", rva(0x41157D), thunk, &ORIGINAL)
    .private_copy(0x78)
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// byte_4F0BC8: a character's lower case, 256 entries (A .. Z and the accented capitals).
pub const LOWER_RVA: usize = rva(0x4F0BC8);

// ---- Rust version ----

/// Where the key's value starts, or None.
pub unsafe fn find_key(text: *const u8, key: *const u8) -> Result<Option<*const u8>, BadInput> {
    if text.is_null() || key.is_null() {
        return Err(BadInput("no settings text or no key"));
    }
    unsafe {
        let lower = |c: u8| game_read::<u8>(LOWER_RVA + c as usize);
        let at = std::cell::Cell::new(text);
        let next = || {
            let c = at.get().read();
            at.set(at.get().add(1));
            c
        };
        'line: loop {
            // The line's first character above a space.
            let mut c = loop {
                match next() {
                    0 => return Ok(None),
                    c if c > b' ' => break c,
                    _ => {}
                }
            };
            // The key, character by character.
            let mut key_at = key;
            let key_left = loop {
                let k = key_at.read();
                key_at = key_at.add(1);
                c = lower(c);
                if c != k {
                    break k;
                }
                if k == 0 {
                    return Err(BadInput("the settings text ends with the key"));
                }
                c = next();
            };
            let mut after_equals = c == b'=';
            if key_left != 0 || (!after_equals && c > b' ') {
                // Not this line: on to the next.
                loop {
                    match c {
                        0 => return Ok(None),
                        b'\n' => continue 'line,
                        _ => c = next(),
                    }
                }
            }
            while !after_equals {
                match c {
                    0 => return Ok(None),
                    b'\n' => return Ok(Some(at.get().sub(1))),
                    b'=' => after_equals = true,
                    c if c > b' ' => return Ok(Some(at.get().sub(1))),
                    _ => c = next(),
                }
            }
            loop {
                match next() {
                    0 => return Ok(None),
                    c if c == b'\n' || c > b' ' => return Ok(Some(at.get().sub(1))),
                    _ => {}
                }
            }
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = match unsafe { find_key(r.esi as *const u8, r.eax as *const u8) } {
        Ok(mine) => mine.map_or(0, |value| value as u32),
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        r.eax = mine;
        r.set_carry(mine == 0);
        r.set_zero(mine == 1);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax != mine || o.carry() != (mine == 0) || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "key={:?} text={:#x}: original eax={:#x} carry={}; rust {mine:#x}",
                unsafe { std::ffi::CStr::from_ptr(r.eax as *const _) },
                r.esi,
                o.eax,
                o.carry()
            )
        });
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test ----

/// Texts for this test and for the tests of what is built on the finder.
pub const TEST_TEXTS: [&[u8]; 15] = [
    b"other=1\nkey\0",
    b"\0",
    b"key=1\0",
    b"key=\0",
    b"key\n\0",
    b"  KEY = 12\r\nother=3\0",
    b"; comment\r\nother=3\r\nkey 7\r\n\0",
    b"keys=1\nkex=2\nke=3\n key\t=\t-4\n\0",
    b"other=key\nxkey=5\nkey=6\0",
    b"key==9\n\0",
    b"key =\n\0",
    b"key = \nnext=1\0",
    b"key x\0",
    b"other\nKey\"quoted value\"\n\0",
    b"\r\n\r\n\tkEy\r\n\0",
];
pub const TEST_KEYS: [&[u8]; 4] = [b"key\0", b"other\0", b"ke\0", b"next\0"];

/// Whether the original can be given this pair (see the file header).
pub fn ends_with_key(text: &[u8], key: &[u8]) -> bool {
    let (text, key) = (&text[..text.len() - 1], &key[..key.len() - 1]);
    let line = text.rsplit(|&c| c == b'\n').next().unwrap_or(text);
    let line = &line[line.iter().take_while(|&&c| c <= b' ').count()..];
    line.eq_ignore_ascii_case(key)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for text in TEST_TEXTS {
        for key in TEST_KEYS {
            if ends_with_key(text, key) {
                let refused = unsafe { find_key(text.as_ptr(), key.as_ptr()) }.is_err();
                t.check(refused, || {
                    format!(
                        "key={:?} text={:?}: the rust version does not refuse a text that ends with the key",
                        String::from_utf8_lossy(key),
                        String::from_utf8_lossy(text)
                    )
                });
                continue;
            }
            let before = Regs {
                eax: key.as_ptr() as u32,
                esi: text.as_ptr() as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = unsafe { find_key(text.as_ptr(), key.as_ptr()) }
                .map(|found| found.map_or(0, |value| value as u32));
            let preserved = ro.preserved_except(&before, EAX);
            t.check(
                mine == Ok(ro.eax) && ro.carry() == (ro.eax == 0) && preserved,
                || {
                    format!(
                        "key={:?} text={:?}: original at {:?} carry={} preserved={preserved}; rust at {:?}",
                        String::from_utf8_lossy(key),
                        String::from_utf8_lossy(text),
                        (ro.eax != 0).then(|| ro.eax.wrapping_sub(text.as_ptr() as u32)),
                        ro.carry(),
                        mine.map(|at| (at != 0).then(|| at.wrapping_sub(text.as_ptr() as u32)))
                    )
                },
            );
        }
    }
    t.finish()
}
