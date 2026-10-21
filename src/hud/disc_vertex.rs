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
//!
//! Converted 2026-10-21: the hook's Rust version is `disc_vertex_plain`, through
//! `rc2k_core::hud::disc::vertex` (plain floats; the x87 version stays for the Rust callers
//! in unconverted code). Shadow mode and the self-test compare within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::hud::add_quad::QUAD_RVA;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::Region;
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

pub use rc2k_core::hud::disc::ENTRIES;

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

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::hud::disc::vertex`
pub unsafe fn disc_vertex_plain(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::hud::disc::{Disc, RingEntry, vertex, white_under};
    let mut g = *entry;
    if g.ecx >= ENTRIES {
        return Err(BadInput("a disc entry past the ring").into());
    }
    unsafe {
        let f = |a: usize| f32::from_bits(game_read(rva(a)));
        let ring = |table: usize| f32::from_bits(game_read(rva(table) + 4 * g.ecx as usize));
        let e = RingEntry {
            x: ring(0x75C188),
            y: ring(0x75C1CC),
            u: ring(0x75C210),
            v: ring(0x75C254),
        };
        let d = Disc {
            radius: f(0x75C050),
            texture_scale: f(0x75C04C),
            texture_middle: f(0x75C01C),
            texture_shift: [f(0x75C054), f(0x75C058)],
            texture_size: f(0x75C020),
            centre: [f(0x75C044), f(0x75C048)],
        };
        let v = vertex(e, &d);
        let put = |o: u32, x: u32| ((g.edi + o) as *mut u32).write_unaligned(x);
        put(0x18, v.texture[0].to_bits());
        put(0x1C, v.texture[1].to_bits());
        put(0, v.place[0].to_bits());
        put(4, v.place[1].to_bits());
        g.eax = white_under(game_read(QUAD_RVA));
        put(0x10, g.eax);
        put(0x14, 0);
    }
    Ok(g)
}

fn regions_at(edi: u32) -> Vec<Region> {
    vec![(edi, 0x20)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { disc_vertex_plain(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions_at(r.edi);
    let places = float_places(&regions);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
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
    let tolerant_places = float_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for n in 0..ENTRIES {
            check_state_floats(
                &mut t,
                "hud::disc_vertex",
                &format!("control word {word:#x}, entry {n}"),
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
                held,
                &rust,
            );
        }
    });
    restore(&vars, &saved);
    t.finish()
}
