//! `marks::draw_snow` (0x4C2E36): the snowflakes, `dword_8D21A4` of them (also into
//! `dword_8D21A0`; `dword_8DA428` the first, 0, then plus the count; `dword_8D219C` the
//! drawn). For each flake k: its place from `g_marksRandomsA` .. [k], y plus the fall
//! `dword_8D2194`, x and z moved by `g_marksCurve` at (the fall's bits 11 .. 19) plus
//! `g_marksRemaindersA` / `B`[k], plus `dword_8D2214` .., wrapped to 0x100000 and centred;
//! turned by the matrix at `dword_8D2220` .. plus `dword_8D2208` .. into `flt_8D22D4` ..
//! (the depth also whole, `dword_8D21F4`); unless that is at or below 0x10000: when
//! `dword_76523C` is set the colour `g_marksFadeColours`[the square of the light's dot
//! with the way from `flt_8DB430` .. times `flt_76518C`, 0 when the dot is below 0]; the
//! projected square (side `flt_8DB548` over the depth), depth times `flt_8D21F8`,
//! texture u `g_marksSpreads`[k] (+ `flt_8DB54C`), v `dword_8DB550` / `8DB554`;
//! `marks::clip_quad`, and unless all is outside `scene::draw_fan`, the drawn count plus 1
//! and `g_polygonVertexCount` 4.
//!
//! In:  nothing (`dword_8D21A4`)
//! Out: the registers as the last flake leaves them (esi the count)
//!
//! Caller: `marks::run_snow` (0x4C2A8F). The x87 blocks are the original's instructions
//! over the game's variables; the clip its Rust version (the copy's call goes to its
//! hook); the draw through `part` (1 `draw_fan`), in the game its Rust version. Refuses,
//! before anything (a dry pass, its scratch put back), more than 0x800 flakes, null marks
//! settings with a flake, a curve index past the 4095 entries and a fade index past the
//! 17. Shadow mode runs the original alone (it draws). Self-test: a copy whose draw goes
//! to a stand-in, against the Rust version, on tables of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::clip_quad::clip_quad;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::draw_snow", rva(0x4C2E36), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COUNT_RVA: usize = rva(0x8D21A4); // dword_8D21A4
const COUNT_COPY_RVA: usize = rva(0x8D21A0); // dword_8D21A0
const FIRST_RVA: usize = rva(0x8DA428); // dword_8DA428
const DRAWN_RVA: usize = rva(0x8D219C); // dword_8D219C
const MOST: i32 = 0x800;
const FALL_RVA: usize = rva(0x8D2194); // dword_8D2194
const RANDOMS: [usize; 3] = [rva(0x8C6000), rva(0x8C8000), rva(0x8CA000)];
const REMAINDERS: [usize; 2] = [rva(0x8CC000), rva(0x8CE000)];
const CURVE_RVA: usize = rva(0x8D2310); // g_marksCurve: 4095 entries
const CURVE: u32 = 4095;
const SHIFT_RVA: usize = rva(0x8D2214); // dword_8D2214 .. dword_8D221C
const DEPTH_RVA: usize = rva(0x8D21F4); // dword_8D21F4
const FADING_RVA: usize = rva(0x76523C); // dword_76523C
const FADE_INDEX_RVA: usize = rva(0x8DB460); // dword_8DB460
const DOT_RVA: usize = rva(0x8DB4C8); // flt_8DB4C8
const FADES_RVA: usize = rva(0x8DB464); // g_marksFadeColours: 17
const FADES: u32 = 17;
const V_RVA: [usize; 2] = [rva(0x8DB550), rva(0x8DB554)];
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
/// What the x87 blocks write: flt_8D22D4 .. flt_8D22DC, dword_8D21F4, flt_8DB448 ..
/// dword_8DB460, flt_8DB4C8.
const SCRATCH: [(usize, usize); 4] = [
    (rva(0x8D22D4), 12),
    (rva(0x8D21F4), 4),
    (rva(0x8DB448), 0x1C),
    (rva(0x8DB4C8), 4),
];

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

unsafe fn table(r: usize, k: u32) -> u32 {
    unsafe { game_read(r + 4 * k as usize) }
}

