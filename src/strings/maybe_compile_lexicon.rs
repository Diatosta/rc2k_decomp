//! `strings::maybe_compile_lexicon` (0x403555): the string table made again from the language
//! file (`strings::compile_lexicon`) when `dword_4E0758` (signed) is 1 or more and
//! `byte_4F6B2A` (a key's state, it seems) is not 0: a developer's way to see a changed
//! language file at once (a reading).
//!
//! In:  nothing
//! Out: every register kept; the flags of the last test (no caller reads them)
//!
//! Callers: the network's list screens (`net::settings_loop`, `net::choose_connection`,
//! `net::choose_session`, `net::lobby_loop`), for a key they do not know. Its call through
//! `part` (`net::lobby_calls`: the Rust version). Shadow mode runs the original alone.
//! Self-test: a copy whose two variables are the test's (the key's state is written by the
//! input thread) and whose call goes to a stand-in, against the Rust version, on both
//! variables either side of their tests.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{COMPILE_LEXICON, Part, a, dispatch_lobby, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::maybe_compile_lexicon",
    rva(0x403555),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEVEL: usize = 0x4E0758; // dword_4E0758
const KEY: usize = 0x4F6B2A; // byte_4F6B2A

/// The Rust version on the variables at `level` (a dword) and `key` (a byte).
pub fn maybe_compile_lexicon_with(
    entry: &Regs,
    level: u32,
    key: u32,
    part: Part,
) -> Result<Regs, Refused> {
    let mut r = *entry;
    let (level, key) = unsafe {
        (
            (level as *const u32).read_volatile(),
            (key as *const u8).read_volatile(),
        )
    };
    if level as i32 >= 1 && key != 0 {
        part(COMPILE_LEXICON, &mut r)?;
    }
    Ok(r)
}

/// The Rust version: the registers it leaves.
pub fn maybe_compile_lexicon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    maybe_compile_lexicon_with(entry, a(LEVEL), a(KEY), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, maybe_compile_lexicon);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = vec![0u32; 2];
    let at = vars.as_mut_ptr() as u32;
    // cmp dword_4E0758, 1
    let copy = match crate::net::lobby::copy_vars(
        0x403555,
        0x19,
        &[0x83, 0x3D, 0x58, 0x07, 0x4E, 0x00, 0x01],
        &[0x12],
        &[COMPILE_LEXICON],
        &[(LEVEL, 4, at), (KEY, 1, at + 4)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [(at, 8)];
    for level in [0u32, 1, 2, 0xFFFF_FFFF] {
        for key in [0u32, 1, 0x80] {
            run_both(
                &mut t,
                &format!("level {level:#x}, key {key:#x}"),
                copy.addr(),
                Regs::SENTINEL,
                &regions,
                0,
                &[],
                &|| unsafe { (at as *mut [u32; 2]).write_volatile([level, key]) },
                &|r, part| maybe_compile_lexicon_with(r, at, at + 4, part),
            );
        }
    }
    let _ = &mut vars;
    t.finish()
}
