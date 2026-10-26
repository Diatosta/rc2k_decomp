//! `marks::clip_quad` (0x4B8A0B): a quad (four vertices of 0x20 bytes at ebp: +0 x, +4 y,
//! +8 .. +0x14 kept, +0x18 / +0x1C texture) clipped against one screen edge by the
//! settings' +8: 4 x at least `g_marksScreenMiddleX`, 3 x at most `g_marksScreenMiddleXLess`,
//! 2 y at least `g_marksScreenMiddleY`, 1 y at most `g_marksScreenMiddleYLess` (each test a
//! signed compare of the floats' bits, as the original's `sub; jl`). All in: CF clear,
//! nothing changed. All out: CF set. Else the run of inside vertices (from the first after
//! an outside one to the last) and the two crossings (the far edge's, then the near
//! edge's: the edge's coordinate the line, the other and the texture interpolated on the
//! x87, +8 .. +0x14 the outside vertex's) are built in `g_marksClipVertices` (`TLVertex`s) and copied over
//! the quad; `g_polygonVertexCount` and `dword_93CC94` their number; CF clear. Other
//! settings: CF clear, nothing changed.
//!
//! In:  ebp the quad (`g_marksSettings`)
//! Out: CF as above; esi kept; after a clip eax the near outside vertex's +0x14, ebx the
//!      count, ecx 0, edx that vertex's offset, edi past the quad's new vertices; else
//!      eax the last test's difference (and edi the scan's place when all are out)
//!
//! Eight callers, each `jb` after it. Refuses null settings or quad. Self-test: quads of
//! the test's (every inside pattern, each mode), compared on the quad, the work area, the
//! counts, the registers and CF.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::clip_quad", rva(0x4B8A0B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WORK_RVA: usize = rva(0x7687F4); // g_marksClipVertices: TLVertex[6]
const WORK_LEN: usize = 6 * 0x20;
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const COUNT2_RVA: usize = rva(0x93CC94); // dword_93CC94
/// Per mode 1 .. 4: the line's variable, its coordinate's offset, whether inside is at or
/// below it.
const EDGES: [(usize, u32, bool); 4] = [
    (rva(0x7687F0), 4, true),  // 1: g_marksScreenMiddleYLess
    (rva(0x7687EC), 4, false), // 2: g_marksScreenMiddleY
    (rva(0x7687E8), 0, true),  // 3: g_marksScreenMiddleXLess
    (rva(0x7687E4), 0, false), // 4: g_marksScreenMiddleX
];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version (see the header): the registers it leaves, CF in `eflags`.
pub unsafe fn clip_quad(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        if settings == 0 || g.ebp == 0 {
            return Err(BadInput("null marks settings or quad").into());
        }
        let mode = get(settings + 8);
        if !(1..=4).contains(&mode) {
            g.set_carry(false);
            return Ok(g);
        }
        let (line_rva, coord, below) = EDGES[mode as usize - 1];
        let line: u32 = game_read(line_rva);
        let q = g.ebp;
        // The test of vertex k: (inside, the difference eax keeps).
        let test = |k: u32| {
            let v = get(q + 0x20 * k + coord);
            if below {
                let d = line.wrapping_sub(v);
                ((line as i32) >= (v as i32), d)
            } else {
                let d = v.wrapping_sub(line);
                ((v as i32) >= (line as i32), d)
            }
        };
        let mut eax = 0u32;
        let mut t = |k: u32| {
            let (i, d) = test(k);
            eax = d;
            i
        };
        let (esi, edi);
        if !t(0) {
            let first = (1..=3).find(|&k| t(k));
            let Some(first) = first else {
                g.eax = eax;
                g.edi = 0x60;
                g.set_carry(true);
                return Ok(g);
            };
            edi = 0x20 * first;
            esi = if t(3) {
                0x60
            } else if t(2) {
                0x40
            } else {
                0x20
            };
        } else if !t(1) {
            esi = 0;
            edi = if t(2) {
                0x40
            } else if t(3) {
                0x60
            } else {
                0
            };
        } else if !t(2) {
            esi = 0x20;
            edi = if t(3) { 0x60 } else { 0 };
        } else if !t(3) {
            esi = 0x40;
            edi = 0;
        } else {
            g.eax = eax;
            g.set_carry(false);
            return Ok(g);
        }
        let work = game_addr(WORK_RVA) as u32;
        let mut ebx = 0u32;
        let mut k = edi;
        loop {
            std::ptr::copy((q + k) as *const u8, (work + ebx) as *mut u8, 0x20);
            ebx += 0x20;
            if k == esi {
                break;
            }
            k = (k + 0x20) & 0x7F;
        }
        let c = (esi + 0x20) & 0x7F;
        let d = (edi.wrapping_sub(0x20)) & 0x7F;
        let other = 4 - coord;
        for (inside, outside) in [(esi, c), (edi, d)] {
            let (s, o) = (q + inside, q + outside);
            let pairs = [
                [get(s + other), get(o + other)],
                [get(s + 0x18), get(o + 0x18)],
                [get(s + 0x1C), get(o + 0x1C)],
            ];
            let v = x87::clip_lerp3(line, get(o + coord), get(s + coord), pairs);
            let w = work + ebx;
            let put = |off: u32, x: u32| ((w + off) as *mut u32).write_unaligned(x);
            put(coord, line);
            put(other, v[0]);
            put(0x18, v[1]);
            put(0x1C, v[2]);
            for off in [8u32, 0xC, 0x10, 0x14] {
                put(off, get(o + off));
            }
            ebx += 0x20;
        }
        // The original's last load, before the copy back.
        g.eax = get(q + d + 0x14);
        std::ptr::copy(work as *const u8, q as *mut u8, ebx as usize);
        let count = ebx >> 5;
        game_write(COUNT_RVA, count);
        game_write(COUNT2_RVA, count);
        g.ebx = count;
        g.ecx = 0;
        g.edx = d;
        g.edi = q + ebx;
        g.set_carry(false);
    }
    Ok(g)
}

