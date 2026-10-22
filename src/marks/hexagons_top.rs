//! `marks::hexagons_top` (0x4BAC87): the hexagons' frame from the settings: the vertex
//! ring from its start (`dword_816232` = `unk_625A44`); the ease and the rate; the place
//! (the settings' +0x2720 ..) and two axes (+0x272C .. and +0x2708 .. less it), their
//! lengths' shares (`flt_814C06`, `flt_814C0A`), made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`); nothing more unless the place's way
//! faces the third axis away (a negative dot, `flt_814718`); the 0x30 points of
//! `g_marksHexagonPoints` scaled into `flt_814CFE` .. and turned onto the axes; unless the
//! rate is 0: `marks::hexagons_frame`, the vertices left in the ring drawn (their count by
//! `dword_82E27A`: `dword_62423C` / `dword_625A40`, `sub_41ADB9`, the platform's), and with
//! the settings' +0x2744 and the drift on `marks::hexagons_step`; then `scene::set_texture`
//! (`dword_5D1E10`) and `g_polygonVertexCount` 4.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the path leaves them
//!
//! Caller: `sub_4C74DA` (0x4C7539). The x87 runs are the original's instructions over the
//! game's variables; the vector helpers their Rust versions; its other calls through
//! `part` (1 `hexagons_frame`, 2 the batch's draw, 3 `hexagons_step`, 4 the texture), in the
//! game their Rust versions (the draw through the game: section 15). Refuses null
//! settings, before anything. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose calls go to stand-ins (the vector helpers real), against the Rust version, on
//! settings of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::hexagons_top", rva(0x4BAC87), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BUFFER_RVA: usize = rva(0x816232); // dword_816232
const BUFFER_START: usize = rva(0x625A44); // unk_625A44
const AXES: [usize; 3] = [rva(0x814C1A), rva(0x814C26), rva(0x814C32)];
const PLACE_RVA: usize = rva(0x8161A2); // flt_8161A2
const FACING_RVA: usize = rva(0x814718); // flt_814718
const POINTS_RVA: usize = rva(0x814F3E); // g_marksHexagonPoints: 0x30 of 12 bytes
const SCALED_RVA: usize = rva(0x814CFE); // flt_814CFE ..
const POINTS: u32 = 0x30;
const SHARES: [usize; 2] = [rva(0x814C06), rva(0x814C0A)];
const RATE_RVA: usize = rva(0x814728); // flt_814728
const PER_VERTEX_RVA: usize = rva(0x82E27A); // dword_82E27A
const BATCH: [usize; 2] = [rva(0x62423C), rva(0x625A40)];
const DRIFTING_RVA: usize = rva(0x7652A0); // dword_7652A0
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn addr(r: usize) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4BAC91 .. 0x4BAD3F: `flt_81622A` (the ease by the ticks), `flt_814728` (the rate), the place (`flt_814C0E` ..) and the first axis (`flt_814C1A` ..) from the settings, its length over `flt_82E272` into `flt_814C06`.
unsafe fn frame_axes(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x42E26A]",
            "fimul dword ptr [{b} + 0x364FD0]",
            "fstp dword ptr [{b} + 0x41622A]",
            "fild dword ptr [{s} + 0x2738]",
            "fmul dword ptr [{b} + 0x42E26E]",
            "fstp dword ptr [{b} + 0x414728]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x414C16]",
            "fstp dword ptr [{b} + 0x414C12]",
            "fstp dword ptr [{b} + 0x414C0E]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x414C1A]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x414C1E]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x414C22]",
            "fld dword ptr [{b} + 0x414C1A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x414C1E]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x414C22]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x42E272]",
            "fstp dword ptr [{b} + 0x414C06]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BAD4C .. 0x4BADAC: the second axis (`flt_814C26` ..) from the settings, its length over `flt_82E276` into `flt_814C0A`.
unsafe fn frame_up(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2708]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x414C26]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x414C2A]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x414C2E]",
            "fld dword ptr [{b} + 0x414C26]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x414C2A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x414C2E]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x42E276]",
            "fstp dword ptr [{b} + 0x414C0A]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BADF8 .. 0x4BAE1C: the place again as floats into `flt_8161A2` ...
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

/// The original's 0x4BAE27 .. 0x4BAE4D: the place's way dotted with the third axis into `flt_814718`.
unsafe fn frame_facing(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4161A2]",
            "fmul dword ptr [{b} + 0x414C32]",
            "fld dword ptr [{b} + 0x4161A6]",
            "fmul dword ptr [{b} + 0x414C36]",
            "fld dword ptr [{b} + 0x4161AA]",
            "fmul dword ptr [{b} + 0x414C3A]",
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

