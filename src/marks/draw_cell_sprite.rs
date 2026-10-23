//! `marks::draw_cell_sprite` (0x4CB796): a ring entry's sprite: the place (st(0) x,
//! st(1) y, st(2) z) turned by `g_viewMatrix` into `flt_8EB8D5` .. (the depth also whole
//! in `dword_9356C0`); unless the depth is at or below `dword_8EB961` (a signed compare of
//! the bits): the four vertices' specular `scene::fog_at` of the depth, colour (ecx's top
//! byte | the entry's +0x90), texture places from cell eax of `g_marksCellPlaces`, depth
//! times `flt_8EB96D`; half the side (edx times `flt_624088` / depth, and that times the
//! cell's `g_marksCornerSlopes`) turned by `g_sines` / `g_cosines`[ebx] into
//! `dword_8EB8BD` ..; the corners about the projected place; `marks::clip_quad`, and unless
//! all is outside `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  st(0) .. st(2) the place (popped), eax the cell, ebx the angle, ecx the alpha (top
//!      byte), edx the size, esi the entry
//! Out: ebp the depth's bits when too near; else as `clip_quad` / `draw_fan` leave them
//!      (eax the cell times 16, ecx / edx the cell's third and fourth places, edi the
//!      turned half side going in)
//!
//! Callers: the ring's kinds 1, 5, 6, 0xD, 0x13, 0x15, 0x16 (`sub_4CB64D`) and 0x4D0D80.
//! The x87 blocks are the original's instructions over the game's variables; the fog and
//! the clip their Rust versions (the copy's calls go to their hooks); the draw through
//! `part` (1 `draw_fan`), in the game its Rust version. Refuses, before anything, a cell
//! past 0x40, an angle of 0x1000 or more, a null entry, and the fog's and the clip's
//! refusals. Thunk by `three_in_hook!`. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose draw goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::marks::clip_quad::clip_quad;
use crate::scene::fog_at::fog_at;
use crate::scene::matrix3::extended;
use crate::scene::vector3::call_with_three;
use crate::service::harness::{Region, at, capture, restore};

const DEPTH_RVA: usize = rva(0x8EB8DD); // flt_8EB8DD
const WHOLE_RVA: usize = rva(0x9356C0); // dword_9356C0
const NEAR_RVA: usize = rva(0x8EB961); // dword_8EB961
const HALF_RVA: usize = rva(0x8EB8BD); // dword_8EB8BD
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const CELLS_RVA: usize = rva(0x935766); // g_marksCellPlaces: 0x41 of 16 bytes
const CELLS: u32 = 0x41;
const ANGLES: u32 = 0x1000;

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The original's 0x4CB796 .. 0x4CB81A: the place turned by the view.
unsafe fn turn(b: u32, p: &[[u8; 10]; 3]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            "fld dword ptr [{b} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{b} + 0x4EB8D5]",
            "fstp dword ptr [{b} + 0x4EB8D9]",
            "fist dword ptr [{b} + 0x5356C0]",
            "fstp dword ptr [{b} + 0x4EB8DD]",
            b = in(reg) b,
            p = in(reg) p.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CB84C .. 0x4CBA37 (its integer moves done before but those it does
/// between the x87 steps): the square, turned, about the projected place. Gives what the
/// original loads into edi at +0x17B: `flt_8EB8C1` between its two stores.
unsafe fn square(b: u32, k: u32, s: u32) -> u32 {
    let edi: u32;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x4EB8DD]",
            "fld dword ptr [{b} + 0x4EB8DD]",
            "fmul dword ptr [{b} + 0x4EB96D]",
            "fst dword ptr [{b} + 0x5605A8]",
            "fst dword ptr [{b} + 0x5605C8]",
            "fst dword ptr [{b} + 0x5605E8]",
            "fstp dword ptr [{b} + 0x560608]",
            "fild dword ptr [{b} + 0x4EB8BD]",
            ".byte 0xD8, 0xC9",
            "fst dword ptr [{b} + 0x4EB8BD]",
            "fmul dword ptr [{b} + {k} + 0x520734]",
            "fstp dword ptr [{b} + 0x4EB8C1]",
            "mov {t}, dword ptr [{b} + 0x4EB8BD]",
            "xor {t}, 0x80000000",
            "mov dword ptr [{b} + 0x4EB8C9], {t}",
            "mov {t}, dword ptr [{b} + 0x4EB8C1]",
            "mov dword ptr [{b} + 0x4EB8C5], {t}",
            "fld dword ptr [{b} + 0x4EB8BD]",
            "fld dword ptr [{b} + 0x4EB8C1]",
            ".byte 0xD9, 0xC1",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + {s} + 0x624AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + {s} + 0x623AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x4EB8BD]",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{b} + {s} + 0x623AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + {s} + 0x624AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x4EB8C1]",
            "fld dword ptr [{b} + 0x4EB8C5]",
            "fld dword ptr [{b} + 0x4EB8C9]",
            ".byte 0xD9, 0xC1",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + {s} + 0x624AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + {s} + 0x623AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x4EB8C5]",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{b} + {s} + 0x623AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + {s} + 0x624AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x4EB8C9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4EB8D5]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4EB8D9]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x4EB8C1]",
            "fstp dword ptr [{b} + 0x5605A4]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x4EB8C9]",
            "fstp dword ptr [{b} + 0x5605C4]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{b} + 0x4EB8C1]",
            "fstp dword ptr [{b} + 0x5605E4]",
            "fadd dword ptr [{b} + 0x4EB8C9]",
            "fstp dword ptr [{b} + 0x560604]",
            "fadd dword ptr [{b} + 0x22409C]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x4EB8BD]",
            "fstp dword ptr [{b} + 0x5605A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x4EB8C5]",
            "fstp dword ptr [{b} + 0x5605C0]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{b} + 0x4EB8BD]",
            "fstp dword ptr [{b} + 0x5605E0]",
            "fadd dword ptr [{b} + 0x4EB8C5]",
            "fstp dword ptr [{b} + 0x560600]",
            b = in(reg) b,
            k = in(reg) k,
            s = in(reg) s,
            t = out(reg) edi,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
    edi
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_cell_sprite(
    entry: &Regs,
    place: [[u8; 10]; 3],
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.eax >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    if g.ebx >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    if g.esi == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    unsafe {
        // The fog's and the clip's refusals, before anything.
        fog_at(0)?;
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = game_addr(0) as u32;
        turn(b, &place);
        g.ebp = game_read(DEPTH_RVA);
        if (g.ebp as i32) <= game_read::<i32>(NEAR_RVA) {
            return Ok(g);
        }
        let fog = fog_at(game_read(WHOLE_RVA))?;
        g.ecx = (g.ecx & 0xFF00_0000) | ((g.esi + 0x90) as *const u32).read_unaligned();
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x14, fog);
            game_write(VERTICES_RVA + 0x20 * n + 0x10, g.ecx);
        }
        game_write(HALF_RVA, g.edx);
        g.eax <<= 4;
        let cell = |n: usize| game_read::<u32>(CELLS_RVA + g.eax as usize + 4 * n);
        [g.ebp, g.edi, g.ecx, g.edx] = [cell(0), cell(1), cell(2), cell(3)];
        let uv = [
            (g.ebp, g.edi),
            (g.ecx, g.edi),
            (g.ecx, g.edx),
            (g.ebp, g.edx),
        ];
        for (n, (u, v)) in uv.into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        g.edi = square(b, g.eax, 4 * g.ebx);
        g.ebp = game_addr(VERTICES_RVA) as u32;
        g = clip_quad(&g)?;
        if g.carry() {
            return Ok(g);
        }
        part(1, &mut g)?;
        game_write(VERTEX_COUNT_RVA, 4u32);
    }
    Ok(g)
}

