//! `files::drive_free_space_kb` (0x40317D): a drive's free space in KB.
//!
//! In:  eax = the drive's letter in its low byte (0: the current drive)
//! Out: eax = the free KB, or -1 when `GetDiskFreeSpaceA` failed (carry set) or the KB do
//!      not fit 32 bits (carry clear, as for an answer); else carry clear and zero by eax.
//!      After an answer ebx = the bytes a sector; ecx, edx, esi kept
//! The root asked about is `x:\` with the letter lowered (`files::drive_root`), and the
//! four figures go to the first 16 bytes of `g_pngInfo` (total clusters, free clusters,
//! bytes a sector, sectors a cluster). The KB are (sectors a cluster x bytes a sector, its
//! low 32 bits) x free clusters, shifted down 10 bits; past 2^42 bytes it gives -1.
//! Callers: the installer (`install::install_files`, `install::list_drives`) and
//! `files::free_space_kb`. The Win32 call through `api` (`installer::game_api` in the
//! game). Private copy (0x5B bytes, two calls relocated), manual patch: a call starts at
//! +3. Shadow mode runs the original alone (the space can change between two asks).
//! Self-test: the original and the Rust version on every letter, `0`, and two that are no
//! drive's, the registers, flags and the 16 bytes compared (asked again up to three times
//! where the free space moved between the two).

use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva, shadow};
use crate::install::installer::{Api, GET_DISK_FREE, SCRATCH, a, game_api, rd};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "files::drive_free_space_kb",
    rva(0x40317D),
    thunk,
    &ORIGINAL,
)
.private_copy(0x5B)
.relocate_calls(&[0x03, 0x23])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The registers and flags it leaves.
pub fn free_space_regs(entry: &Regs, api: Api) -> Regs {
    let mut r = *entry;
    let root = unsafe { crate::files::drive_type::drive_root(r.eax as u8) };
    let root_ptr = root.as_ref().map_or(0, |b| b.as_ptr() as u32);
    let s = a(SCRATCH);
    let answer = api(GET_DISK_FREE, &[root_ptr, s + 0xC, s + 8, s + 4, s]);
    if answer < 1 {
        r.eax = u32::MAX;
        r.set_carry(true);
        r.set_zero(false);
        return r;
    }
    r.ebx = rd(s + 8);
    // Converted: `rc2k_core::files::log::free_kb`.
    r.eax = rc2k_core::files::log::free_kb(rd(s + 0xC), rd(s + 8), rd(s + 4));
    r.set_carry(false);
    r.set_zero(r.eax == 0);
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    *r = Regs {
        esp: r.esp,
        ..free_space_regs(r, &mut game_api)
    };
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scratch = (a(SCRATCH), 16usize);
    let saved = crate::service::harness::capture(&[scratch]);
    let flags = CARRY_FLAG | ZERO_FLAG;
    for letter in (b'a'..=b'z').chain(*b"\0C1\xC9") {
        let before = Regs {
            eax: 0xAABB_CC00 | letter as u32,
            ..Regs::SENTINEL
        };
        let mut result = (false, String::new());
        for _ in 0..3 {
            let mut o = before;
            unsafe { call_original(&mut o) };
            let theirs = crate::service::harness::capture(&[scratch]);
            let m = free_space_regs(&before, &mut game_api);
            let mine = crate::service::harness::capture(&[scratch]);
            let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
            let ok = regs(&o) == regs(&m) && (o.eflags ^ m.eflags) & flags == 0 && theirs == mine;
            result = (
                ok,
                format!(
                    "letter {letter:#x}: original eax={:#x} ebx={:#x} flags={:#x} {theirs:x?}; \
                     rust eax={:#x} ebx={:#x} flags={:#x} {mine:x?}",
                    o.eax,
                    o.ebx,
                    o.eflags & flags,
                    m.eax,
                    m.ebx,
                    m.eflags & flags
                ),
            );
            if ok {
                break;
            }
        }
        t.check(result.0, || result.1);
    }
    crate::service::harness::restore(&[scratch], &saved);
    t.finish()
}