fn regions(quad: u32) -> Vec<Region> {
    let mut r = vec![
        (game_addr(WORK_RVA) as u32, WORK_LEN),
        at(COUNT_RVA),
        at(COUNT2_RVA),
    ];
    if quad != 0 {
        r.push((quad, 6 * 0x20));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { clip_quad(r) }?;
    let mut left = left_of(&g);
    left.carry = Some(g.carry());
    Ok(left)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.ebp), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return t.skip("no marks settings");
    }
    let mut quad = vec![0u32; 6 * 8];
    let qa = quad.as_mut_ptr() as u32;
    let mut vars = regions(qa);
    vars.push((settings + 8, 4));
    vars.push((game_addr(EDGES[3].0) as u32, 16));
    let saved = capture(&vars);
    let regions = regions(qa);
    for mode in 0..=5u32 {
        for pattern in 0..16u32 {
            check_state(
                &mut t,
                &format!("mode {mode}, inside pattern {pattern:04b}"),
                call_original,
                Regs {
                    ebp: qa,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    restore(&vars, &saved);
                    ((settings + 8) as *mut u32).write_unaligned(mode);
                    for (n, v) in [100.0f32, 300.0, 200.0, 250.0].iter().enumerate() {
                        game_write(EDGES[3].0 + 4 * n, v.to_bits());
                    }
                    let p = qa as *mut u32;
                    for k in 0..4u32 {
                        let inside = pattern & (1 << k) != 0;
                        // x: about 250 (inside for 4 above 250, for 3 below 200), y: about 100 / 300.
                        let (x, y) = match mode {
                            4 => (
                                if inside {
                                    260.0 + k as f32 * 7.0
                                } else {
                                    240.0 - k as f32
                                },
                                50.0 + k as f32,
                            ),
                            3 => (
                                if inside {
                                    190.0 - k as f32 * 7.0
                                } else {
                                    210.0 + k as f32
                                },
                                50.0 + k as f32,
                            ),
                            2 => (
                                20.0 + k as f32,
                                if inside {
                                    110.0 + k as f32 * 3.0
                                } else {
                                    90.0 - k as f32
                                },
                            ),
                            _ => (
                                20.0 + k as f32,
                                if inside {
                                    290.0 - k as f32 * 3.0
                                } else {
                                    310.0 + k as f32
                                },
                            ),
                        };
                        let base = (8 * k) as usize;
                        *p.add(base) = x.to_bits();
                        *p.add(base + 1) = y.to_bits();
                        for n in 2..6 {
                            *p.add(base + n) = 0x1111_1111 * (k + 1) + n as u32;
                        }
                        *p.add(base + 6) = (k as f32 * 0.25).to_bits();
                        *p.add(base + 7) = (1.0 - k as f32 * 0.2).to_bits();
                    }
                    for n in 32..48 {
                        *p.add(n) = 0xAAAA_AAAA;
                    }
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&vars, &saved);
    drop(quad);
    t.finish()
}