/// The original's 0x4C2EF0 .. 0x4C2F79: the place (ebx, eax, ecx as `fild`ed) turned.
unsafe fn turn(b: u32, v: &[u32; 3]) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{v}]",
            "fild dword ptr [{v} + 4]",
            "fild dword ptr [{v} + 8]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4D2220]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{b} + 0x4D222C]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{b} + 0x4D2238]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4D2208]",
            "fstp dword ptr [{b} + 0x4D22D4]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4D2224]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{b} + 0x4D2230]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{b} + 0x4D223C]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4D220C]",
            "fstp dword ptr [{b} + 0x4D22D8]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4D2228]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{b} + 0x4D2234]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4D2240]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4D2210]",
            "fist dword ptr [{b} + 0x4D21F4]",
            "fstp dword ptr [{b} + 0x4D22DC]",
            b = in(reg) b,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C2F9C .. 0x4C3038: the light's dot and its fade index.
unsafe fn fade(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4D22D4]",
            "fsub dword ptr [{b} + 0x4DB430]",
            "fst dword ptr [{b} + 0x4DB448]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x4D22D8]",
            "fsub dword ptr [{b} + 0x4DB434]",
            "fst dword ptr [{b} + 0x4DB44C]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x4D22DC]",
            "fsub dword ptr [{b} + 0x4DB438]",
            "fst dword ptr [{b} + 0x4DB450]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{b} + 0x36516C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4DB448]",
            "fst dword ptr [{b} + 0x4DB454]",
            "fmul dword ptr [{b} + 0x4DB43C]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4DB44C]",
            "fst dword ptr [{b} + 0x4DB458]",
            "fmul dword ptr [{b} + 0x4DB440]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{b} + 0x4DB450]",
            "fst dword ptr [{b} + 0x4DB45C]",
            "fmul dword ptr [{b} + 0x4DB444]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fst dword ptr [{b} + 0x4DB4C8]",
            ".byte 0xD8, 0xC8",
            "fmul dword ptr [{b} + 0x36518C]",
            "fistp dword ptr [{b} + 0x4DB460]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C305B .. 0x4C30EE: the projected square, its depth and u.
unsafe fn square(b: u32, i: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4D22DC]",
            "fdivr dword ptr [{b} + 0x224088]",
            "fst dword ptr [{b} + 0x4D21F4]",
            "fld dword ptr [{b} + 0x4D22D8]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4D22D4]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x5605A4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x5605A0]",
            "fld dword ptr [{b} + 0x4DB548]",
            "fmul dword ptr [{b} + 0x4D21F4]",
            "fld dword ptr [{b} + 0x5605A0]",
            ".byte 0xD8, 0xC1",
            "fst dword ptr [{b} + 0x5605C0]",
            "fst dword ptr [{b} + 0x5605E0]",
            ".byte 0xD8, 0xE1",
            "fstp dword ptr [{b} + 0x560600]",
            "fld dword ptr [{b} + 0x5605A4]",
            "fst dword ptr [{b} + 0x5605C4]",
            ".byte 0xDE, 0xC1",
            "fst dword ptr [{b} + 0x5605E4]",
            "fstp dword ptr [{b} + 0x560604]",
            "fld dword ptr [{b} + 0x4D22DC]",
            "fmul dword ptr [{b} + 0x4D21F8]",
            "fst dword ptr [{b} + 0x5605A8]",
            "fst dword ptr [{b} + 0x5605C8]",
            "fst dword ptr [{b} + 0x5605E8]",
            "fstp dword ptr [{b} + 0x560608]",
            "fld dword ptr [{b} + {i} + 0x4D0000]",
            "fst dword ptr [{b} + 0x5605B8]",
            "fst dword ptr [{b} + 0x560618]",
            "fadd dword ptr [{b} + 0x4DB54C]",
            "fst dword ptr [{b} + 0x5605D8]",
            "fstp dword ptr [{b} + 0x5605F8]",
            b = in(reg) b,
            i = in(reg) i,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// Flake k's whole-number place and the two curve indices: (eax, ebx, ecx, x index, z
