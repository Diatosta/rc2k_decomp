//! `hud::add_polygon` (0x4B7413): `g_polygonVertices` (`g_polygonVertexCount` of them)
//! scaled from the virtual screen to the real one (x times `g_screenWidth` over
//! `flt_75C02C`, y times `g_screenHeight` over `flt_75C030`) and added to the batch: when
//! the batch holds more than 0x64 vertices or the texture asked for (`dword_75C034`) is not
//! the batch's (`dword_75C038`), `hud::flush_batch`, the texture made the batch's and the
//! shown one (`dword_6D14FE`), `hud::set_batch_texture`; the vertices copied after the
//! batch's; the indices of a triangle (and with other than three vertices a second one
//! making a quad).
//!
//! In:  nothing (`g_polygonVertices`, `g_polygonVertexCount`)
//! Out: eax the count, ecx 0, esi past the vertices, edi the triangle's indices; ebx and
//!      the rest as the flush leaves them
//!
//! Callers: `sub_4B5A86`, `hud::add_triangle`, `sub_4B7058`, `hud::add_quad`. The flush
//! and the texture through `part` (1, 2), in the game their Rust versions. Refuses a count
//! of 0 (the original's loop would run on) or past 0x10, and a batch it would write past,
//! before anything. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose two calls go to stand-ins, against the Rust version, batch counts and textures by
//! lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::batch::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::add_polygon", rva(0x4B7413), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const VIRTUAL_RVA: [usize; 2] = [rva(0x75C02C), rva(0x75C030)]; // flt_75C02C, flt_75C030
const FULL: u32 = 0x64;
const MOST: u32 = 0x10;

/// A call by place: 1 `flush_batch`, 2 `set_batch_texture`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `fld v; fimul i; fdiv k; fstp`.
fn times_int_over(v: u32, i: u32, k: u32) -> u32 {
    let (a, mut out) = ([v, i, k], 0u32);
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{a}]",
            "fimul dword ptr [{a} + 4]",
            "fdiv dword ptr [{a} + 8]",
            "fstp dword ptr [{o}]",
            a = in(reg) a.as_ptr(),
            o = in(reg) &raw mut out,
            out("st(0)") _,
            options(nostack),
        )
    };
    out
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn add_polygon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let count: u32 = game_read(POLYGON_COUNT_RVA);
        if count == 0 || count > MOST {
            return Err(BadInput("a HUD polygon of 0 or too many vertices").into());
        }
        let texture: u32 = game_read(TEXTURE_RVA);
        let flushes = game_read::<u32>(VERTEX_COUNT_RVA) > FULL
            || texture != game_read::<u32>(BATCH_TEXTURE_RVA);
        let empties = flushes && game_read::<u32>(INDEX_COUNT_RVA) != 0;
        let [vc, ic] = if empties {
            [0, 0]
        } else {
            [
                game_read::<u32>(VERTEX_COUNT_RVA),
                game_read(INDEX_COUNT_RVA),
            ]
        };
        if vc + count > VERTEX_ROOM || ic + 6 > INDEX_ROOM {
            return Err(BadInput("a HUD polygon past the batch").into());
        }
        let [w, h, kx, ky] = [
            game_read(WIDTH_RVA),
            game_read(HEIGHT_RVA),
            game_read(VIRTUAL_RVA[0]),
            game_read(VIRTUAL_RVA[1]),
        ];
        for n in 0..count as usize {
            let v = POLYGON_RVA + 0x20 * n;
            game_write(v, times_int_over(game_read(v), w, kx));
            game_write(v + 4, times_int_over(game_read(v + 4), h, ky));
        }
        g.ecx = count;
        if flushes {
            // Not full: the flush is reached from the texture's compare (eax the texture).
            if game_read::<u32>(VERTEX_COUNT_RVA) <= FULL {
                g.eax = texture;
            }
            part(1, &mut g)?;
            game_write(SHOWN_TEXTURE_RVA, texture);
            game_write(BATCH_TEXTURE_RVA, texture);
            g.eax = texture;
            part(2, &mut g)?;
        }
        let vc: u32 = game_read(VERTEX_COUNT_RVA);
        std::ptr::copy(
            game_addr(POLYGON_RVA) as *const u8,
            (game_addr(VERTICES_RVA) as u32 + vc * 0x20) as *mut u8,
            (count * 0x20) as usize,
        );
        g.esi = game_addr(POLYGON_RVA) as u32 + count * 0x20;
        g.ecx = 0;
        let ic: u32 = game_read(INDEX_COUNT_RVA);
        g.edi = game_addr(INDICES_RVA) as u32 + ic * 2;
        let put = |k: u32, v: u32| ((g.edi + 2 * k) as *mut u16).write_unaligned(v as u16);
        for k in 0..3 {
            put(k, vc + k);
        }
        game_write(INDEX_COUNT_RVA, ic + 3);
        if count != 3 {
            for (k, d) in [0, 2, 3].into_iter().enumerate() {
                put(3 + k as u32, vc + d);
            }
            game_write(INDEX_COUNT_RVA, ic + 6);
        }
        g.eax = count;
        game_write(VERTEX_COUNT_RVA, vc + count);
    }
    Ok(g)
}

/// In the game: the flush's and the texture's Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::{flush_batch, set_batch_texture};
    unsafe {
        *r = match place {
            1 => flush_batch::flush_batch(r, &mut flush_batch::game_part)?,
            _ => set_batch_texture::set_batch_texture(r, &mut set_batch_texture::game_part)?,
        };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { add_polygon(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The batch and the polygon by lot (for this test and the HUD's other tests).
pub unsafe fn test_batch(case: u32) {
    unsafe {
        std::ptr::write_bytes(
            game_addr(VERTICES_RVA) as *mut u8,
            0xAA,
            (VERTEX_ROOM * 0x20) as usize,
        );
        std::ptr::write_bytes(
            game_addr(INDICES_RVA) as *mut u8,
            0xAA,
            (INDEX_ROOM * 2) as usize,
        );
        game_write(
            VERTEX_COUNT_RVA,
            [0u32, 40, 0x64, 0x65, 0x90][case as usize % 5],
        );
        game_write(
            INDEX_COUNT_RVA,
            [0u32, 60, 0x96, 0, 0xD8][case as usize % 5],
        );
        game_write(TEXTURE_RVA, 3 + case % 2);
        game_write(BATCH_TEXTURE_RVA, 3u32);
        game_write(VIRTUAL_RVA[0], 640.0f32.to_bits());
        game_write(VIRTUAL_RVA[1], 480.0f32.to_bits());
        let p = game_addr(POLYGON_RVA) as *mut f32;
        for n in 0..32 {
            *p.add(n) = n as f32 * 7.5 + case as f32;
        }
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov ecx, 0
    let entry: &[u8] = &[0xB9, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x4B7413,
        0xDC,
        entry,
        &[0x4F, 0x63],
        &[(0x4B74EF, 1), (0x45A855, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([TEXTURE_RVA, VIRTUAL_RVA[0], VIRTUAL_RVA[1]].map(at));
    let saved = capture(&vars);
    for case in 0..10u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                test_batch(case);
                game_write(POLYGON_COUNT_RVA, 3 + case / 5);
            },
            &|r| unsafe { add_polygon(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
