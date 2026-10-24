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
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C74DA` (0x4C7539). The x87 runs are the original's instructions over the
//! game's variables; the vector helpers their Rust versions; its other calls through
//! `part` (1 `hexagons_frame`, 2 the batch's draw, 3 `hexagons_step`, 4 the texture), in the
//! game their Rust versions (the draw through the game: section 15). Refuses null
//! settings, before anything. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose calls go to stand-ins (the vector helpers real), against the Rust version, on
//! settings of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;

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

/// The Rust version (see the header).
pub unsafe fn hexagons_top() -> Result<(), Refused> {
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = addr(0);
        game_write(BUFFER_RVA, addr(BUFFER_START));
        frame_axes(b, s);
        let [a0, a1, a2] = AXES.map(addr);
        normalise_vector3(a0);
        frame_up(b, s);
        normalise_vector3(a1);
        cross_product3(a1, a0, a2)?;
        normalise_vector3(a2);
        cross_product3(a0, a2, a1)?;
        frame_place(b, s);
        normalise_vector3(addr(PLACE_RVA));
        frame_facing(b);
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
            return Ok(());
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
        if game_read::<u32>(RATE_RVA) != 0 {
            crate::marks::hexagons_frame::hexagons_frame()?;
            let at_now: u32 = game_read(BUFFER_RVA);
            if at_now != addr(BUFFER_START) {
                let quads = x87::int_over_int_to_int(
                    at_now.wrapping_sub(addr(BUFFER_START)) as i32,
                    game_read(PER_VERTEX_RVA),
                ) as u32;
                game_write(BUFFER_RVA, addr(BUFFER_START));
                game_write(BATCH[0], quads.wrapping_mul(12));
                game_write(BATCH[1], quads.wrapping_mul(6));
                crate::marks::hexagons_frame::draw_batch()?;
            }
            let settings: u32 = game_read(SETTINGS_RVA);
            if ((settings + 0x2744) as *const u32).read_unaligned() != 0
                && game_read::<u32>(DRIFTING_RVA) != 0
            {
                crate::marks::hexagons_step::hexagons_step();
            }
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        game_write(COUNT_RVA, 4u32);
    }
    Ok(())
}
