//! `marks::hexagons_spawn` (0x4BC5D9): new hexagons this frame. `dword_814724` plus 1;
//! with the x87 rounding set to chop throughout (put back at the end): a rate (the sine of
//! `flt_76923E` + `g_marksShapeTilt` times `flt_814728`, plus -(`flt_814728` times
//! `flt_811BE7`) times its cosine times `flt_82E2AA`) into `flt_81472C`; unless below
//! `flt_82E2AE`: its whole part (held to 0x80) of `marks::pick_hexagon`s, and one more when
//! the marks' random's bits 8 .. 23 are at most its fraction times `flt_82E2B2`.
//!
//! In:  nothing
//! Out: eax the fraction's share, ebx the random's bits (and the picks' registers), when
//!      the rate is high enough
//!
//! Caller: `sub_4BC0B1` (0x4BC225). The x87 run is the original's instructions over the
//! game's variables; the picks their Rust versions (under the same rounding). Self-test:
//! the original and the Rust version on the game's items with rates by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::pick_hexagon::pick_hexagon;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::vector3::STATUS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::hexagons_spawn", rva(0x4BC5D9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FRAMES_RVA: usize = rva(0x814724); // dword_814724
const RATE_RVA: usize = rva(0x81472C); // flt_81472C
const SHARE_K_RVA: usize = rva(0x82E2B2); // flt_82E2B2
const MOST: i32 = 0x80;

/// The original's 0x4BC60E .. 0x4BC66B: the rate, compared (`fstsw` into `g_fpuStatus`).
unsafe fn rate(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x36923E]",
            "fadd dword ptr [{b} + 0x411BC3]",
            ".byte 0xD9, 0xFE",
            "fmul dword ptr [{b} + 0x414728]",
            "fstp dword ptr [{b} + 0x41472C]",
            "fld dword ptr [{b} + 0x414728]",
            "fmul dword ptr [{b} + 0x411BE7]",
            ".byte 0xD9, 0xE0",
            "fld dword ptr [{b} + 0x36923E]",
            "fadd dword ptr [{b} + 0x411BC3]",
            ".byte 0xD9, 0xFF",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{b} + 0x42E2AA]",
            "fadd dword ptr [{b} + 0x41472C]",
            "fstp dword ptr [{b} + 0x41472C]",
            "fld dword ptr [{b} + 0x41472C]",
            "fcomp dword ptr [{b} + 0x42E2AE]",
            "fstsw word ptr [{b} + 0x31C814]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn hexagons_spawn(entry: &Regs) -> Regs {
    let mut g = *entry;
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
                g = pick_hexagon(&g);
            }
            g.eax = x87::fraction_times_to_int(v, game_read(SHARE_K_RVA)) as u32;
            let s = game_read::<u32>(SEED_RVA)
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            game_write(SEED_RVA, s);
            g.ebx = (s >> 8) & 0xFFFF;
            if g.ebx as i32 <= g.eax as i32 {
                g = pick_hexagon(&g);
            }
        }
        x87::set_control(old);
    }
    g
}

fn regions() -> Vec<Region> {
    let mut r = crate::marks::spawn_hexagon::regions();
    r.extend([FRAMES_RVA, RATE_RVA, SEED_RVA, STATUS_RVA].map(at));
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { hexagons_spawn(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend(
        [
            rva(0x76923E),
            rva(0x811BC3),
            rva(0x814728),
            rva(0x811BE7),
            rva(0x82E2AA),
            rva(0x82E2AE),
            SHARE_K_RVA,
        ]
        .map(at),
    );
    let saved = capture(&vars);
    for case in 0..6u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                let k = case as f32;
                game_write(rva(0x76923E), (0.4f32 + k * 0.3).to_bits());
                game_write(rva(0x811BC3), 0.1f32.to_bits());
                game_write(rva(0x814728), (2.5f32 + k * 40.0).to_bits());
                game_write(rva(0x811BE7), (0.2f32 - k * 0.05).to_bits());
                game_write(rva(0x82E2AA), 1.5f32.to_bits());
                game_write(
                    rva(0x82E2AE),
                    if case == 5 { 1e6f32 } else { 0.5 }.to_bits(),
                );
                game_write(SHARE_K_RVA, 65535.0f32.to_bits());
                game_write(SEED_RVA, case.wrapping_mul(0x2545_F491));
                game_write(crate::marks::spawn_hexagon::CURSOR_RVA, case * 0x30 * 11);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
