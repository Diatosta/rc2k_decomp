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
//! Out: eax .. ecx the settings' +0x26AC .. +0x26B4, esi the settings; edi as the path
//!      leaves it
//!
//! Caller: `sub_4BC0B1` (0x4BC0BA). The x87 runs are the original's instructions over the
//! game's variables (the three values it keeps on the x87 stack across the two calls kept
//! as 80 bits); the calls their Rust versions. Refuses null settings, before anything.
//! Self-test: the original and the Rust version on settings and views of the test's, all
//! three paths; put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::adjugate3::adjugate3;
use crate::marks::setup::SETTINGS_RVA;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{Axis, extended, turn_matrix3};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::hexagons_drift", rva(0x4BCBE9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DRIFT_RVA: usize = rva(0x811BDF); // flt_811BDF ..
const FIRST_RVA: usize = rva(0x8161AE); // dword_8161AE
const SECOND_RVA: usize = rva(0x8161B2); // dword_8161B2
const KEPT_RVA: usize = rva(0x8161D6); // flt_8161D6 ..
const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC
const TURN_RVA: usize = rva(0x82E2C6); // dword_82E2C6 .. 3 x 3
const ADJUGATE_RVA: usize = rva(0x82E2F6); // unk_82E2F6
const ANGLE_RVA: usize = rva(0x82E332); // flt_82E332

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The original's 0x4BCC3D .. 0x4BCD03: the view's move (the settings' +0x26AC .. less the kept `flt_8161D6` .., times `flt_764FE0`) into `flt_8161EE` .., less the last move into `flt_8161FA` .., plus the last move times `flt_82E326` ...
unsafe fn view_moved(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0x26AC]",
            "fsub dword ptr [{b} + 0x4161D6]",
            "fmul dword ptr [{b} + 0x364FE0]",
            "fld dword ptr [{s} + 0x26B0]",
            "fsub dword ptr [{b} + 0x4161DA]",
            "fmul dword ptr [{b} + 0x364FE0]",
            "fld dword ptr [{s} + 0x26B4]",
            "fsub dword ptr [{b} + 0x4161DE]",
            "fmul dword ptr [{b} + 0x364FE0]",
            "fstp dword ptr [{b} + 0x4161F6]",
            "fstp dword ptr [{b} + 0x4161F2]",
            "fstp dword ptr [{b} + 0x4161EE]",
            "fld dword ptr [{b} + 0x4161EE]",
            "fsub dword ptr [{b} + 0x4161E2]",
            "fld dword ptr [{b} + 0x4161F2]",
            "fsub dword ptr [{b} + 0x4161E6]",
            "fld dword ptr [{b} + 0x4161F6]",
            "fsub dword ptr [{b} + 0x4161EA]",
            "fstp dword ptr [{b} + 0x416202]",
            "fstp dword ptr [{b} + 0x4161FE]",
            "fstp dword ptr [{b} + 0x4161FA]",
            "fld dword ptr [{b} + 0x4161E2]",
            "fmul dword ptr [{b} + 0x42E326]",
            "fadd dword ptr [{b} + 0x4161FA]",
            "fstp dword ptr [{b} + 0x4161FA]",
            "fld dword ptr [{b} + 0x4161E6]",
            "fmul dword ptr [{b} + 0x42E32A]",
            "fadd dword ptr [{b} + 0x4161FE]",
            "fstp dword ptr [{b} + 0x4161FE]",
            "fld dword ptr [{b} + 0x4161EA]",
            "fmul dword ptr [{b} + 0x42E32E]",
            "fadd dword ptr [{b} + 0x416202]",
            "fstp dword ptr [{b} + 0x416202]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BCD69 .. 0x4BCDCD: that move turned by the adjugate at m (`[esp]` temporaries at t), the three kept 80 bits at l.
unsafe fn drift_turned(b: u32, m: u32, t: u32, l: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x416202]",
            "fld dword ptr [{b} + 0x4161FE]",
            "fld dword ptr [{b} + 0x4161FA]",
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
            b = in(reg) b,
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
unsafe fn drift_back(b: u32, m: u32, t: u32, l: u32) {
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
            "fst dword ptr [{b} + 0x41621E]",
            "fmul dword ptr [{b} + 0x42E336]",
            "fstp dword ptr [{b} + 0x411BDF]",
            "fst dword ptr [{b} + 0x416222]",
            "fmul dword ptr [{b} + 0x42E33A]",
            "fstp dword ptr [{b} + 0x411BE3]",
            "fst dword ptr [{b} + 0x416226]",
            "fmul dword ptr [{b} + 0x42E33E]",
            "fstp dword ptr [{b} + 0x411BE7]",
            b = in(reg) b,
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
unsafe fn move_kept(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0x26AC]",
            "fsub dword ptr [{b} + 0x4161D6]",
            "fmul dword ptr [{b} + 0x364FE0]",
            "fld dword ptr [{s} + 0x26B0]",
            "fsub dword ptr [{b} + 0x4161DA]",
            "fmul dword ptr [{b} + 0x364FE0]",
            "fld dword ptr [{s} + 0x26B4]",
            "fsub dword ptr [{b} + 0x4161DE]",
            "fmul dword ptr [{b} + 0x364FE0]",
            "fstp dword ptr [{b} + 0x4161EA]",
            "fstp dword ptr [{b} + 0x4161E6]",
            "fstp dword ptr [{b} + 0x4161E2]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn hexagons_drift(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for n in 0..3 {
            game_write(DRIFT_RVA + 4 * n, 0u32);
        }
        g.esi = s;
        let b = game_addr(0) as u32;
        g.eax = game_read(FIRST_RVA);
        game_write(FIRST_RVA, 1u32);
        if g.eax != 0 {
            g.eax = game_read(SECOND_RVA);
            game_write(SECOND_RVA, 1u32);
            if g.eax != 0 {
                view_moved(b, s);
                g.edi = game_addr(VIEW_RVA) as u32;
                for col in 0..3u32 {
                    let v = [
                        get(g.edi + 4 * col),
                        get(g.edi + 0x10 + 4 * col),
                        get(g.edi + 0x20 + 4 * col),
                    ];
                    [g.eax, g.ebx, g.ecx] = v;
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
                drift_turned(b, adj, t, l);
                turn_matrix3(Axis::X, turn, extended(game_read(ANGLE_RVA)))?;
                drift_back(b, turn, t, l);
                g.edi = turn;
            }
            move_kept(b, s);
        }
        [g.eax, g.ebx, g.ecx] = [get(s + 0x26AC), get(s + 0x26B0), get(s + 0x26B4)];
        for (k, v) in [g.eax, g.ebx, g.ecx].into_iter().enumerate() {
            game_write(KEPT_RVA + 4 * k, v);
        }
    }
    Ok(g)
}

