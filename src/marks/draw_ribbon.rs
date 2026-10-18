//! `marks::draw_ribbon` (0x4C5E63): a ribbon piece between two places already in the
//! view's frame (`flt_8EB8D5` .. and `dword_8EB8E1` ..): half the width (edx >> 1, whole)
//! in `dword_8EB8B5`, the four vertices' colour (ecx << 24) | esi and texture places from
//! cell eax of `g_marksCellPlaces`; unless either depth is at or below `dword_8EB961` (a
//! signed compare of the bits): both projected (`flt_8EB8CD`, `flt_8EB8D1` their scales),
//! the way between them made one long and turned a quarter (`flt_8F3985`, `flt_8F3989`),
//! the sides that times the half width and each scale (`flt_8F3991` ..), the four corners,
//! the depths times `flt_8EB96D`; `marks::clip_quad`, and unless all is outside
//! `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  eax the cell, ecx the alpha, edx the width, esi the colour (and the two places)
//! Out: ecx the failing depth's bits when too near; else as `clip_quad` / `draw_fan`
//!      leave them (eax the cell times 16, ebp, edi, edx the cell's first, second and
//!      fourth places going in)
//!
//! Callers: the marks' lines (0x4C5D65) and 0x4CFAA3. The x87 block is the original's
//! instructions over the game's variables; the clip its Rust version (the copy's call
//! goes to its hook); the draw through `part` (1 `draw_fan`), in the game its Rust
//! version. Refuses, before anything, a cell past 0x40 and null marks settings (the
//! clip's refusal). Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose draw goes to a stand-in, against the Rust version, places near and far.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::clip_quad::clip_quad;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::draw_ribbon", rva(0x4C5E63), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const HALF_RVA: usize = rva(0x8EB8B5); // dword_8EB8B5
const DEPTHS_RVA: [usize; 2] = [rva(0x8EB8DD), rva(0x8EB8E9)];
const NEAR_RVA: usize = rva(0x8EB961); // dword_8EB961
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const CELLS_RVA: usize = rva(0x935766); // g_marksCellPlaces: 0x41 of 16 bytes
const CELLS: u32 = 0x41;

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The original's 0x4C5EF7 .. 0x4C60E9: the two projected and the ribbon's corners.
unsafe fn ribbon(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x4EB8DD]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4EB8D5]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4EB8D9]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x4EB8D9]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x4EB8D5]",
            "fstp dword ptr [{b} + 0x4EB8CD]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x4EB8E9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4EB8E1]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4EB8E5]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x4EB8E5]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x4EB8E1]",
            "fstp dword ptr [{b} + 0x4EB8D1]",
            "fld dword ptr [{b} + 0x4EB8E1]",
            "fsub dword ptr [{b} + 0x4EB8D5]",
            "fstp dword ptr [{b} + 0x4F3985]",
            "fld dword ptr [{b} + 0x4EB8E5]",
            "fsub dword ptr [{b} + 0x4EB8D9]",
            "fstp dword ptr [{b} + 0x4F3989]",
            "fld dword ptr [{b} + 0x4F3985]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x4F3989]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            ".byte 0xD9, 0xE8",
            ".byte 0xDE, 0xF1",
            "fld dword ptr [{b} + 0x4F3985]",
            "fld dword ptr [{b} + 0x4F3989]",
            ".byte 0xD8, 0xCA",
            "fstp dword ptr [{b} + 0x4F3985]",
            ".byte 0xDE, 0xC9",
            ".byte 0xD9, 0xE0",
            "fstp dword ptr [{b} + 0x4F3989]",
            "fld dword ptr [{b} + 0x4F3985]",
            "fimul dword ptr [{b} + 0x4EB8B5]",
            "fmul dword ptr [{b} + 0x4EB8CD]",
            "fstp dword ptr [{b} + 0x4F3991]",
            "fld dword ptr [{b} + 0x4F3989]",
            "fimul dword ptr [{b} + 0x4EB8B5]",
            "fmul dword ptr [{b} + 0x4EB8CD]",
            "fstp dword ptr [{b} + 0x4F3995]",
            "fld dword ptr [{b} + 0x4F3985]",
            "fimul dword ptr [{b} + 0x4EB8B5]",
            "fmul dword ptr [{b} + 0x4EB8D1]",
            "fstp dword ptr [{b} + 0x4F399D]",
            "fld dword ptr [{b} + 0x4F3989]",
            "fimul dword ptr [{b} + 0x4EB8B5]",
            "fmul dword ptr [{b} + 0x4EB8D1]",
            "fstp dword ptr [{b} + 0x4F39A1]",
            "fld dword ptr [{b} + 0x4EB8D5]",
            "fsub dword ptr [{b} + 0x4F3991]",
            "fstp dword ptr [{b} + 0x5605A0]",
            "fld dword ptr [{b} + 0x4EB8D9]",
            "fsub dword ptr [{b} + 0x4F3995]",
            "fstp dword ptr [{b} + 0x5605A4]",
            "fld dword ptr [{b} + 0x4EB8D5]",
            "fadd dword ptr [{b} + 0x4F3991]",
            "fstp dword ptr [{b} + 0x5605C0]",
            "fld dword ptr [{b} + 0x4EB8D9]",
            "fadd dword ptr [{b} + 0x4F3995]",
            "fstp dword ptr [{b} + 0x5605C4]",
            "fld dword ptr [{b} + 0x4EB8E1]",
            "fadd dword ptr [{b} + 0x4F399D]",
            "fstp dword ptr [{b} + 0x5605E0]",
            "fld dword ptr [{b} + 0x4EB8E5]",
            "fadd dword ptr [{b} + 0x4F39A1]",
            "fstp dword ptr [{b} + 0x5605E4]",
            "fld dword ptr [{b} + 0x4EB8E1]",
            "fsub dword ptr [{b} + 0x4F399D]",
            "fstp dword ptr [{b} + 0x560600]",
            "fld dword ptr [{b} + 0x4EB8E5]",
            "fsub dword ptr [{b} + 0x4F39A1]",
            "fstp dword ptr [{b} + 0x560604]",
            "fld dword ptr [{b} + 0x4EB8DD]",
            "fmul dword ptr [{b} + 0x4EB96D]",
            "fst dword ptr [{b} + 0x5605A8]",
            "fstp dword ptr [{b} + 0x5605C8]",
            "fld dword ptr [{b} + 0x4EB8E9]",
            "fmul dword ptr [{b} + 0x4EB96D]",
            "fst dword ptr [{b} + 0x5605E8]",
            "fstp dword ptr [{b} + 0x560608]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_ribbon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.eax >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    unsafe {
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        g.edx >>= 1;
        game_write(HALF_RVA, g.edx);
        g.ecx = (g.ecx << 24) | g.esi;
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, g.ecx);
        }
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
        for r in DEPTHS_RVA {
            g.ecx = game_read(r);
            if (g.ecx as i32) <= game_read::<i32>(NEAR_RVA) {
                return Ok(g);
            }
        }
        ribbon(game_addr(0) as u32);
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_ribbon(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::take_called;
    use crate::race::mud::{copy, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(crate::marks::setup::SETTINGS_RVA) } == 0 {
        return t.skip("no marks settings (the clip reads them)");
    }
    // shr edx, 1; mov dword_8EB8B5, edx
    let entry: &[u8] = &[0xD1, 0xEA, 0x89, 0x15, 0xB5, 0xB8, 0x8E, 0x00];
    let copy = match copy(0x4C5E63, 0x2A3, entry, &[0x28C, 0x293], &[(0x44EA65, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(rva(0x8EB8B5)) as u32, 0x38),
        (game_addr(rva(0x8F3985)) as u32, 0x20),
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
        ]
        .map(at),
    );
    let saved = capture(&vars);
    let put = |r: usize, v: f32| unsafe { game_write(r, v.to_bits()) };
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let k = case as f32;
        let before = Regs {
            eax: 3 + case * 9,
            ecx: 0x80 + case,
            edx: 9 + case * 4,
            esi: 0x0012_3456,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            put(NEAR_RVA, 1.0);
            put(rva(0x624088), 320.0);
            put(rva(0x62409C), 320.5);
            put(rva(0x6240A0), 240.25);
            put(rva(0x8EB96D), 1.0 / 3000.0);
            put(rva(0x8EB8D5), k * 3.0 - 8.0);
            put(rva(0x8EB8D9), 2.0 + k);
            put(rva(0x8EB8DD), if case == 3 { 0.5 } else { 40.0 + k * 9.0 });
            put(rva(0x8EB8E1), k * 3.0 - 6.5);
            put(rva(0x8EB8E5), 2.5 - k);
            put(rva(0x8EB8E9), if case == 4 { -3.0 } else { 45.0 + k * 7.0 });
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0xC0);
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe { draw_ribbon(&before, &mut test_part) };
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
    t.finish()
}
