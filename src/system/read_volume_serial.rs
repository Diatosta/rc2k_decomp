//! `system::read_volume_serial` (0x403131): the current drive's serial number and volume
//! name (`GetVolumeInformationA` with no root), into the first 16 bytes of `g_pngInfo`
//! (0x93E8B0, a scratch place here): +0 the serial, +4 the name in 12 bytes, +0xF made 0.
//!
//! In:  esi = anything
//! Out: the original pushes esi and ends with `pop eax`, so eax = the entry's esi (the
//!      serial meant for eax is lost: `notes/bugs.md`), esi = 0x93E8B0. Worked: ebx, ecx,
//!      edx = the dwords at +4, +8, +0xC; carry clear, zero set if the serial is 0
//!      (`test eax, eax`). Failed: ebx, ecx, edx = 0, carry and zero set. edi, ebp kept.
//! One caller, `start` (0x40100B), before `math::stir_random`, which stirs in eax. In the
//! game this never gets a call (the hook DLL is loaded later); the standalone program
//! needs it. The Win32 call is made from Rust (it is the same call in both programs).
//! Shadow mode: both (the call only reads), compared. Self-test: the original and the
//! Rust version, registers, both flags and the 16 bytes compared.

use crate::framework::{
    CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, game_addr, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationA;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::read_volume_serial",
    rva(0x403131),
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

// ---- game data ----

const VOLUME_RVA: usize = rva(0x93E8B0); // g_pngInfo: its first 16 bytes
const VOLUME_LEN: usize = 16;

// ---- Rust version ----

/// The registers and flags the original leaves (see the header).
pub unsafe fn read_volume_serial(entry: &Regs) -> Regs {
    let at = game_addr(VOLUME_RVA) as *mut u8;
    unsafe {
        let worked = GetVolumeInformationA(
            std::ptr::null(),
            at.add(4),
            12,
            at as *mut u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        ) != 0;
        let mut r = Regs {
            eax: entry.esi,
            esi: at as u32,
            ..*entry
        };
        if worked {
            at.add(0xF).write(0);
            let d = |o: usize| (at.add(o) as *const u32).read_unaligned();
            (r.ebx, r.ecx, r.edx) = (d(4), d(8), d(0xC));
            r.set_carry(false);
            r.set_zero(d(0) == 0);
        } else {
            (r.ebx, r.ecx, r.edx) = (0, 0, 0);
            r.set_carry(true);
            r.set_zero(true);
        }
        r
    }
}

// ---- hook ----

fn same(a: &Regs, b: &Regs) -> bool {
    let flags = CARRY_FLAG | ZERO_FLAG;
    [a.eax, a.ebx, a.ecx, a.edx, a.esi, a.edi, a.ebp]
        == [b.eax, b.ebx, b.ecx, b.edx, b.esi, b.edi, b.ebp]
        && a.eflags & flags == b.eflags & flags
}

fn regs_text(r: &Regs) -> String {
    format!(
        "eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} carry={} zero={}",
        r.eax,
        r.ebx,
        r.ecx,
        r.edx,
        r.esi,
        r.carry(),
        r.eflags & ZERO_FLAG != 0
    )
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = unsafe { read_volume_serial(r) };
    if !shadow() {
        *r = Regs { esp: r.esp, ..mine };
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if !same(&mine, &o) {
        HOOK.mismatch(|| format!("original {}; rust {}", regs_text(&o), regs_text(&mine)));
    }
    *r = Regs { esp: r.esp, ..o };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 1] = [(game_addr(VOLUME_RVA) as u32, VOLUME_LEN)];
    let saved = capture(&regions);
    for esi in [0x2222_2222u32, 0] {
        let before = Regs {
            esi,
            ..Regs::SENTINEL
        };
        restore(&regions, &[0xAA; VOLUME_LEN]);
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = capture(&regions);
        restore(&regions, &[0xAA; VOLUME_LEN]);
        let mine = unsafe { read_volume_serial(&before) };
        let state = capture(&regions);
        t.check(same(&ro, &mine) && theirs == state, || {
            format!(
                "esi={esi:#x}: original {} {theirs:02x?}; rust {} {state:02x?}",
                regs_text(&ro),
                regs_text(&mine)
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
