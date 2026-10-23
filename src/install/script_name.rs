//! `install::script_name` (0x41370C): a name copied from the script (`[ebx]` on) to
//! `byte_95EAF8` (`[esi]`), byte by byte, until a byte below a space (a control byte or the
//! end); a NUL written at the end.
//!
//! In:  ebx the script cursor, esi where to write
//! Out: al the stop byte, ebx at it (not past), esi where the NUL went; the flags of
//!      `cmp al, 20h` (no caller reads them)
//! Caller `install::run_script` (three times). A loop back to its first byte: private copy,
//! manual patch. Shadow mode compares. Self-test: scripts of the test's against the
//! original.

use crate::bridge::Raw;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, wb};
use crate::race::viewer::{dispatch_regs, regs_entry};
use rc2k_core::install::script;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::script_name", rva(0x41370C), thunk, &ORIGINAL)
    .private_copy(0x10)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn script_name(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::install::script::script_name` (read whole, then written; a
    // name with no end is refused before anything is written).
    let script = unsafe { Raw::new(r.ebx as *const u8) };
    let (name, stop) = script::script_name(&script)?;
    for &c in &name {
        wb(r.esi, c);
        r.esi = r.esi.wrapping_add(1);
    }
    wb(r.esi, 0);
    r.ebx = r.ebx.wrapping_add(name.len() as u32);
    r.eax = r.eax & !0xFF | stop as u32;
    r.set_carry(true);
    r.set_zero(false);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    // the destination is `byte_95EAF8`, up to the source's length
    let regions = [(r.esi, script::NAME_LIMIT)];
    dispatch_regs(&HOOK, r, call_original, &regions, CF | ZF, &script_name);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &script_name)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut dst = vec![0xAAu8; 0x40];
    let dptr = dst.as_mut_ptr() as u32;
    for src in [
        &b"files\x01"[..],
        b"\x01",
        b"a\tb",
        b"d\\e\\f\x00",
        b"x\x1f",
    ] {
        let mut s = src.to_vec();
        let before = Regs {
            ebx: s.as_mut_ptr() as u32,
            esi: dptr,
            ..Regs::SENTINEL
        };
        let regions = [(dptr, 0x40usize)];
        crate::race::viewer::check_original(
            &mut t,
            &format!("src {}", crate::strings::utils::bytes_str(src)),
            call_original,
            before,
            &regions,
            CF | ZF,
            &|| {
                for k in 0..0x40u32 {
                    wb(dptr + k, 0xAA);
                }
            },
            &script_name,
        );
    }
    drop(dst);
    t.finish()
}
