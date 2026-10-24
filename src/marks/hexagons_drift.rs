//! `marks::hexagons_drift` (0x4BCBE9): the hexagons' drift from the view's moves.
//! `flt_811BDF` .. 0; unless this is the first time (`dword_8161AE`, then 1): unless the
//! second (`dword_8161B2`, then 1), the view's move (the settings' +0x26AC .. less the kept
//! place, times `flt_764FE0`) less the last move (plus the last times `flt_82E326` ..),
//! turned by the adjugate of the view's turn (`flt_5EA3CC`'s, copied by columns into
//! `dword_82E2C6`, `marks::adjugate3` into `unk_82E2F6`), the matrix made a turn about x by
//! `flt_82E332` (`scene::turn_matrix3_x`) and the move turned back by it, negated, into
//! `flt_81621E` .. and times `flt_82E336` .. into `flt_811BDF` ..; then the move kept
//! (`flt_8161E2` ..); and the place kept (`flt_8161D6` ..).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BC0B1` (0x4BC0BA). The x87 runs are the original's instructions over the
//! game's variables (the three values it keeps on the x87 stack across the two calls kept
//! as 80 bits); the calls their Rust versions. Refuses null settings, before anything.
//! Self-test: the original and the Rust version on settings and views of the test's, all
//! three paths; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::adjugate3::adjugate3;
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::scene::matrix3::{Axis, extended, turn_matrix3};

const DRIFT_RVA: Slot = Slot::at(0x811BDF); // flt_811BDF ..
const FIRST_RVA: Slot = Slot::at(0x8161AE); // dword_8161AE
const SECOND_RVA: Slot = Slot::at(0x8161B2); // dword_8161B2
const KEPT_RVA: Slot = Slot::at(0x8161D6); // flt_8161D6 ..
const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC
const TURN_RVA: Slot = Slot::at(0x82E2C6); // dword_82E2C6 .. 3 x 3
const ADJUGATE_RVA: Slot = Slot::at(0x82E2F6); // unk_82E2F6
const ANGLE_RVA: Slot = Slot::at(0x82E332); // flt_82E332

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The original's 0x4BCC3D .. 0x4BCD03: the view's move (the settings' +0x26AC .. less the kept `flt_8161D6` .., times `flt_764FE0`) into `flt_8161EE` .., less the last move into `flt_8161FA` .., plus the last move times `flt_82E326` ...
unsafe fn view_moved(s: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0x26AC]",
            "fsub dword ptr [{mk} + 0xB1296]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B0]",
            "fsub dword ptr [{mk} + 0xB129A]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B4]",
            "fsub dword ptr [{mk} + 0xB129E]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fstp dword ptr [{mk} + 0xB12B6]",
            "fstp dword ptr [{mk} + 0xB12B2]",
            "fstp dword ptr [{mk} + 0xB12AE]",
            "fld dword ptr [{mk} + 0xB12AE]",
            "fsub dword ptr [{mk} + 0xB12A2]",
            "fld dword ptr [{mk} + 0xB12B2]",
            "fsub dword ptr [{mk} + 0xB12A6]",
            "fld dword ptr [{mk} + 0xB12B6]",
            "fsub dword ptr [{mk} + 0xB12AA]",
            "fstp dword ptr [{mk} + 0xB12C2]",
            "fstp dword ptr [{mk} + 0xB12BE]",
            "fstp dword ptr [{mk} + 0xB12BA]",
            "fld dword ptr [{mk} + 0xB12A2]",
            "fmul dword ptr [{mk} + 0xC93E6]",
            "fadd dword ptr [{mk} + 0xB12BA]",
            "fstp dword ptr [{mk} + 0xB12BA]",
            "fld dword ptr [{mk} + 0xB12A6]",
            "fmul dword ptr [{mk} + 0xC93EA]",
            "fadd dword ptr [{mk} + 0xB12BE]",
            "fstp dword ptr [{mk} + 0xB12BE]",
            "fld dword ptr [{mk} + 0xB12AA]",
            "fmul dword ptr [{mk} + 0xC93EE]",
            "fadd dword ptr [{mk} + 0xB12C2]",
            "fstp dword ptr [{mk} + 0xB12C2]",
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BCD69 .. 0x4BCDCD: that move turned by the adjugate at m (`[esp]` temporaries at t), the three kept 80 bits at l.
unsafe fn drift_turned(m: u32, t: u32, l: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{mk} + 0xB12C2]",
            "fld dword ptr [{mk} + 0xB12BE]",
            "fld dword ptr [{mk} + 0xB12BA]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp tbyte ptr [{l}]",
            "fstp tbyte ptr [{l} + 10]",
            "fstp tbyte ptr [{l} + 20]",
            mk = in(reg) crate::marks::store::base(),
            m = in(reg) m,
            t = in(reg) t,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BCDE6 .. 0x4BCE6A: the three (from l) turned back by the matrix at m, negated, into `flt_81621E` .. and times `flt_82E336` .. into `flt_811BDF` ...
unsafe fn drift_back(m: u32, t: u32, l: u32) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{l} + 20]",
            "fld tbyte ptr [{l} + 10]",
            "fld tbyte ptr [{l}]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xE0",
            "fst dword ptr [{mk} + 0xB12DE]",
            "fmul dword ptr [{mk} + 0xC93F6]",
            "fstp dword ptr [{mk} + 0xACC9F]",
            "fst dword ptr [{mk} + 0xB12E2]",
            "fmul dword ptr [{mk} + 0xC93FA]",
            "fstp dword ptr [{mk} + 0xACCA3]",
            "fst dword ptr [{mk} + 0xB12E6]",
            "fmul dword ptr [{mk} + 0xC93FE]",
            "fstp dword ptr [{mk} + 0xACCA7]",
            mk = in(reg) crate::marks::store::base(),
            m = in(reg) m,
            t = in(reg) t,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BCE6A .. 0x4BCEB2: the view's move kept in `flt_8161E2` ...