/// In the game: `draw_fan`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part) };
    Ok(())
}

/// The test, on the copy of the original at `address` (this one or its twin's).
pub fn test_at(hook: &RawHook, address: usize) -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(crate::marks::setup::SETTINGS_RVA) } == 0 {
        return t.skip("no marks settings (the clip reads them)");
    }
    // fld g_viewMatrix
    let entry: &[u8] = &[0xD9, 0x05, 0xCC, 0xA2, 0x5E, 0x00];
    let copy = match copy(
        address,
        0x2BE,
        entry,
        &[0x9C, 0x2A7, 0x2AE],
        &[(0x44EA65, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut ring_entry = vec![0u32; 0x94 / 4];
    ring_entry[0x90 / 4] = 0x0012_3456;
    let ea = ring_entry.as_ptr() as u32;
    let regions: Vec<Region> = vec![
        (game_addr(rva(0x8EB8BD)) as u32, 0x10),
        (game_addr(rva(0x8EB8D5)) as u32, 12),
        at(WHOLE_RVA),
        (game_addr(VERTICES_RVA) as u32, 0xC0),
        at(VERTEX_COUNT_RVA),
        (game_addr(rva(0x7687F4)) as u32, 6 * 0x20),
        at(rva(0x93CC94)),
    ];
    let mut vars = regions.clone();
    vars.extend(
        [
            NEAR_RVA,
            rva(0x624088),
            rva(0x62409C),
            rva(0x6240A0),
            rva(0x8EB96D),
            rva(0x63761C),
        ]
        .map(at),
    );
    vars.push((game_addr(rva(0x5EA2CC)) as u32, 0x40));
    vars.push((game_addr(rva(0x63661C)) as u32, 0x100));
    let saved = capture(&vars);
    let put = |r: usize, v: f32| unsafe { game_write(r, v.to_bits()) };
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let k = case as f32;
        let place = [
            extended((k * 9.0 - 20.0).to_bits()),
            extended((3.0 + k).to_bits()),
            extended((if case == 2 { -10.0 } else { 50.0 + k * 40.0 }).to_bits()),
        ];
        let before = Regs {
            eax: [0, 7, 0x20, 0x40, 3][case as usize],
            ebx: (case * 0x3A1) & 0xFFF,
            ecx: 0x8000_0000 + case,
            edx: 4 + case * 6,
            esi: ea,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            for n in 0..16 {
                put(rva(0x5EA2CC) + 4 * n, if n % 5 == 0 { 1.0 } else { 0.0 });
            }
            put(rva(0x5EA2CC) + 0x38, 2.0);
            put(NEAR_RVA, 1.0);
            put(rva(0x624088), 320.0);
            put(rva(0x62409C), 320.5);
            put(rva(0x6240A0), 240.25);
            put(rva(0x8EB96D), 1.0 / 3000.0);
            game_write(rva(0x63761C), 0x40u32);
            for n in 0..0x100 {
                game_write(rva(0x63661C) + n, (n * 3) as u8);
            }
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0xC0);
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { call_with_three(&mut ro, copy.addr(), &place) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_cell_sprite(&before, place, &mut |p, r| {
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
                "case {case}: original {:x?} {:x?}; rust {:x?} {:x?}; state first differs at {:?}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b)
            )
        });
    }
    restore(&vars, &saved);
    drop(ring_entry);
    t.finish()
}