fn regions() -> Vec<Region> {
    vec![
        (game_addr(DRIFT_RVA) as u32, 12),
        at(FIRST_RVA),
        at(SECOND_RVA),
        (game_addr(KEPT_RVA) as u32, 0x50),
        (game_addr(TURN_RVA) as u32, 0x30 + 0x24),
        (game_addr(rva(0x81621E)) as u32, 12),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { hexagons_drift(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut settings = vec![0u32; 0x26B8 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([SETTINGS_RVA, ANGLE_RVA, rva(0x764FE0)].map(at));
    vars.push((game_addr(VIEW_RVA) as u32, 0x30));
    vars.push((game_addr(rva(0x82E326)) as u32, 0x24));
    let saved = capture(&vars);
    for case in 0..5u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SETTINGS_RVA, sa);
                let k = case as f32;
                for n in 0..3u32 {
                    ((sa + 0x26AC + 4 * n) as *mut u32)
                        .write_unaligned((n as f32 * 3.5 + k).to_bits());
                    game_write(KEPT_RVA + 4 * n as usize, (n as f32 * 3.0).to_bits());
                    game_write(
                        rva(0x8161E2) + 4 * n as usize,
                        (0.25 * n as f32 - k * 0.1).to_bits(),
                    );
                }
                game_write(FIRST_RVA, u32::from(case > 0));
                game_write(SECOND_RVA, u32::from(case > 1));
                game_write(ANGLE_RVA, (0.3f32 + k * 0.2).to_bits());
                game_write(rva(0x764FE0), 0.5f32.to_bits());
                for n in 0..12 {
                    game_write(
                        VIEW_RVA + 4 * n,
                        (((n * 7 + case as usize) % 11) as f32 * 0.17 - 0.8).to_bits(),
                    );
                }
                for n in 0..9 {
                    game_write(rva(0x82E326) + 4 * n, (0.1f32 + n as f32 * 0.05).to_bits());
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