/// index, the shift's edx, ebp, edi).
unsafe fn place(k: u32) -> ([u32; 3], [u32; 2], [u32; 3]) {
    unsafe {
        let mut eax = table(RANDOMS[0], k);
        let mut ebx = table(RANDOMS[1], k).wrapping_add(game_read::<u32>(FALL_RVA));
        let mut ecx = table(RANDOMS[2], k);
        let f = (ebx & 0xF_FFFF) >> 11;
        let ix = f.wrapping_add(table(REMAINDERS[0], k));
        let iz = f.wrapping_add(table(REMAINDERS[1], k));
        let shift: [u32; 3] = game_read(SHIFT_RVA);
        let curve = |i: u32| if i < CURVE { table(CURVE_RVA, i) } else { 0 };
        eax = eax.wrapping_add(curve(ix)).wrapping_add(shift[0]);
        ecx = ecx.wrapping_add(curve(iz)).wrapping_add(shift[2]);
        ebx = ebx.wrapping_add(shift[1]);
        let wrap = |v: u32| (v & 0xF_FFFF).wrapping_sub(0x7_FFFF);
        ([wrap(eax), wrap(ebx), wrap(ecx)], [ix, iz], shift)
    }
}

/// Why it refuses, if it does: a dry pass, its scratch put back.
unsafe fn refusal(b: u32) -> Option<&'static str> {
    unsafe {
        let n: i32 = game_read(COUNT_RVA);
        if n > MOST {
            return Some("more than 0x800 snowflakes");
        }
        if n > 0 && game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Some("no marks settings");
        }
        let scratch: Vec<Region> = SCRATCH
            .iter()
            .map(|&(r, l)| (game_addr(r) as u32, l))
            .collect();
        let saved = capture(&scratch);
        let mut why = None;
        for k in 0..n.max(0) as u32 {
            let ([eax, ebx, ecx], idx, _) = place(k);
            if idx.iter().any(|&i| i >= CURVE) {
                why = Some("a snow curve index past the table");
                break;
            }
            turn(b, &[ebx, eax, ecx]);
            if game_read::<i32>(DEPTH_RVA) > 0x1_0000 && game_read::<u32>(FADING_RVA) != 0 {
                fade(b);
                let i: u32 = game_read(FADE_INDEX_RVA);
                if game_read::<i32>(DOT_RVA) >= 0 && i >= FADES {
                    why = Some("a snow fade index past the table");
                    break;
                }
            }
        }
        restore(&scratch, &saved);
        why
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_snow(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    let b = game_addr(0) as u32;
    if let Some(why) = unsafe { refusal(b) } {
        return Err(BadInput(why).into());
    }
    unsafe {
        let n: i32 = game_read(COUNT_RVA);
        game_write(COUNT_COPY_RVA, n);
        game_write(FIRST_RVA, 0u32);
        g.eax = n as u32;
        game_write(DRAWN_RVA, 0u32);
        g.esi = 0;
        game_write(FIRST_RVA, n);
        for _ in 0..n.max(0) {
            let k = g.esi;
            let ([eax, ebx, ecx], _, shift) = place(k);
            [g.eax, g.ebx, g.ecx] = [eax, ebx, ecx];
            [g.edx, g.ebp, g.edi] = shift;
            turn(b, &[ebx, eax, ecx]);
            if game_read::<i32>(DEPTH_RVA) > 0x1_0000 {
                g.ebp = game_addr(VERTICES_RVA) as u32;
                if game_read::<u32>(FADING_RVA) != 0 {
                    fade(b);
                    g.eax = game_read(FADE_INDEX_RVA);
                    if game_read::<i32>(DOT_RVA) < 0 {
                        g.eax = 0;
                    }
                    g.eax = table(FADES_RVA, g.eax);
                    for n in 0..4 {
                        game_write(VERTICES_RVA + 0x20 * n + 0x10, g.eax);
                    }
                }
                square(b, 4 * k);
                g.eax = game_read(V_RVA[0]);
                g.ebx = game_read(V_RVA[1]);
                for n in 0..4 {
                    game_write(
                        VERTICES_RVA + 0x20 * n + 0x1C,
                        if n < 2 { g.eax } else { g.ebx },
                    );
                }
                g = clip_quad(&g)?;
                if !g.carry() {
                    part(1, &mut g)?;
                    g.esi = k;
                    game_write(DRAWN_RVA, game_read::<u32>(DRAWN_RVA).wrapping_add(1));
                    game_write(VERTEX_COUNT_RVA, 4u32);
                }
            }
            g.esi = k + 1;
        }
    }
    Ok(g)
}

