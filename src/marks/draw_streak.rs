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
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C16F9` (0x4C1AD0). The arithmetic is the original's instructions over the
//! game's variables (register forms as bytes); the clip its Rust version (the copy's call
//! goes to its hook); the draw through `part` (1 `draw_fan`), in the game its Rust version.
//! Refuses null marks settings (`clip_quad`'s refusal), before anything. Thunk by
//! `three_in_hook!`. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose draw goes to a stand-in, against the Rust version, points near and far.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};

// flt_8BDD29 .. flt_8BDE51
const DEPTH_RVA: Slot = Slot::at(0x8BDD5D); // flt_8BDD5D
const FAR_RVA: Slot = Slot::at(0x8BDE51); // flt_8BDE51
const SWITCH_RVA: Slot = Slot::at(0x8BC405); // dword_8BC405
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount

/// The original's 0x4C1DCD .. 0x4C1F11: the point scaled and turned, and the last place.
unsafe fn turn(p: &[[u8; 10]; 3]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E25]",
            "fst dword ptr [{mk} + 0x158F09]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E29]",
            "fst dword ptr [{mk} + 0x158F0D]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E2D]",
            "fst dword ptr [{mk} + 0x158F11]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E49]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E55]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E61]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E31]",
            "fstp dword ptr [{mk} + 0x158EFD]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E4D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E59]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E65]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E35]",
            "fstp dword ptr [{mk} + 0x158F01]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E51]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E5D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E69]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E39]",
            "fst dword ptr [{mk} + 0x158E1D]",
            "fstp dword ptr [{mk} + 0x158F05]",
            "fld dword ptr [{mk} + 0x158F0D]",
            "fsub dword ptr [{mk} + 0x157541]",
            "fld dword ptr [{mk} + 0x158F09]",
            "fsub dword ptr [{mk} + 0x15753D]",
            "fld dword ptr [{mk} + 0x158F11]",
            "fsub dword ptr [{mk} + 0x157545]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E49]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E55]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E61]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E31]",
            "fstp dword ptr [{mk} + 0x158F09]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E4D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E59]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E65]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E35]",
            "fstp dword ptr [{mk} + 0x158F0D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E51]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E5D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E69]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E39]",
            "fstp dword ptr [{mk} + 0x158F11]",
            mk = in(reg) crate::marks::store::base(),
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
            "fld dword ptr [{mk} + 0x158F11]",
            "fdivr dword ptr [{b} + 0x224088]",
            "fld dword ptr [{mk} + 0x158F05]",
            "fdivr dword ptr [{b} + 0x224088]",
            "fld dword ptr [{mk} + 0x158F09]",
            "fld dword ptr [{mk} + 0x158F0D]",
            "fld dword ptr [{mk} + 0x158EFD]",
            "fld dword ptr [{mk} + 0x158F01]",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xCA",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xC9",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCD",
            "fmul dword ptr [{mk} + 0x1610A5]",
            ".byte 0xD9, 0xCC",
            "fmul dword ptr [{mk} + 0x1610A9]",
            ".byte 0xD9, 0xCA",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x158E01]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x158E05]",
            ".byte 0xD9, 0xC9",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x158E0D]",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x158E11]",
            "fstp dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158E15]",
            ".byte 0xD9, 0xE8",
            "fdiv dword ptr [{b} + 0x2241C8]",
            "fld dword ptr [{mk} + 0x158E0D]",
            "fsub dword ptr [{mk} + 0x158E01]",
            "fst dword ptr [{mk} + 0x158DED]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x158E11]",
            "fsub dword ptr [{mk} + 0x158E05]",
            ".byte 0xD9, 0xE0",
            "fst dword ptr [{mk} + 0x158DE9]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x1610AD]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158DE9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158DE9]",
            "fmul dword ptr [{mk} + 0x158E15]",
            "fstp dword ptr [{mk} + 0x158DF5]",
            "fmul dword ptr [{mk} + 0x158DED]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158DED]",
            "fmul dword ptr [{mk} + 0x158E15]",
            "fstp dword ptr [{mk} + 0x158DF9]",
            "fstp dword ptr [{mk} + 0x158E21]",
            "fld dword ptr [{mk} + 0x158F11]",
            "fmul dword ptr [{mk} + 0x158E21]",
            "fst dword ptr [{b} + 0x5605A8]",
            "fst dword ptr [{b} + 0x5605C8]",
            "fst dword ptr [{b} + 0x5605E8]",
            "fstp dword ptr [{b} + 0x560608]",
            "fld dword ptr [{mk} + 0x158E01]",
            "fst dword ptr [{b} + 0x5605A0]",
            "fsub dword ptr [{mk} + 0x158DE9]",
            "fstp dword ptr [{b} + 0x5605C0]",
            "fld dword ptr [{mk} + 0x158E0D]",
            "fst dword ptr [{b} + 0x560600]",
            "fsub dword ptr [{mk} + 0x158DF5]",
            "fstp dword ptr [{b} + 0x5605E0]",
            "fld dword ptr [{mk} + 0x158E05]",
            "fst dword ptr [{b} + 0x5605A4]",
            "fsub dword ptr [{mk} + 0x158DED]",
            "fstp dword ptr [{b} + 0x5605C4]",
            "fld dword ptr [{mk} + 0x158E11]",
            "fst dword ptr [{b} + 0x560604]",
            "fsub dword ptr [{mk} + 0x158DF9]",
            "fstp dword ptr [{b} + 0x5605E4]",
            b = in(reg) b,
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_streak(place: [[u8; 10]; 3]) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = game_addr(0) as u32;
        turn(&place);
        if game_read::<i32>(DEPTH_RVA) <= 0x2_0000
            || game_read::<u32>(SWITCH_RVA) == 0
            || game_read::<i32>(FAR_RVA) <= 0x4800_0000
        {
            return Ok(());
        }
        project(b);
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
        game_write(COUNT_RVA, 4u32);
    }
    Ok(())
}
