//! `strings::same_text` (0x401EDE): compares two texts without regard to case.
//!
//! In:  eax, ebx = the texts (NUL-terminated)
//! Out: zero set if they are the same; else zero clear, and carry set if the first's
//!      differing character is the lower; every register preserved
//! Character by character, each through `g_lowerCase`. Five callers.

use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva, shadow};
use crate::settings::find_key::LOWER_RVA;
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::same_text", rva(0x401EDE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// How the first text's first differing character stands to the second's.
pub unsafe fn compare_texts(a: *const u8, b: *const u8) -> Ordering {
    unsafe {
        let lower = |c: u8| game_read::<u8>(LOWER_RVA + c as usize);
        let (mut a, mut b) = (a, b);
        loop {
            let (x, y) = (lower(a.read()), lower(b.read()));
            if x != y {
                return x.cmp(&y);
            }
            if x == 0 {
                return Ordering::Equal;
            }
            (a, b) = (a.add(1), b.add(1));
        }
    }
}

pub unsafe fn same_text(a: *const u8, b: *const u8) -> bool {
    unsafe { compare_texts(a, b) == Ordering::Equal }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = unsafe { compare_texts(r.eax as *const u8, r.ebx as *const u8) };
    if !shadow() {
        r.set_zero(mine == Ordering::Equal);
        r.set_carry(mine == Ordering::Less);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = (o.eflags & 0x40 != 0, o.carry());
    if theirs != (mine == Ordering::Equal, mine == Ordering::Less) || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "{:?} and {:?}: original zero={} carry={}; rust {mine:?}",
                unsafe { std::ffi::CStr::from_ptr(r.eax as *const _) },
                unsafe { std::ffi::CStr::from_ptr(r.ebx as *const _) },
                theirs.0,
                theirs.1
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let texts: [&[u8]; 9] = [
        b"\0",
        b"a\0",
        b"A\0",
        b"b\0",
        b"C:\\Games\\RC2k\\\0",
        b"c:\\games\\rc2k\\\0",
        b"c:\\games\\rc2k\0",
        b"\xC9t\xE9\0",
        b"\xE9T\xC9\0",
    ];
    for a in texts {
        for b in texts {
            let before = Regs {
                eax: a.as_ptr() as u32,
                ebx: b.as_ptr() as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = unsafe { compare_texts(a.as_ptr(), b.as_ptr()) };
            let theirs = (ro.eflags & 0x40 != 0, ro.carry());
            let preserved = ro.preserved_except(&before, 0);
            t.check(
                theirs == (mine == Ordering::Equal, mine == Ordering::Less) && preserved,
                || {
                    format!(
                        "{:?} and {:?}: original zero={} carry={} preserved={preserved}; rust {mine:?}",
                        String::from_utf8_lossy(a),
                        String::from_utf8_lossy(b),
                        theirs.0,
                        theirs.1
                    )
                },
            );
        }
    }
    t.finish()
}