/// In the game: `draw_fan`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_snow(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(crate::marks::setup::SETTINGS_RVA) } == 0 {
        return t.skip("no marks settings (the clip reads them)");
    }
    // push dword_8D21A4
    let entry: &[u8] = &[0xFF, 0x35, 0xA4, 0x21, 0x8D, 0x00];
    let copy = match copy(0x4C2E36, 0x306, entry, &[0x2D5, 0x2DD], &[(0x44EA65, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    const FLAKES: u32 = 24;
    let mut regions: Vec<Region> = vec![
        at(COUNT_COPY_RVA),
        at(FIRST_RVA),
        at(DRAWN_RVA),
        (game_addr(VERTICES_RVA) as u32, 0xC0),
        at(VERTEX_COUNT_RVA),
        (game_addr(rva(0x7687F4)) as u32, 6 * 0x20),
        at(rva(0x93CC94)),
    ];
    regions.extend(SCRATCH.iter().map(|&(r, l)| (game_addr(r) as u32, l)));
    let mut vars = regions.clone();
    vars.extend([COUNT_RVA, FALL_RVA, FADING_RVA, V_RVA[0], V_RVA[1]].map(at));
    for r in RANDOMS
        .iter()
        .chain(REMAINDERS.iter())
        .chain([rva(0x8D0000)].iter())
    {
        vars.push((game_addr(*r) as u32, 4 * FLAKES as usize));
    }
    vars.push((game_addr(CURVE_RVA) as u32, 0x400 * 4));
    vars.push((game_addr(rva(0x8D2208)) as u32, 0x3C)); // the places, the shift, the matrix
    vars.push((game_addr(rva(0x8DB430)) as u32, 0x18));
    vars.push((game_addr(FADES_RVA) as u32, 4 * FADES as usize));
    vars.extend(
        [
            rva(0x76516C),
            rva(0x76518C),
            rva(0x8DB548),
            rva(0x8D21F8),
            rva(0x8DB54C),
        ]
        .map(at),
    );
    let saved = capture(&vars);
    let put = |r: usize, v: f32| unsafe { game_write(r, v.to_bits()) };
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..4u32 {
        let k = case as f32;
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(COUNT_RVA, [FLAKES, 0, 7, FLAKES][case as usize]);
            game_write(FALL_RVA, 0x3_4567 * (case + 1));
            game_write(FADING_RVA, u32::from(case % 2 == 1));
            for n in 0..FLAKES {
                for (m, r) in RANDOMS.iter().enumerate() {
                    game_write(
                        *r + 4 * n as usize,
                        (n * 0x1_2345 + m as u32 * 0x9_8765) & 0xF_FFFF,
                    );
                }
                for r in REMAINDERS {
                    game_write(r + 4 * n as usize, (n * 37) % 0x200);
                }
                put(rva(0x8D0000) + 4 * n as usize, n as f32 / 16.0);
            }
            for n in 0..0x400 {
                game_write(CURVE_RVA + 4 * n, (n as u32 * 977) % 0x4000);
            }
            for n in 0..3 {
                game_write(SHIFT_RVA + 4 * n, 0x1000 * n as u32);
                game_write(rva(0x8D2208) + 4 * n, 0);
                for m in 0..3 {
                    put(
                        rva(0x8D2220) + 0xC * n + 4 * m,
                        if n == m { 1.0 } else { 0.0 },
                    );
                }
                put(rva(0x8DB430) + 4 * n, n as f32 * 1000.0 - k);
                put(rva(0x8DB43C) + 4 * n, 0.5 - n as f32 * 0.25);
            }
            // The depth (z) well in front: the third row's place.
            game_write(rva(0x8D2210), 0x9_0000u32);
            for n in 0..FADES {
                game_write(FADES_RVA + 4 * n as usize, 0x0101_0101 * n);
            }
            put(rva(0x76516C), 1.0);
            put(rva(0x76518C), 12.0 + k);
            put(rva(0x8DB548), 40000.0);
            put(rva(0x8D21F8), 1.0 / 3000.0);
            put(rva(0x8DB54C), 0.125);
            game_write(V_RVA[0], 0u32);
            put(V_RVA[1], 1.0);
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0xC0);
        };
        prepare();
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_snow(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?} {} draws; rust {:x?} {} draws; state first differs at {:?}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b)
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