/// The original's 0x4BAEAB .. 0x4BAEEC: a point (x, y at p) turned onto the axes (x, y, z back at p).
unsafe fn point_turned(b: u32, p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{p} + 0x0]",
            "fld dword ptr [{p} + 0x4]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C1A]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C26]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{p} + 0x0]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C1E]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x414C2A]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{p} + 0x4]",
            "fmul dword ptr [{b} + 0x414C2E]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x414C22]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{p} + 0x8]",
            b = in(reg) b,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn hexagons_top(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = addr(0);
        game_write(BUFFER_RVA, addr(BUFFER_START));
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
        frame_place(b, s);
        normalise_vector3(addr(PLACE_RVA));
        [g.esi, g.edi] = [addr(PLACE_RVA), a2];
        frame_facing(b);
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
            return Ok(g);
        }
        let [kx, ky] = SHARES.map(|r| game_read::<u32>(r));
        for n in 0..POINTS as usize {
            let p = POINTS_RVA + 12 * n;
            let o = SCALED_RVA + 12 * n;
            game_write(o, x87::mul(game_read(p), kx));
            game_write(o + 4, x87::mul(game_read(p + 4), ky));
            game_write(o + 8, 0u32);
        }
        for n in 0..POINTS {
            point_turned(b, addr(SCALED_RVA) + 12 * n);
        }
        [g.esi, g.edi] = [
            addr(SCALED_RVA) + 12 * POINTS,
            addr(SCALED_RVA) + 12 * POINTS,
        ];
        g.eax = POINTS;
        if game_read::<u32>(RATE_RVA) != 0 {
            part(1, &mut g)?;
            let at_now: u32 = game_read(BUFFER_RVA);
            if at_now != addr(BUFFER_START) {
                g.eax = x87::int_over_int_to_int(
                    at_now.wrapping_sub(addr(BUFFER_START)) as i32,
                    game_read(PER_VERTEX_RVA),
                ) as u32;
                game_write(BUFFER_RVA, addr(BUFFER_START));
                g.ebx = g.eax.wrapping_mul(6);
                g.eax = g.eax.wrapping_mul(12);
                game_write(BATCH[0], g.eax);
                game_write(BATCH[1], g.ebx);
                part(2, &mut g)?;
            }
            g.esi = game_read(SETTINGS_RVA);
            if ((g.esi + 0x2744) as *const u32).read_unaligned() != 0
                && game_read::<u32>(DRIFTING_RVA) != 0
            {
                part(3, &mut g)?;
            }
        }
        g.eax = game_read(TEXTURE_RVA);
        part(4, &mut g)?;
        game_write(COUNT_RVA, 4u32);
    }
    Ok(g)
}

/// In the game: the calls' Rust versions (the batch's draw through the game).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::{hexagons_frame, hexagons_step};
    unsafe {
        match place {
            1 => {
                let left = hexagons_frame::hexagons_frame(r, &mut hexagons_frame::game_part)?;
                *r = Regs { esp: r.esp, ..left };
            }
            2 => crate::calls::call(rva(0x41ADB9), r),
            3 => {
                let left = hexagons_step::hexagons_step(r);
                *r = Regs { esp: r.esp, ..left };
            }
            _ => crate::scene::set_texture::set_texture_in_game(r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { hexagons_top(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov dword_816232, offset unk_625A44
    let entry: &[u8] = &[0xC7, 0x05, 0x32, 0x62, 0x81, 0x00, 0x44, 0x5A, 0x62, 0x00];
    let copy = match copy(
        0x4BAC87,
        0x310,
        entry,
        &[
            0xBF, 0x12C, 0x143, 0x14E, 0x165, 0x19B, 0x281, 0x2CF, 0x2F6, 0x300,
        ],
        &[(0x4BC0B1, 1), (0x41ADB9, 2), (0x4BCAD0, 3), (0x44E77E, 4)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2748 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        at(BUFFER_RVA),
        (addr(rva(0x814C06)), 0x38),
        (addr(rva(0x81622A)), 4),
        (addr(RATE_RVA), 4),
        (addr(PLACE_RVA), 12),
        (addr(FACING_RVA), 4),
        (addr(SCALED_RVA), (12 * POINTS) as usize),
        at(BATCH[0]),
        at(BATCH[1]),
        at(COUNT_RVA),
    ];
    let mut vars = regions.clone();
    vars.extend([SETTINGS_RVA, DRIFTING_RVA, PER_VERTEX_RVA, TEXTURE_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..4u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SETTINGS_RVA, sa);
                let p = |o: u32, v: u32| ((sa + o) as *mut u32).write_unaligned(v);
                for n in 0..3u32 {
                    p(0x2720 + 4 * n, [100, 50, 400][n as usize] + case * 9);
                    p(0x272C + 4 * n, [300, 60, 380][n as usize]);
                    p(0x2708 + 4 * n, [90, 250, 420][n as usize]);
                }
                p(0x2738, if case == 3 { 0 } else { 5 + case });
                p(0x2744, case % 2);
                game_write(DRIFTING_RVA, 1u32);
                game_write(PER_VERTEX_RVA, 0x20u32);
                game_write(TEXTURE_RVA, 0x40 + case);
                // The rate's and the ease's factors: as the game has them.
                if case == 1 {
                    game_write(BUFFER_RVA, addr(BUFFER_START) + 0x180);
                }
            },
            &|r| unsafe { hexagons_top(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
