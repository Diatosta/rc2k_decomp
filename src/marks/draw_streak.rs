//! `marks::draw_streak` (0x4C1DCD): a streak (a point and where it was) drawn as a quad
//! facing the viewer. The point (st(0) .. st(2), scaled by `flt_8BDD65` .. into
//! `flt_8BDE49` ..) turned by the matrix at `dword_8BDD89` .. plus the whole numbers at
//! `dword_8BDD71` .. into `flt_8BDE3D` .. (its depth also `flt_8BDD5D`); the point less
//! `dword_8BC47D` .. turned the same into `flt_8BDE49` ... Unless the first depth is at or
//! below 0x20000 or the second at or below 0x48000000 (signed compares of the bits), or
//! `dword_8BC405` is 0: both projected (`flt_624088` over the depth; scales `flt_8C5FE5`,
//! `flt_8C5FE9`, centre `flt_62409C`, `flt_6240A0`) into `flt_8BDD41` .., the half width
//! `flt_8C5FED` over the screen length across the line, the four vertices' x, y and depth
//! (times `flt_8BDD61`), `marks::clip_quad`, and unless all is outside `scene::draw_fan`
//! and `g_polygonVertexCount` 4. Rain, by the look of it (a reading).
//!
//! In:  st(0) .. st(2) the point (popped)
//! Out: the registers as they come when not drawn; else as `clip_quad` / `draw_fan` leave
//!      them (edi `flt_8BDE3D`, esi `flt_8BDE49` going in)
//!
//! Caller: `sub_4C16F9` (0x4C1AD0). The arithmetic is the original's instructions over the
//! game's variables (register forms as bytes); the clip its Rust version (the copy's call
//! goes to its hook); the draw through `part` (1 `draw_fan`), in the game its Rust version.
//! Refuses null marks settings (`clip_quad`'s refusal), before anything. Thunk by
//! `three_in_hook!`. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose draw goes to a stand-in, against the Rust version, points near and far.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva, shadow};
use crate::marks::clip_quad::clip_quad;
use crate::scene::matrix3::extended;
use crate::scene::vector3::{Triple80, call_with_three, three_in_hook};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static TRIPLE: Triple80 = Triple80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::draw_streak", rva(0x4C1DCD), thunk, &ORIGINAL).self_test(self_test);

three_in_hook!(
    thunk,
    call_original,
    original_with_three,
    dispatch = dispatch,
    original = ORIGINAL,
    triple = TRIPLE
);

const WORK_RVA: usize = rva(0x8BDD29); // flt_8BDD29 .. flt_8BDE51
const WORK_LEN: usize = 0x12C;
const DEPTH_RVA: usize = rva(0x8BDD5D); // flt_8BDD5D
const FAR_RVA: usize = rva(0x8BDE51); // flt_8BDE51
const SWITCH_RVA: usize = rva(0x8BC405); // dword_8BC405
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The original's 0x4C1DCD .. 0x4C1F11: the point scaled and turned, and the last place.
unsafe fn turn(b: u32, p: &[[u8; 10]; 3]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{b} + 0x4BDD65]",
            "fst dword ptr [{b} + 0x4BDE49]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4BDD69]",
            "fst dword ptr [{b} + 0x4BDE4D]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{b} + 0x4BDD6D]",
            "fst dword ptr [{b} + 0x4BDE51]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4BDD89]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{b} + 0x4BDD95]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{b} + 0x4BDDA1]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4BDD71]",
            "fstp dword ptr [{b} + 0x4BDE3D]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4BDD8D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{b} + 0x4BDD99]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{b} + 0x4BDDA5]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4BDD75]",
            "fstp dword ptr [{b} + 0x4BDE41]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4BDD91]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{b} + 0x4BDD9D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4BDDA9]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4BDD79]",
            "fst dword ptr [{b} + 0x4BDD5D]",
            "fstp dword ptr [{b} + 0x4BDE45]",
            "fld dword ptr [{b} + 0x4BDE4D]",
            "fsub dword ptr [{b} + 0x4BC481]",
            "fld dword ptr [{b} + 0x4BDE49]",
            "fsub dword ptr [{b} + 0x4BC47D]",
            "fld dword ptr [{b} + 0x4BDE51]",
            "fsub dword ptr [{b} + 0x4BC485]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4BDD89]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{b} + 0x4BDD95]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{b} + 0x4BDDA1]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4BDD71]",
            "fstp dword ptr [{b} + 0x4BDE49]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x4BDD8D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{b} + 0x4BDD99]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{b} + 0x4BDDA5]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4BDD75]",
            "fstp dword ptr [{b} + 0x4BDE4D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4BDD91]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{b} + 0x4BDD9D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4BDDA9]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{b} + 0x4BDD79]",
            "fstp dword ptr [{b} + 0x4BDE51]",
            b = in(reg) b,
            p = in(reg) p.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C1F4A .. 0x4C20B3: both projected and the quad's x, y and depth.