unsafe fn move_kept(s: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0x26AC]",
            "fsub dword ptr [{mk} + 0xB1296]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B0]",
            "fsub dword ptr [{mk} + 0xB129A]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B4]",
            "fsub dword ptr [{mk} + 0xB129E]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fstp dword ptr [{mk} + 0xB12AA]",
            "fstp dword ptr [{mk} + 0xB12A6]",
            "fstp dword ptr [{mk} + 0xB12A2]",
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn hexagons_drift() -> Result<(), Refused> {
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for n in 0..3 {
            game_write(DRIFT_RVA + 4 * n, 0u32);
        }
        let first: u32 = game_read(FIRST_RVA);
        game_write(FIRST_RVA, 1u32);
        if first != 0 {
            let second: u32 = game_read(SECOND_RVA);
            game_write(SECOND_RVA, 1u32);
            if second != 0 {
                view_moved(s);
                let view = game_addr(VIEW_RVA) as u32;
                for col in 0..3u32 {
                    let v = [
                        get(view + 4 * col),
                        get(view + 0x10 + 4 * col),
                        get(view + 0x20 + 4 * col),
                    ];
                    for (k, x) in v.into_iter().enumerate() {
                        game_write(TURN_RVA + 12 * col as usize + 4 * k, x);
                    }
                }
                let turn = game_addr(TURN_RVA) as u32;
                let adj = game_addr(ADJUGATE_RVA) as u32;
                adjugate3(turn, adj)?;
                let mut temps = [0u32; 2];
                let mut kept = [0u8; 30];
                let (t, l) = (temps.as_mut_ptr() as u32, kept.as_mut_ptr() as u32);
                drift_turned(adj, t, l);
                turn_matrix3(Axis::X, turn, extended(game_read(ANGLE_RVA)))?;
                drift_back(turn, t, l);
            }
            move_kept(s);
        }
        for (k, v) in [get(s + 0x26AC), get(s + 0x26B0), get(s + 0x26B4)]
            .into_iter()
            .enumerate()
        {
            game_write(KEPT_RVA + 4 * k, v);
        }
    }
    Ok(())
}
