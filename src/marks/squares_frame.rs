//! `marks::squares_frame` (0x4BF60F): the squares' frame (as `marks::draw_plane`'s, with
//! the rate times `flt_8BC19E` and the lengths over `flt_8BC1A2` / `flt_8BC1A6`): the
//! rate, the place and two axes from the settings, made one long and square, scaled
//! (`marks::scale_axes`); nothing more unless the place's way faces the third axis away;
//! then `marks::run_pong`; unless the rate times `flt_8BC1AA` comes to 0 (`fistp`, as the
//! x87 is set) `marks::run_squares`, and with the settings' +0x2744 and `dword_7652A0`
//! `marks::squares_step`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the path leaves them
//!
//! Caller: `sub_4C74DA` (0x4C7534). The x87 runs are the original's instructions over the
//! game's variables; the vector helpers, `scale_axes` and `squares_step` their Rust
//! versions; pong and the squares through `part` (1 `run_pong`, 2 `run_squares`), in the
//! game their Rust versions. Refuses null settings and `squares_step`'s refusals. Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose pong and squares go
//! to stand-ins, against the Rust version, on settings of the test's; put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::squares_step::{self, squares_step};
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::squares_frame", rva(0x4BF60F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const AXES: [usize; 3] = [rva(0x89CF1A), rva(0x89CF26), rva(0x89CF32)];
const RATE_RVA: usize = rva(0x89CA34); // flt_89CA34
const PLACE_RVA: usize = rva(0x8161A2); // flt_8161A2
const FACING_RVA: usize = rva(0x814718); // flt_814718
const SHARE_RVA: usize = rva(0x8BC1AA); // flt_8BC1AA
const RUNNING_RVA: usize = rva(0x7652A0); // dword_7652A0

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn addr(r: usize) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4BF615 .. 0x4BF6AB: the rate (`flt_89CA34`), the place and the first axis from the settings, its length over `flt_8BC1A2` into `flt_89CEEE`.
unsafe fn frame_axes(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2740]",
            "fmul dword ptr [{b} + 0x4BC19E]",
            "fstp dword ptr [{b} + 0x49CA34]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CEFE]",
            "fstp dword ptr [{b} + 0x49CEFA]",
            "fstp dword ptr [{b} + 0x49CEF6]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x49CF1A]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x49CF1E]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CF22]",
            "fld dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF22]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x4BC1A2]",
            "fstp dword ptr [{b} + 0x49CEEE]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF6B8 .. 0x4BF718: the second axis, its length over `flt_8BC1A6` into `flt_89CEF2`.
unsafe fn frame_up(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2708]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x49CF26]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x49CF2A]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CF2E]",
            "fld dword ptr [{b} + 0x49CF26]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF2A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF2E]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x4BC1A6]",
            "fstp dword ptr [{b} + 0x49CEF2]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF769 .. 0x4BF78D: the place as floats into `flt_8161A2` ...
unsafe fn frame_place(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x4161A2]",
            "fild dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x4161A6]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x4161AA]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF7A4 .. 0x4BF7BE: the place's way dotted with the third axis into `flt_814718`.
unsafe fn frame_facing(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4161A2]",
            "fmul dword ptr [{b} + 0x49CF32]",
            "fld dword ptr [{b} + 0x4161A6]",
            "fmul dword ptr [{b} + 0x49CF36]",
            "fld dword ptr [{b} + 0x4161AA]",
            "fmul dword ptr [{b} + 0x49CF3A]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x414718]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn squares_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = addr(0);
        g.esi = s;
        frame_axes(b, s);
        let [a0, a1, a2] = AXES.map(addr);
        normalise_vector3(a0);
        frame_up(b, s);
        normalise_vector3(a1);
        [g.esi, g.edi, g.ebp] = [a1, a0, a2];
        cross_product3(a1, a0, a2)?;
        normalise_vector3(a2);
        [g.esi, g.edi, g.ebp] = [a0, a2, a1];
        cross_product3(a0, a2, a1)?;
        g.esi = s;
        g = crate::marks::scale_axes::scale_axes(&g);
        g.esi = s;
        frame_place(b, s);
        normalise_vector3(addr(PLACE_RVA));
        [g.esi, g.edi] = [addr(PLACE_RVA), a2];
        frame_facing(b);
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
            return Ok(g);
        }
        part(1, &mut g)?;
        g.eax = x87::mul_to_int(game_read(RATE_RVA), game_read(SHARE_RVA)) as u32;
        if g.eax == 0 {
            return Ok(g);
        }
        part(2, &mut g)?;
        g.esi = game_read(SETTINGS_RVA);
        if ((g.esi + 0x2744) as *const u32).read_unaligned() == 0 {
            return Ok(g);
        }
        if game_read::<u32>(RUNNING_RVA) == 0 {
            return Ok(g);
        }
        g = squares_step(&g)?;
    }
    Ok(g)
}

/// In the game: pong's and the squares' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::{run_pong, run_squares};
    unsafe {
        *r = match place {
            1 => run_pong::run_pong(r, &mut run_pong::game_part)?,
            _ => run_squares::run_squares(r, &mut run_squares::game_part)?,
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
    match unsafe { squares_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov esi, g_marksSettings
    let entry: &[u8] = &[0x8B, 0x35, 0x28, 0x7A, 0x76, 0x00];
    let copy = match copy(
        0x4BF60F,
        0x200,
        entry,
        &[
            0xA3, 0x110, 0x127, 0x132, 0x149, 0x14F, 0x184, 0x1BC, 0x1DA, 0x1FA,
        ],
        &[(0x4B7EDA, 1), (0x4BFFEC, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2748 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let mut regions: Vec<Region> = vec![
        at(RATE_RVA),
        (addr(rva(0x89CEEE)), 0xA0),
        (addr(PLACE_RVA), 12),
        at(FACING_RVA),
    ];
    regions.extend(squares_step::regions());
    let mut vars = regions.clone();
    vars.extend(
        [
            SETTINGS_RVA,
            SHARE_RVA,
            RUNNING_RVA,
            rva(0x82FC56),
            rva(0x833D7E),
        ]
        .map(at),
    );
    let saved = capture(&vars);
    for case in 0..8u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SETTINGS_RVA, sa);
                let p = |o: u32, v: u32| ((sa + o) as *mut u32).write_unaligned(v);
                let flip = if case % 2 == 0 { 1i32 } else { -1 };
                for n in 0..3u32 {
                    p(
                        0x2720 + 4 * n,
                        ([100i32, 50, 400][n as usize] * flip + case as i32 * 9) as u32,
                    );
                    p(0x272C + 4 * n, [300, 60, 380][n as usize]);
                    p(0x2708 + 4 * n, [90, 250, 420][n as usize]);
                }
                p(0x2740, if case == 6 { 0 } else { 3 + case });
                p(0x2744, u32::from(case != 4));
                game_write(SHARE_RVA, 0.5f32.to_bits());
                game_write(RUNNING_RVA, u32::from(case != 2));
                crate::marks::expire_squares::test_items(case);
            },
            &|r| unsafe { squares_frame(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
