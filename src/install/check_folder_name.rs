//! `install::check_folder_name` (0x413A6C): whether a folder path (from its third byte on,
//! past a drive letter and colon) holds a character that is not allowed in a folder name
//! (`unk_51DE18`: `"`, `+`, `*`, `,`, `;`, `=`, `/`, `:`, `<`, `>`, `?`, `\`, `{`, `}`,
//! `[`, `]`, `|`, 0x7F) or a double backslash.
//!
//! In:  edx the path
//! Out: carry set if a bad character or `\\` is found, clear otherwise; every register kept
//! Caller `install::install_files`. Private copy (0x2B bytes): the loop's `jnz` at +0x12 goes
//! back to +4, inside the patched bytes. Shadow mode compares. Self-test: paths of the test's
//! against the original.

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::CF;
use crate::race::viewer::{check_original, dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "install::check_folder_name",
    rva(0x413A6C),
    thunk,
    &ORIGINAL,
)
.private_copy(0x2B)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn check_folder_name(entry: &Regs) -> Regs {
    let mut r = *entry;
    // Converted: `rc2k_core::install::paths::bad_folder_name` (the characters, RAL.EXE's
    // `unk_51DE18`, as its constant).
    let path = unsafe { Raw::new(r.edx as *const u8) };
    r.set_carry(read_through(rc2k_core::install::paths::bad_folder_name(
        &path,
    )));
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(check_folder_name(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &[], CF, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for path in [
        &b"c:\\games\\rally"[..],
        b"c:\\games",
        b"c:\\a*b",
        b"c:\\a\\\\b",
        b"d:good\\path",
        b"c:",
        b"c:\\",
        b"c:\\a?b",
        b"c:\\a|b",
        b"ab",
    ] {
        let mut p = path.to_vec();
        p.push(0);
        let before = Regs {
            edx: p.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        };
        check_original(
            &mut t,
            &format!("path {}", crate::strings::utils::bytes_str(path)),
            call_original,
            before,
            &[],
            CF,
            &|| {},
            &rust,
        );
    }
    t.finish()
}
