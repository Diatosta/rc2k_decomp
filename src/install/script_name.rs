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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, rb, wb};
use crate::race::viewer::dispatch_regs;
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

pub const NAME_LIMIT: u32 = 0x1000;

pub fn script_name(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    for _ in 0..NAME_LIMIT {
        let al = rb(r.ebx);
        r.eax = r.eax & !0xFF | al as u32;
        r.set_carry(al < 0x20);
        r.set_zero(al == 0x20);
        if al < 0x20 {
            wb(r.esi, 0);
            return Ok(r);
        }
        wb(r.esi, al);
        r.ebx = r.ebx.wrapping_add(1);
        r.esi = r.esi.wrapping_add(1);
    }
    Err(crate::errors::BadInput("a script name with no control byte in 0x1000 bytes").into())
}

extern "C" fn dispatch(r: &mut Regs) {
    // the destination is `byte_95EAF8`, up to the source's length
    let regions = [(r.esi, NAME_LIMIT as usize)];
    dispatch_regs(&HOOK, r, call_original, &regions, CF | ZF, &script_name);
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