unsafe fn project(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4BDE51]",
            "fdivr dword ptr [{b} + 0x224088]",
            "fld dword ptr [{b} + 0x4BDE45]",
            "fdivr dword ptr [{b} + 0x224088]",
            "fld dword ptr [{b} + 0x4BDE49]",
            "fld dword ptr [{b} + 0x4BDE4D]",
            "fld dword ptr [{b} + 0x4BDE3D]",
            "fld dword ptr [{b} + 0x4BDE41]",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xCA",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xC9",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCD",
            "fmul dword ptr [{b} + 0x4C5FE5]",
            ".byte 0xD9, 0xCC",
            "fmul dword ptr [{b} + 0x4C5FE9]",
            ".byte 0xD9, 0xCA",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x4BDD41]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x4BDD45]",
            ".byte 0xD9, 0xC9",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x4BDD4D]",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x4BDD51]",
            "fstp dword ptr [{b} + 0x4BDD49]",
            "fstp dword ptr [{b} + 0x4BDD55]",
            ".byte 0xD9, 0xE8",
            "fdiv dword ptr [{b} + 0x2241C8]",
            "fld dword ptr [{b} + 0x4BDD4D]",
            "fsub dword ptr [{b} + 0x4BDD41]",
            "fst dword ptr [{b} + 0x4BDD2D]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x4BDD51]",
            "fsub dword ptr [{b} + 0x4BDD45]",
            ".byte 0xD9, 0xE0",
            "fst dword ptr [{b} + 0x4BDD29]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{b} + 0x4C5FED]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4BDD29]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4BDD49]",
            "fstp dword ptr [{b} + 0x4BDD29]",
            "fmul dword ptr [{b} + 0x4BDD55]",
            "fstp dword ptr [{b} + 0x4BDD35]",
            "fmul dword ptr [{b} + 0x4BDD2D]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4BDD49]",
            "fstp dword ptr [{b} + 0x4BDD2D]",
            "fmul dword ptr [{b} + 0x4BDD55]",
            "fstp dword ptr [{b} + 0x4BDD39]",
            "fstp dword ptr [{b} + 0x4BDD61]",
            "fld dword ptr [{b} + 0x4BDE51]",
            "fmul dword ptr [{b} + 0x4BDD61]",
            "fst dword ptr [{b} + 0x5605A8]",
            "fst dword ptr [{b} + 0x5605C8]",
            "fst dword ptr [{b} + 0x5605E8]",
            "fstp dword ptr [{b} + 0x560608]",
            "fld dword ptr [{b} + 0x4BDD41]",
            "fst dword ptr [{b} + 0x5605A0]",
            "fsub dword ptr [{b} + 0x4BDD29]",
            "fstp dword ptr [{b} + 0x5605C0]",
            "fld dword ptr [{b} + 0x4BDD4D]",
            "fst dword ptr [{b} + 0x560600]",
            "fsub dword ptr [{b} + 0x4BDD35]",
            "fstp dword ptr [{b} + 0x5605E0]",
            "fld dword ptr [{b} + 0x4BDD45]",
            "fst dword ptr [{b} + 0x5605A4]",
            "fsub dword ptr [{b} + 0x4BDD2D]",
            "fstp dword ptr [{b} + 0x5605C4]",
            "fld dword ptr [{b} + 0x4BDD51]",
            "fst dword ptr [{b} + 0x560604]",
            "fsub dword ptr [{b} + 0x4BDD39]",
            "fstp dword ptr [{b} + 0x5605E4]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_streak(entry: &Regs, place: [[u8; 10]; 3], part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = game_addr(0) as u32;
        turn(b, &place);
        if game_read::<i32>(DEPTH_RVA) <= 0x2_0000
            || game_read::<u32>(SWITCH_RVA) == 0
            || game_read::<i32>(FAR_RVA) <= 0x4800_0000
        {
            return Ok(g);
        }
        project(b);
        g.edi = game_addr(rva(0x8BDE3D)) as u32;
        g.esi = game_addr(rva(0x8BDE49)) as u32;
        g.ebp = game_addr(VERTICES_RVA) as u32;
        g = clip_quad(&g)?;
        if g.carry() {
            return Ok(g);
        }
        part(1, &mut g)?;
        game_write(COUNT_RVA, 4u32);
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
        unsafe { original_with_three(r) };
        return;
    }
    match unsafe { draw_streak(r, TRIPLE.inputs(), &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, original_with_three) },
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
    // fxch st(2); fmul flt_8BDD65
    let entry: &[u8] = &[0xD9, 0xCA, 0xD8, 0x0D, 0x65, 0xDD, 0x8B, 0x00];
    let copy = match copy(0x4C1DCD, 0x303, entry, &[0x2EC, 0x2F3], &[(0x44EA65, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(WORK_RVA) as u32, WORK_LEN),
        (game_addr(VERTICES_RVA) as u32, 0xC0),
        at(COUNT_RVA),
        (game_addr(rva(0x7687F4)) as u32, 6 * 0x20),
        at(rva(0x93CC94)),
    ];
    let mut vars = regions.clone();
    vars.extend(
        [
            SWITCH_RVA,
            rva(0x624088),
            rva(0x62409C),
            rva(0x6240A0),
            rva(0x6241C8),
        ]
        .map(at),
    );
    vars.push((game_addr(rva(0x8BC47D)) as u32, 12));
    vars.push((game_addr(rva(0x8C5FE5)) as u32, 12));
    let saved = capture(&vars);
    let put = |r: usize, v: f32| unsafe { game_write(r, v.to_bits()) };
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let k = case as f32;
        let place = [
            extended((k * 3.0 - 5.0).to_bits()),
            extended((2.0 + k).to_bits()),
            extended((if case == 3 { -40.0 } else { 30.0 + k * 20.0 }).to_bits()),
        ];
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(SWITCH_RVA, u32::from(case != 4));
            // Scales 1, the turn the identity, places 0, the last place a little back.
            for n in 0..3 {
                put(rva(0x8BDD65) + 4 * n, 1.0);
                game_write(rva(0x8BDD71) + 4 * n, 0u32);
                for m in 0..3 {
                    put(
                        rva(0x8BDD89) + 0xC * n + 4 * m,
                        if n == m { 0x10000 as f32 } else { 0.0 },
                    );
                }
            }
            put(rva(0x8BC47D), 0.25 + k * 0.1);
            put(rva(0x8BC481), -0.5);
            put(rva(0x8BC485), 0.75);
            put(rva(0x624088), 320.0);
            put(rva(0x62409C), 320.5);
            put(rva(0x6240A0), 240.25);
            put(rva(0x6241C8), 3000.0);
            put(rva(0x8C5FE5), 1.0);
            put(rva(0x8C5FE9), 1.0);
            put(rva(0x8C5FED), 1.5 + k);
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0xC0);
        };
        prepare();
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { call_with_three(&mut ro, copy.addr(), &place) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_streak(&Regs::SENTINEL, place, &mut |p, r| {
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
    t.finish()
}
