//! `marks::hexagons_spawn` (0x4BC5D9): new hexagons this frame. `dword_814724` plus 1;
//! with the x87 rounding set to chop throughout (put back at the end): a rate (the sine of
//! `flt_76923E` + `g_marksShapeTilt` times `flt_814728`, plus -(`flt_814728` times
//! `flt_811BE7`) times its cosine times `flt_82E2AA`) into `flt_81472C`; unless below
//! `flt_82E2AE`: its whole part (held to 0x80) of `marks::pick_hexagon`s, and one more when
//! the marks' random's bits 8 .. 23 are at most its fraction times `flt_82E2B2`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BC0B1` (0x4BC225). The x87 run is the original's instructions over the
//! game's variables; the picks their Rust versions (under the same rounding). Self-test:
//! the original and the Rust version on the game's items with rates by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::marks::pick_hexagon::pick_hexagon;
use crate::marks::setup::SEED_RVA;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::vector3::STATUS_RVA;

const FRAMES_RVA: Slot = Slot::at(0x814724); // dword_814724
const RATE_RVA: Slot = Slot::at(0x81472C); // flt_81472C
const SHARE_K_RVA: Slot = Slot::at(0x82E2B2); // flt_82E2B2
const MOST: i32 = 0x80;

/// The original's 0x4BC60E .. 0x4BC66B: the rate, compared (`fstsw` into `g_fpuStatus`).
unsafe fn rate(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{mk} + 0x42FE]",
            "fadd dword ptr [{mk} + 0xACC83]",
            ".byte 0xD9, 0xFE",
            "fmul dword ptr [{mk} + 0xAF7E8]",
            "fstp dword ptr [{mk} + 0xAF7EC]",
            "fld dword ptr [{mk} + 0xAF7E8]",
            "fmul dword ptr [{mk} + 0xACCA7]",
            ".byte 0xD9, 0xE0",
            "fld dword ptr [{mk} + 0x42FE]",
            "fadd dword ptr [{mk} + 0xACC83]",
            ".byte 0xD9, 0xFF",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{mk} + 0xC936A]",
            "fadd dword ptr [{mk} + 0xAF7EC]",
            "fstp dword ptr [{mk} + 0xAF7EC]",
            "fld dword ptr [{mk} + 0xAF7EC]",
            "fcomp dword ptr [{mk} + 0xC936E]",
            "fstsw word ptr [{b} + 0x31C814]",
            b = in(reg) b,
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn hexagons_spawn() {
    unsafe {
        game_write(FRAMES_RVA, game_read::<u32>(FRAMES_RVA).wrapping_add(1));
        let old = x87::set_chop();
        rate(game_addr(0) as u32);
        let status = game_read::<u32>(STATUS_RVA) & 0x700;
        game_write(STATUS_RVA, status);
        if status != 0x100 {
            let v: u32 = game_read(RATE_RVA);
            let n = x87::to_int(v).min(MOST);
            for _ in 0..n.max(0) {
                pick_hexagon();
            }
            let share = x87::fraction_times_to_int(v, game_read(SHARE_K_RVA));
            let s = game_read::<u32>(SEED_RVA)
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            game_write(SEED_RVA, s);
            if ((s >> 8) & 0xFFFF) as i32 <= share {
                pick_hexagon();
            }
        }
        x87::set_control(old);
    }
}
