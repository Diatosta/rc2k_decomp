//! `hud::disc_vertex` (0x4B6D10): vertex edi of the HUD's disc from its ring entry ecx:
//! the texture place (`flt_75C210`[ecx], -`flt_75C254`[ecx], times `dword_75C050` times
//! `dword_75C04C`, plus `flt_75C01C` and `dword_75C054` / `dword_75C058`, over
//! `flt_75C020`); the place (`flt_75C188`[ecx], -`flt_75C1CC`[ecx], times `dword_75C050`,
//! plus `dword_75C044` / `dword_75C048`); colour white under the quad record's alpha,
//! specular 0.
//!
//! In:  ecx the ring entry (0 the centre .. 0x10), edi the vertex
//! Out: eax the colour; the rest kept
//!
//! Caller: `hud::draw_disc` (three a triangle). Refuses an entry past 0x10, before
//! anything. Self-test: the original and the Rust version on a vertex of the test's,
//! every entry.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::hud::add_quad::QUAD_RVA;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::disc_vertex", rva(0x4B6D10), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const ENTRIES: u32 = 0x11;

unsafe fn place(b: u32, k: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + {k} + 0x35C210]",
            "fmul dword ptr [{b} + 0x35C050]",
            "fmul dword ptr [{b} + 0x35C04C]",
            "fadd dword ptr [{b} + 0x35C01C]",
            "fadd dword ptr [{b} + 0x35C054]",
            "fdiv dword ptr [{b} + 0x35C020]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{b} + {k} + 0x35C254]",
            "fchs",
            "fmul dword ptr [{b} + 0x35C050]",
            "fmul dword ptr [{b} + 0x35C04C]",
            "fadd dword ptr [{b} + 0x35C01C]",
            "fadd dword ptr [{b} + 0x35C058]",
            "fdiv dword ptr [{b} + 0x35C020]",
            "fstp dword ptr [{e} + 0x1C]",
            "fld dword ptr [{b} + {k} + 0x35C188]",
            "fmul dword ptr [{b} + 0x35C050]",
            "fadd dword ptr [{b} + 0x35C044]",
            "fstp dword ptr [{e}]",
            "fld dword ptr [{b} + {k} + 0x35C1CC]",
            "fchs",
            "fmul dword ptr [{b} + 0x35C050]",
            "fadd dword ptr [{b} + 0x35C048]",
            "fstp dword ptr [{e} + 4]",
            b = in(reg) b,
            k = in(reg) k,
            e = in(reg) e,
            out("st(0)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn disc_vertex(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.ecx >= ENTRIES {
        return Err(BadInput("a disc entry past the ring").into());
    }
    unsafe {
        place(game_addr(0) as u32, g.ecx * 4, g.edi);
        g.eax = (game_read::<u32>(QUAD_RVA) << 24) | 0xFF_FFFF;
        ((g.edi + 0x10) as *mut u32).write_unaligned(g.eax);
        ((g.edi + 0x14) as *mut u32).write_unaligned(0);
    }
    Ok(g)
}

fn regions_at(edi: u32) -> Vec<Region> {
    vec![(edi, 0x20)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { disc_vertex(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions_at(r.edi);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The disc's tables by lot (for this test and `draw_disc`'s); what they held is the
/// caller's to put back (`tables()`).
pub unsafe fn test_tables(case: u32) {
    unsafe {
        let f = game_addr(rva(0x75C188)) as *mut f32;
        for n in 0..(4 * ENTRIES as usize) {
            *f.add(n) = ((n as f32) * 0.3 + case as f32).sin();
        }
        let v = |r: usize, x: f32| crate::framework::game_write(rva(r), x.to_bits());
        v(0x75C050, 40.0 + case as f32);
        v(0x75C04C, 0.9);
        v(0x75C01C, 128.0);
        v(0x75C054, 3.0);
        v(0x75C058, -2.0);
        v(0x75C020, 256.0);
        v(0x75C044, 320.0);
        v(0x75C048, 240.0);
        crate::framework::game_write(QUAD_RVA, 0xC0 + case);
    }
}

/// What `test_tables` writes.
pub fn tables() -> Vec<Region> {
    let mut r = vec![(game_addr(rva(0x75C188)) as u32, 0x44 * 4)];
    r.push((game_addr(rva(0x75C01C)) as u32, 8));
    r.push((game_addr(rva(0x75C044)) as u32, 0x18));
    r.push((game_addr(QUAD_RVA) as u32, 4));
    r
}

fn self_test() -> (usize, usize) {
    use crate::service::harness::{capture, restore};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vertex = [0u32; 8];
    let va = vertex.as_mut_ptr() as u32;
    let regions = regions_at(va);
    let vars = tables();
    let saved = capture(&vars);
    for n in 0..ENTRIES {
        check_state(
            &mut t,
            &format!("entry {n}"),
            call_original,
            Regs {
                ecx: n,
                edi: va,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&vars, &saved);
                test_tables(n % 3);
                std::ptr::write_bytes(va as *mut u8, 0xAA, 0x20);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
