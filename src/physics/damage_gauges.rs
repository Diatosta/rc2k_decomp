//! `physics::damage_gauges` (0x4AC817): the damage gauges' step (`physics/damage_model.rs`):
//! each gauge (`unk_71A284` .. `unk_71A33C`, the gearbox's eight `unk_71A2AC[gear]` by
//! `dword_75B3BC`) raised by its rate (`flt_75B5D0` ..) times this step's shares and hit
//! figures (`flt_75B38C`, `flt_75B370`, `flt_75B398`, `flt_75B3B4`, `flt_75B450` ..), each
//! held 0 ..= `flt_75B34C` after (`fcompp`, below or unordered raised); the hit's side
//! (`dword_75B3C4` 1 .. 4) choosing gauges, its size (`flt_75B3C8`) scaled by
//! `flt_71C708` past `flt_75B628`, the direction `dword_75B3B8` (1 / -1) adding fixed
//! steps; the kind tables `flt_75B630` / `flt_75B638` by `flt_75B3E4`'s bits.
//!
//! In:  nothing
//! Out: esi and ebx the gear (where the original loads them), the rest as the steps leave
//!      them; no flags read
//!
//! Caller: `physics::damage_step`.
//! Self-test: `physics/damage_model.rs` (`damage_test`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
#[allow(unused_imports)] // the generated body uses some of them
use crate::math::x87::{loaded, mul};
#[allow(unused_imports)]
use crate::math::x87_run::{abs_compare, compare_full};
use crate::physics::damage_model::{damage_test, regions};
#[allow(unused_imports)]
use crate::physics::flow::{a, base, c0, g, hold_high, hold_low, p, table_lerp, x87_run};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::damage_gauges", rva(0x4AC817), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `unk_71A2AC[gear]` (the gear -1 .. 7 as the original indexes it).
fn gear_gauge(gear: u32) -> u32 {
    a(0x71A2AC).wrapping_add(gear.wrapping_mul(4))
}

/// The gauge at `at` held 0 ..= `flt_75B34C` (the two `fcompp` holds).
fn hold_at(at: u32) {
    let rd = || unsafe { (at as *const u32).read_unaligned() };
    let put = |v: u32| unsafe { (at as *mut u32).write_unaligned(v) };
    if c0(rd(), g(0x75B350)) {
        put(loaded(g(0x75B350)));
    }
    if !c0(rd(), g(0x75B34C)) {
        put(loaded(g(0x75B34C)));
    }
}

/// The steps (generated from the disassembly by idiom; the indexed gauges by hand).
pub fn steps(r: &mut Regs) {
    x87_run!(
        "fld dword ptr [{b} + 0x35B5D0]", // fld     flt_75B5D0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A290]", // fadd    dword ptr unk_71A290
        "fstp dword ptr [{b} + 0x31A290]"; // fstp    dword ptr unk_71A290
        b = base()
    );
    hold_low(0x71A290, 0x75B350);
    hold_high(0x71A290, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5D4]", // fld     flt_75B5D4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A294]", // fadd    dword ptr unk_71A294
        "fstp dword ptr [{b} + 0x31A294]"; // fstp    dword ptr unk_71A294
        b = base()
    );
    hold_low(0x71A294, 0x75B350);
    hold_high(0x71A294, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5D8]", // fld     flt_75B5D8
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A29C]", // fadd    dword ptr unk_71A29C
        "fstp dword ptr [{b} + 0x31A29C]"; // fstp    dword ptr unk_71A29C
        b = base()
    );
    hold_low(0x71A29C, 0x75B350);
    hold_high(0x71A29C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5DC]", // fld     flt_75B5DC
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2A0]", // fadd    dword ptr unk_71A2A0
        "fstp dword ptr [{b} + 0x31A2A0]"; // fstp    dword ptr unk_71A2A0
        b = base()
    );
    hold_low(0x71A2A0, 0x75B350);
    hold_high(0x71A2A0, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E0]", // fld     flt_75B5E0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2A4]", // fadd    dword ptr unk_71A2A4
        "fstp dword ptr [{b} + 0x31A2A4]"; // fstp    dword ptr unk_71A2A4
        b = base()
    );
    hold_low(0x71A2A4, 0x75B350);
    hold_high(0x71A2A4, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E4]", // fld     flt_75B5E4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2E8]", // fadd    dword ptr unk_71A2E8
        "fstp dword ptr [{b} + 0x31A2E8]"; // fstp    dword ptr unk_71A2E8
        b = base()
    );
    hold_low(0x71A2E8, 0x75B350);
    hold_high(0x71A2E8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E8]", // fld     flt_75B5E8
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2EC]", // fadd    dword ptr unk_71A2EC
        "fstp dword ptr [{b} + 0x31A2EC]"; // fstp    dword ptr unk_71A2EC
        b = base()
    );
    hold_low(0x71A2EC, 0x75B350);
    hold_high(0x71A2EC, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E8]", // fld     flt_75B5E8
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2F0]", // fadd    dword ptr unk_71A2F0
        "fstp dword ptr [{b} + 0x31A2F0]"; // fstp    dword ptr unk_71A2F0
        b = base()
    );
    hold_low(0x71A2F0, 0x75B350);
    hold_high(0x71A2F0, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E8]", // fld     flt_75B5E8
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2F4]", // fadd    dword ptr unk_71A2F4
        "fstp dword ptr [{b} + 0x31A2F4]"; // fstp    dword ptr unk_71A2F4
        b = base()
    );
    hold_low(0x71A2F4, 0x75B350);
    hold_high(0x71A2F4, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E8]", // fld     flt_75B5E8
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2F8]", // fadd    dword ptr unk_71A2F8
        "fstp dword ptr [{b} + 0x31A2F8]"; // fstp    dword ptr unk_71A2F8
        b = base()
    );
    hold_low(0x71A2F8, 0x75B350);
    hold_high(0x71A2F8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E8]", // fld     flt_75B5E8
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2FC]", // fadd    dword ptr unk_71A2FC
        "fstp dword ptr [{b} + 0x31A2FC]"; // fstp    dword ptr unk_71A2FC
        b = base()
    );
    hold_low(0x71A2FC, 0x75B350);
    hold_high(0x71A2FC, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5E8]", // fld     flt_75B5E8
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A300]", // fadd    dword ptr unk_71A300
        "fstp dword ptr [{b} + 0x31A300]"; // fstp    dword ptr unk_71A300
        b = base()
    );
    hold_low(0x71A300, 0x75B350);
    hold_high(0x71A300, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5EC]", // fld     flt_75B5EC
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A308]", // fadd    dword ptr unk_71A308
        "fstp dword ptr [{b} + 0x31A308]"; // fstp    dword ptr unk_71A308
        b = base()
    );
    hold_low(0x71A308, 0x75B350);
    hold_high(0x71A308, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5F0]", // fld     flt_75B5F0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A30C]", // fadd    dword ptr unk_71A30C
        "fstp dword ptr [{b} + 0x31A30C]"; // fstp    dword ptr unk_71A30C
        b = base()
    );
    hold_low(0x71A30C, 0x75B350);
    hold_high(0x71A30C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5F0]", // fld     flt_75B5F0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A310]", // fadd    dword ptr unk_71A310
        "fstp dword ptr [{b} + 0x31A310]"; // fstp    dword ptr unk_71A310
        b = base()
    );
    hold_low(0x71A310, 0x75B350);
    hold_high(0x71A310, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5F0]", // fld     flt_75B5F0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A314]", // fadd    dword ptr unk_71A314
        "fstp dword ptr [{b} + 0x31A314]"; // fstp    dword ptr unk_71A314
        b = base()
    );
    hold_low(0x71A314, 0x75B350);
    hold_high(0x71A314, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5F0]", // fld     flt_75B5F0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A318]", // fadd    dword ptr unk_71A318
        "fstp dword ptr [{b} + 0x31A318]"; // fstp    dword ptr unk_71A318
        b = base()
    );
    hold_low(0x71A318, 0x75B350);
    hold_high(0x71A318, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5A4]", // fld     flt_75B5A4
        "fmul dword ptr [{b} + 0x35B3C0]", // fmul    flt_75B3C0
        "fmul dword ptr [{b} + 0x35B398]", // fmul    flt_75B398
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A298]", // fadd    dword ptr unk_71A298
        "fstp dword ptr [{b} + 0x31A298]"; // fstp    dword ptr unk_71A298
        b = base()
    );
    hold_low(0x71A298, 0x75B350);
    hold_high(0x71A298, 0x75B34C);
    r.esi = g(0x75B3BC);
    let gauge = gear_gauge(r.esi);
    x87_run!(
        "fld dword ptr [{b} + 0x35B398]",
        "fmul dword ptr [{b} + 0x35B5A8]",
        "fmul dword ptr [{b} + 0x35B370]",
        "fmul dword ptr [{t}]",
        "fadd dword ptr [{v}]",
        "fstp dword ptr [{v}]";
        b = base(), t = a(0x75B5B0).wrapping_add(r.esi.wrapping_mul(4)), v = gauge
    );
    hold_at(gauge);
    x87_run!(
        ".byte 0xD9, 0xE8", // fld1
        "fsub dword ptr [{b} + 0x35B400]", // fsub    flt_75B400
        "fstp dword ptr [{b} + 0x35B43C]", // fstp    flt_75B43C
        "fld dword ptr [{b} + 0x35B5F4]", // fld     flt_75B5F4
        "fmul dword ptr [{b} + 0x35B400]", // fmul    flt_75B400
        "fmul dword ptr [{b} + 0x35B3B0]", // fmul    flt_75B3B0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A320]", // fadd    dword ptr unk_71A320
        "fstp dword ptr [{b} + 0x31A320]"; // fstp    dword ptr unk_71A320
        b = base()
    );
    hold_low(0x71A320, 0x75B350);
    hold_high(0x71A320, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5F4]", // fld     flt_75B5F4
        "fmul dword ptr [{b} + 0x35B43C]", // fmul    flt_75B43C
        "fmul dword ptr [{b} + 0x35B3B0]", // fmul    flt_75B3B0
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A324]", // fadd    dword ptr unk_71A324
        "fstp dword ptr [{b} + 0x31A324]"; // fstp    dword ptr unk_71A324
        b = base()
    );
    hold_low(0x71A324, 0x75B350);
    hold_high(0x71A324, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5F8]", // fld     flt_75B5F8
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2E8]", // fadd    dword ptr unk_71A2E8
        "fstp dword ptr [{b} + 0x31A2E8]"; // fstp    dword ptr unk_71A2E8
        b = base()
    );
    hold_low(0x71A2E8, 0x75B350);
    hold_high(0x71A2E8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5FC]", // fld     flt_75B5FC
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2EC]", // fadd    dword ptr unk_71A2EC
        "fstp dword ptr [{b} + 0x31A2EC]"; // fstp    dword ptr unk_71A2EC
        b = base()
    );
    hold_low(0x71A2EC, 0x75B350);
    hold_high(0x71A2EC, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5FC]", // fld     flt_75B5FC
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2F0]", // fadd    dword ptr unk_71A2F0
        "fstp dword ptr [{b} + 0x31A2F0]"; // fstp    dword ptr unk_71A2F0
        b = base()
    );
    hold_low(0x71A2F0, 0x75B350);
    hold_high(0x71A2F0, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5FC]", // fld     flt_75B5FC
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2F4]", // fadd    dword ptr unk_71A2F4
        "fstp dword ptr [{b} + 0x31A2F4]"; // fstp    dword ptr unk_71A2F4
        b = base()
    );
    hold_low(0x71A2F4, 0x75B350);
    hold_high(0x71A2F4, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5FC]", // fld     flt_75B5FC
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2F8]", // fadd    dword ptr unk_71A2F8
        "fstp dword ptr [{b} + 0x31A2F8]"; // fstp    dword ptr unk_71A2F8
        b = base()
    );
    hold_low(0x71A2F8, 0x75B350);
    hold_high(0x71A2F8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5FC]", // fld     flt_75B5FC
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A2FC]", // fadd    dword ptr unk_71A2FC
        "fstp dword ptr [{b} + 0x31A2FC]"; // fstp    dword ptr unk_71A2FC
        b = base()
    );
    hold_low(0x71A2FC, 0x75B350);
    hold_high(0x71A2FC, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B5FC]", // fld     flt_75B5FC
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B38C]", // fmul    flt_75B38C
        "fadd dword ptr [{b} + 0x31A300]", // fadd    dword ptr unk_71A300
        "fstp dword ptr [{b} + 0x31A300]"; // fstp    dword ptr unk_71A300
        b = base()
    );
    hold_low(0x71A300, 0x75B350);
    hold_high(0x71A300, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B39C]", // fld     flt_75B39C
        "fld dword ptr [{b} + 0x35B484]", // fld     flt_75B484
        "fmul dword ptr [{b} + 0x35B380]", // fmul    flt_75B380
        ".byte 0xDE, 0xF9", // fdivp   st(1), st
        "fmul dword ptr [{b} + 0x35B34C]", // fmul    flt_75B34C
        "fmul dword ptr [{b} + 0x35B3F4]", // fmul    flt_75B3F4
        "fst dword ptr [{b} + 0x35B488]", // fst     flt_75B488
        "fstp dword ptr [{b} + 0x35B48C]"; // fstp    flt_75B48C
        b = base()
    );
    r.ebx = g(0x75B3E4);
    x87_run!(
        "fld dword ptr [{b} + 0x35B488]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{b} + 0x35B488]",
        "fld dword ptr [{b} + 0x35B48C]",
        "fmul dword ptr [{u}]",
        "fstp dword ptr [{b} + 0x35B48C]";
        b = base(), t = a(0x75B630).wrapping_add(r.ebx.wrapping_mul(4)),
        u = a(0x75B638).wrapping_add(r.ebx.wrapping_mul(4))
    );
    x87_run!(
                "fld dword ptr [{b} + 0x35B46C]", // fld     flt_75B46C
        "fmul dword ptr [{b} + 0x35B488]", // fmul    flt_75B488
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A32C]", // fadd    dword ptr unk_71A32C
        "fstp dword ptr [{b} + 0x31A32C]"; // fstp    dword ptr unk_71A32C
        b = base()
    );
    hold_low(0x71A32C, 0x75B350);
    hold_high(0x71A32C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B46C]", // fld     flt_75B46C
        "fmul dword ptr [{b} + 0x35B488]", // fmul    flt_75B488
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A330]", // fadd    dword ptr unk_71A330
        "fstp dword ptr [{b} + 0x31A330]"; // fstp    dword ptr unk_71A330
        b = base()
    );
    hold_low(0x71A330, 0x75B350);
    hold_high(0x71A330, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B470]", // fld     flt_75B470
        "fmul dword ptr [{b} + 0x35B48C]", // fmul    flt_75B48C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A334]", // fadd    dword ptr unk_71A334
        "fstp dword ptr [{b} + 0x31A334]"; // fstp    dword ptr unk_71A334
        b = base()
    );
    hold_low(0x71A334, 0x75B350);
    hold_high(0x71A334, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B470]", // fld     flt_75B470
        "fmul dword ptr [{b} + 0x35B48C]", // fmul    flt_75B48C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A338]", // fadd    dword ptr unk_71A338
        "fstp dword ptr [{b} + 0x31A338]"; // fstp    dword ptr unk_71A338
        b = base()
    );
    hold_low(0x71A338, 0x75B350);
    hold_high(0x71A338, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B45C]", // fld     flt_75B45C
        "fmul dword ptr [{b} + 0x35B604]", // fmul    flt_75B604
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A290]", // fadd    dword ptr unk_71A290
        "fstp dword ptr [{b} + 0x31A290]"; // fstp    dword ptr unk_71A290
        b = base()
    );
    hold_low(0x71A290, 0x75B350);
    hold_high(0x71A290, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B45C]", // fld     flt_75B45C
        "fmul dword ptr [{b} + 0x35B600]", // fmul    flt_75B600
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A29C]", // fadd    dword ptr unk_71A29C
        "fstp dword ptr [{b} + 0x31A29C]"; // fstp    dword ptr unk_71A29C
        b = base()
    );
    hold_low(0x71A29C, 0x75B350);
    hold_high(0x71A29C, 0x75B34C);
    if compare_full(g(0x75B464), g(0x75B350)) & 0x100 == 0 {
        x87_run!(
            "fld dword ptr [{b} + 0x31A28C]", // fld     dword ptr unk_71A28C
            "fmul dword ptr [{b} + 0x35B608]", // fmul    flt_75B608
            "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
            "fadd dword ptr [{b} + 0x31A290]", // fadd    dword ptr unk_71A290
            "fstp dword ptr [{b} + 0x31A290]"; // fstp    dword ptr unk_71A290
            b = base()
        );
        hold_low(0x71A290, 0x75B350);
        hold_high(0x71A290, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B464]", // fld     flt_75B464
            "fmul dword ptr [{b} + 0x35B60C]", // fmul    flt_75B60C
            "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
            "fadd dword ptr [{b} + 0x31A29C]", // fadd    dword ptr unk_71A29C
            "fstp dword ptr [{b} + 0x31A29C]"; // fstp    dword ptr unk_71A29C
            b = base()
        );
        hold_low(0x71A29C, 0x75B350);
        hold_high(0x71A29C, 0x75B34C);
    }
    if g(0x75B3B8) == 0x1 {
        x87_run!(
            "fld dword ptr [{b} + 0x31A2A0]", // fld     dword ptr unk_71A2A0
            "fadd dword ptr [{b} + 0x35B610]", // fadd    flt_75B610
            "fstp dword ptr [{b} + 0x31A2A0]"; // fstp    dword ptr unk_71A2A0
            b = base()
        );
    }
    if g(0x75B3B8) == 0xFFFFFFFF {
        x87_run!(
            "fld dword ptr [{b} + 0x31A2A0]", // fld     dword ptr unk_71A2A0
            "fadd dword ptr [{b} + 0x35B614]", // fadd    flt_75B614
            "fstp dword ptr [{b} + 0x31A2A0]"; // fstp    dword ptr unk_71A2A0
            b = base()
        );
    }
    r.esi = g(0x75B3BC);
    if g(0x75B3B8) == 0x1 {
        let gauge = gear_gauge(r.esi);
        x87_run!(
            "fld dword ptr [{v}]",
            "fadd dword ptr [{b} + 0x35B618]",
            "fstp dword ptr [{v}]";
            b = base(), v = gauge
        );
    }
    r.esi = g(0x75B3BC);
    if g(0x75B3B8) == 0xFFFFFFFF {
        let gauge = gear_gauge(r.esi);
        x87_run!(
            "fld dword ptr [{v}]",
            "fadd dword ptr [{b} + 0x35B61C]",
            "fstp dword ptr [{v}]";
            b = base(), v = gauge
        );
    }
    x87_run!(
        "fld dword ptr [{b} + 0x35B450]", // fld     flt_75B450
        "fmul dword ptr [{b} + 0x35B670]", // fmul    flt_75B670
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2D0]", // fadd    dword ptr unk_71A2D0
        "fstp dword ptr [{b} + 0x31A2D0]"; // fstp    dword ptr unk_71A2D0
        b = base()
    );
    hold_low(0x71A2D0, 0x75B350);
    hold_high(0x71A2D0, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B450]", // fld     flt_75B450
        "fmul dword ptr [{b} + 0x35B670]", // fmul    flt_75B670
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2D4]", // fadd    dword ptr unk_71A2D4
        "fstp dword ptr [{b} + 0x31A2D4]"; // fstp    dword ptr unk_71A2D4
        b = base()
    );
    hold_low(0x71A2D4, 0x75B350);
    hold_high(0x71A2D4, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B450]", // fld     flt_75B450
        "fmul dword ptr [{b} + 0x35B670]", // fmul    flt_75B670
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2D8]", // fadd    dword ptr unk_71A2D8
        "fstp dword ptr [{b} + 0x31A2D8]"; // fstp    dword ptr unk_71A2D8
        b = base()
    );
    hold_low(0x71A2D8, 0x75B350);
    hold_high(0x71A2D8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B450]", // fld     flt_75B450
        "fmul dword ptr [{b} + 0x35B670]", // fmul    flt_75B670
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2DC]", // fadd    dword ptr unk_71A2DC
        "fstp dword ptr [{b} + 0x31A2DC]"; // fstp    dword ptr unk_71A2DC
        b = base()
    );
    hold_low(0x71A2DC, 0x75B350);
    hold_high(0x71A2DC, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B458]", // fld     flt_75B458
        "fmul dword ptr [{b} + 0x35B678]", // fmul    flt_75B678
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A30C]", // fadd    dword ptr unk_71A30C
        "fstp dword ptr [{b} + 0x31A30C]"; // fstp    dword ptr unk_71A30C
        b = base()
    );
    hold_low(0x71A30C, 0x75B350);
    hold_high(0x71A30C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B458]", // fld     flt_75B458
        "fmul dword ptr [{b} + 0x35B678]", // fmul    flt_75B678
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A310]", // fadd    dword ptr unk_71A310
        "fstp dword ptr [{b} + 0x31A310]"; // fstp    dword ptr unk_71A310
        b = base()
    );
    hold_low(0x71A310, 0x75B350);
    hold_high(0x71A310, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B458]", // fld     flt_75B458
        "fmul dword ptr [{b} + 0x35B678]", // fmul    flt_75B678
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A314]", // fadd    dword ptr unk_71A314
        "fstp dword ptr [{b} + 0x31A314]"; // fstp    dword ptr unk_71A314
        b = base()
    );
    hold_low(0x71A314, 0x75B350);
    hold_high(0x71A314, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B458]", // fld     flt_75B458
        "fmul dword ptr [{b} + 0x35B678]", // fmul    flt_75B678
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A318]", // fadd    dword ptr unk_71A318
        "fstp dword ptr [{b} + 0x31A318]"; // fstp    dword ptr unk_71A318
        b = base()
    );
    hold_low(0x71A318, 0x75B350);
    hold_high(0x71A318, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B67C]", // fmul    flt_75B67C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
        "fstp dword ptr [{b} + 0x31A2E0]"; // fstp    dword ptr unk_71A2E0
        b = base()
    );
    hold_low(0x71A2E0, 0x75B350);
    hold_high(0x71A2E0, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B680]", // fmul    flt_75B680
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A308]", // fadd    dword ptr unk_71A308
        "fstp dword ptr [{b} + 0x31A308]"; // fstp    dword ptr unk_71A308
        b = base()
    );
    hold_low(0x71A308, 0x75B350);
    hold_high(0x71A308, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B684]", // fmul    flt_75B684
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A290]", // fadd    dword ptr unk_71A290
        "fstp dword ptr [{b} + 0x31A290]"; // fstp    dword ptr unk_71A290
        b = base()
    );
    hold_low(0x71A290, 0x75B350);
    hold_high(0x71A290, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B688]", // fmul    flt_75B688
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A294]", // fadd    dword ptr unk_71A294
        "fstp dword ptr [{b} + 0x31A294]"; // fstp    dword ptr unk_71A294
        b = base()
    );
    hold_low(0x71A294, 0x75B350);
    hold_high(0x71A294, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2A8]", // fadd    dword ptr unk_71A2A8
        "fstp dword ptr [{b} + 0x31A2A8]"; // fstp    dword ptr unk_71A2A8
        b = base()
    );
    hold_low(0x71A2A8, 0x75B350);
    hold_high(0x71A2A8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2B0]", // fadd    dword ptr unk_71A2B0
        "fstp dword ptr [{b} + 0x31A2B0]"; // fstp    dword ptr unk_71A2B0
        b = base()
    );
    hold_low(0x71A2B0, 0x75B350);
    hold_high(0x71A2B0, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2B4]", // fadd    dword ptr unk_71A2B4
        "fstp dword ptr [{b} + 0x31A2B4]"; // fstp    dword ptr unk_71A2B4
        b = base()
    );
    hold_low(0x71A2B4, 0x75B350);
    hold_high(0x71A2B4, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2B8]", // fadd    dword ptr unk_71A2B8
        "fstp dword ptr [{b} + 0x31A2B8]"; // fstp    dword ptr unk_71A2B8
        b = base()
    );
    hold_low(0x71A2B8, 0x75B350);
    hold_high(0x71A2B8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2BC]", // fadd    dword ptr unk_71A2BC
        "fstp dword ptr [{b} + 0x31A2BC]"; // fstp    dword ptr unk_71A2BC
        b = base()
    );
    hold_low(0x71A2BC, 0x75B350);
    hold_high(0x71A2BC, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2C0]", // fadd    dword ptr unk_71A2C0
        "fstp dword ptr [{b} + 0x31A2C0]"; // fstp    dword ptr unk_71A2C0
        b = base()
    );
    hold_low(0x71A2C0, 0x75B350);
    hold_high(0x71A2C0, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2C4]", // fadd    dword ptr unk_71A2C4
        "fstp dword ptr [{b} + 0x31A2C4]"; // fstp    dword ptr unk_71A2C4
        b = base()
    );
    hold_low(0x71A2C4, 0x75B350);
    hold_high(0x71A2C4, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B68C]", // fmul    flt_75B68C
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2C8]", // fadd    dword ptr unk_71A2C8
        "fstp dword ptr [{b} + 0x31A2C8]"; // fstp    dword ptr unk_71A2C8
        b = base()
    );
    hold_low(0x71A2C8, 0x75B350);
    hold_high(0x71A2C8, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B454]", // fld     flt_75B454
        "fmul dword ptr [{b} + 0x35B690]", // fmul    flt_75B690
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A2A4]", // fadd    dword ptr unk_71A2A4
        "fstp dword ptr [{b} + 0x31A2A4]"; // fstp    dword ptr unk_71A2A4
        b = base()
    );
    hold_low(0x71A2A4, 0x75B350);
    hold_high(0x71A2A4, 0x75B34C);
    if compare_full(g(0x75B3C8), g(0x75B62C)) & 0x100 == 0 {
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6AC]", // fmul    flt_75B6AC
            "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
            "fstp dword ptr [{b} + 0x31A2E0]"; // fstp    dword ptr unk_71A2E0
            b = base()
        );
        hold_low(0x71A2E0, 0x75B350);
        hold_high(0x71A2E0, 0x75B34C);
    }
    if compare_full(g(0x75B3C8), g(0x75B628)) & 0x100 == 0 {
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x31C708]", // fmul    flt_71C708
            "fstp dword ptr [{b} + 0x35B3C8]"; // fstp    flt_75B3C8
            b = base()
        );
        if g(0x75B3C4) == 0x1 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6D8]", // fmul    flt_75B6D8
                "fadd dword ptr [{b} + 0x31A2EC]", // fadd    dword ptr unk_71A2EC
                "fstp dword ptr [{b} + 0x31A2EC]"; // fstp    dword ptr unk_71A2EC
                b = base()
            );
            hold_low(0x71A2EC, 0x75B350);
            hold_high(0x71A2EC, 0x75B34C);
        }
        if g(0x75B3C4) == 0x2 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6D8]", // fmul    flt_75B6D8
                "fadd dword ptr [{b} + 0x31A2F0]", // fadd    dword ptr unk_71A2F0
                "fstp dword ptr [{b} + 0x31A2F0]"; // fstp    dword ptr unk_71A2F0
                b = base()
            );
            hold_low(0x71A2F0, 0x75B350);
            hold_high(0x71A2F0, 0x75B34C);
        }
        if g(0x75B3C4) == 0x3 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6D8]", // fmul    flt_75B6D8
                "fadd dword ptr [{b} + 0x31A2F4]", // fadd    dword ptr unk_71A2F4
                "fstp dword ptr [{b} + 0x31A2F4]"; // fstp    dword ptr unk_71A2F4
                b = base()
            );
            hold_low(0x71A2F4, 0x75B350);
            hold_high(0x71A2F4, 0x75B34C);
        }
        if g(0x75B3C4) == 0x4 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6D8]", // fmul    flt_75B6D8
                "fadd dword ptr [{b} + 0x31A2F8]", // fadd    dword ptr unk_71A2F8
                "fstp dword ptr [{b} + 0x31A2F8]"; // fstp    dword ptr unk_71A2F8
                b = base()
            );
            hold_low(0x71A2F8, 0x75B350);
            hold_high(0x71A2F8, 0x75B34C);
        }
        if g(0x75B3C4) == 0x1 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6DC]", // fmul    flt_75B6DC
                "fadd dword ptr [{b} + 0x31A2FC]", // fadd    dword ptr unk_71A2FC
                "fstp dword ptr [{b} + 0x31A2FC]"; // fstp    dword ptr unk_71A2FC
                b = base()
            );
            hold_low(0x71A2FC, 0x75B350);
            hold_high(0x71A2FC, 0x75B34C);
        }
        if g(0x75B3C4) == 0x2 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6DC]", // fmul    flt_75B6DC
                "fadd dword ptr [{b} + 0x31A2FC]", // fadd    dword ptr unk_71A2FC
                "fstp dword ptr [{b} + 0x31A2FC]"; // fstp    dword ptr unk_71A2FC
                b = base()
            );
            hold_low(0x71A2FC, 0x75B350);
            hold_high(0x71A2FC, 0x75B34C);
        }
        if g(0x75B3C4) == 0x3 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6DC]", // fmul    flt_75B6DC
                "fadd dword ptr [{b} + 0x31A300]", // fadd    dword ptr unk_71A300
                "fstp dword ptr [{b} + 0x31A300]"; // fstp    dword ptr unk_71A300
                b = base()
            );
            hold_low(0x71A300, 0x75B350);
            hold_high(0x71A300, 0x75B34C);
        }
        if g(0x75B3C4) == 0x4 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6DC]", // fmul    flt_75B6DC
                "fadd dword ptr [{b} + 0x31A300]", // fadd    dword ptr unk_71A300
                "fstp dword ptr [{b} + 0x31A300]"; // fstp    dword ptr unk_71A300
                b = base()
            );
            hold_low(0x71A300, 0x75B350);
            hold_high(0x71A300, 0x75B34C);
        }
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6D4]", // fmul    flt_75B6D4
            "fadd dword ptr [{b} + 0x31A308]", // fadd    dword ptr unk_71A308
            "fstp dword ptr [{b} + 0x31A308]"; // fstp    dword ptr unk_71A308
            b = base()
        );
        hold_low(0x71A308, 0x75B350);
        hold_high(0x71A308, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6D0]", // fmul    flt_75B6D0
            "fadd dword ptr [{b} + 0x31A2E8]", // fadd    dword ptr unk_71A2E8
            "fstp dword ptr [{b} + 0x31A2E8]"; // fstp    dword ptr unk_71A2E8
            b = base()
        );
        hold_low(0x71A2E8, 0x75B350);
        hold_high(0x71A2E8, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6BC]", // fmul    flt_75B6BC
            "fadd dword ptr [{b} + 0x31A298]", // fadd    dword ptr unk_71A298
            "fstp dword ptr [{b} + 0x31A298]"; // fstp    dword ptr unk_71A298
            b = base()
        );
        hold_low(0x71A298, 0x75B350);
        hold_high(0x71A298, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C0]", // fmul    flt_75B6C0
            "fadd dword ptr [{b} + 0x31A2A0]", // fadd    dword ptr unk_71A2A0
            "fstp dword ptr [{b} + 0x31A2A0]"; // fstp    dword ptr unk_71A2A0
            b = base()
        );
        hold_low(0x71A2A0, 0x75B350);
        hold_high(0x71A2A0, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C4]", // fmul    flt_75B6C4
            "fadd dword ptr [{b} + 0x31A2A4]", // fadd    dword ptr unk_71A2A4
            "fstp dword ptr [{b} + 0x31A2A4]"; // fstp    dword ptr unk_71A2A4
            b = base()
        );
        hold_low(0x71A2A4, 0x75B350);
        hold_high(0x71A2A4, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2A8]", // fadd    dword ptr unk_71A2A8
            "fstp dword ptr [{b} + 0x31A2A8]"; // fstp    dword ptr unk_71A2A8
            b = base()
        );
        hold_low(0x71A2A8, 0x75B350);
        hold_high(0x71A2A8, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2AC]", // fadd    dword ptr unk_71A2AC
            "fstp dword ptr [{b} + 0x31A2AC]"; // fstp    dword ptr unk_71A2AC
            b = base()
        );
        hold_low(0x71A2AC, 0x75B350);
        hold_high(0x71A2AC, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2B0]", // fadd    dword ptr unk_71A2B0
            "fstp dword ptr [{b} + 0x31A2B0]"; // fstp    dword ptr unk_71A2B0
            b = base()
        );
        hold_low(0x71A2B0, 0x75B350);
        hold_high(0x71A2B0, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2B4]", // fadd    dword ptr unk_71A2B4
            "fstp dword ptr [{b} + 0x31A2B4]"; // fstp    dword ptr unk_71A2B4
            b = base()
        );
        hold_low(0x71A2B4, 0x75B350);
        hold_high(0x71A2B4, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2B8]", // fadd    dword ptr unk_71A2B8
            "fstp dword ptr [{b} + 0x31A2B8]"; // fstp    dword ptr unk_71A2B8
            b = base()
        );
        hold_low(0x71A2B8, 0x75B350);
        hold_high(0x71A2B8, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2BC]", // fadd    dword ptr unk_71A2BC
            "fstp dword ptr [{b} + 0x31A2BC]"; // fstp    dword ptr unk_71A2BC
            b = base()
        );
        hold_low(0x71A2BC, 0x75B350);
        hold_high(0x71A2BC, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2C0]", // fadd    dword ptr unk_71A2C0
            "fstp dword ptr [{b} + 0x31A2C0]"; // fstp    dword ptr unk_71A2C0
            b = base()
        );
        hold_low(0x71A2C0, 0x75B350);
        hold_high(0x71A2C0, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2C4]", // fadd    dword ptr unk_71A2C4
            "fstp dword ptr [{b} + 0x31A2C4]"; // fstp    dword ptr unk_71A2C4
            b = base()
        );
        hold_low(0x71A2C4, 0x75B350);
        hold_high(0x71A2C4, 0x75B34C);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
            "fmul dword ptr [{b} + 0x35B6C8]", // fmul    flt_75B6C8
            "fadd dword ptr [{b} + 0x31A2C8]", // fadd    dword ptr unk_71A2C8
            "fstp dword ptr [{b} + 0x31A2C8]"; // fstp    dword ptr unk_71A2C8
            b = base()
        );
        hold_low(0x71A2C8, 0x75B350);
        hold_high(0x71A2C8, 0x75B34C);
        r.ebx = g(0x75B3BC);
        let gauge = gear_gauge(r.ebx);
        x87_run!(
            "fld dword ptr [{b} + 0x35B3C8]",
            "fmul dword ptr [{b} + 0x35B6CC]",
            "fadd dword ptr [{v}]",
            "fstp dword ptr [{v}]";
            b = base(), v = gauge
        );
        hold_at(gauge);
        if g(0x75B3C4) == 0x1 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E8]", // fmul    flt_75B6E8
                "fadd dword ptr [{b} + 0x31A2D0]", // fadd    dword ptr unk_71A2D0
                "fstp dword ptr [{b} + 0x31A2D0]"; // fstp    dword ptr unk_71A2D0
                b = base()
            );
            hold_low(0x71A2D0, 0x75B350);
            hold_high(0x71A2D0, 0x75B34C);
        }
        if g(0x75B3C4) == 0x2 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E8]", // fmul    flt_75B6E8
                "fadd dword ptr [{b} + 0x31A2D4]", // fadd    dword ptr unk_71A2D4
                "fstp dword ptr [{b} + 0x31A2D4]"; // fstp    dword ptr unk_71A2D4
                b = base()
            );
            hold_low(0x71A2D4, 0x75B350);
            hold_high(0x71A2D4, 0x75B34C);
        }
        if g(0x75B3C4) == 0x3 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E8]", // fmul    flt_75B6E8
                "fadd dword ptr [{b} + 0x31A2D8]", // fadd    dword ptr unk_71A2D8
                "fstp dword ptr [{b} + 0x31A2D8]"; // fstp    dword ptr unk_71A2D8
                b = base()
            );
            hold_low(0x71A2D8, 0x75B350);
            hold_high(0x71A2D8, 0x75B34C);
        }
        if g(0x75B3C4) == 0x4 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E8]", // fmul    flt_75B6E8
                "fadd dword ptr [{b} + 0x31A2DC]", // fadd    dword ptr unk_71A2DC
                "fstp dword ptr [{b} + 0x31A2DC]"; // fstp    dword ptr unk_71A2DC
                b = base()
            );
            hold_low(0x71A2DC, 0x75B350);
            hold_high(0x71A2DC, 0x75B34C);
        }
        if g(0x75B3C4) == 0x1 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6F0]", // fmul    flt_75B6F0
                "fadd dword ptr [{b} + 0x31A30C]", // fadd    dword ptr unk_71A30C
                "fstp dword ptr [{b} + 0x31A30C]"; // fstp    dword ptr unk_71A30C
                b = base()
            );
            hold_low(0x71A30C, 0x75B350);
            hold_high(0x71A30C, 0x75B34C);
        }
        if g(0x75B3C4) == 0x2 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6F0]", // fmul    flt_75B6F0
                "fadd dword ptr [{b} + 0x31A310]", // fadd    dword ptr unk_71A310
                "fstp dword ptr [{b} + 0x31A310]"; // fstp    dword ptr unk_71A310
                b = base()
            );
            hold_low(0x71A310, 0x75B350);
            hold_high(0x71A310, 0x75B34C);
        }
        if g(0x75B3C4) == 0x3 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6F0]", // fmul    flt_75B6F0
                "fadd dword ptr [{b} + 0x31A314]", // fadd    dword ptr unk_71A314
                "fstp dword ptr [{b} + 0x31A314]"; // fstp    dword ptr unk_71A314
                b = base()
            );
            hold_low(0x71A314, 0x75B350);
            hold_high(0x71A314, 0x75B34C);
        }
        if g(0x75B3C4) == 0x4 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6F0]", // fmul    flt_75B6F0
                "fadd dword ptr [{b} + 0x31A318]", // fadd    dword ptr unk_71A318
                "fstp dword ptr [{b} + 0x31A318]"; // fstp    dword ptr unk_71A318
                b = base()
            );
            hold_low(0x71A318, 0x75B350);
            hold_high(0x71A318, 0x75B34C);
        }
        if g(0x75B3C4) == 0x1 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E4]", // fmul    flt_75B6E4
                "fadd dword ptr [{b} + 0x31A29C]", // fadd    dword ptr unk_71A29C
                "fstp dword ptr [{b} + 0x31A29C]"; // fstp    dword ptr unk_71A29C
                b = base()
            );
            hold_low(0x71A29C, 0x75B350);
            hold_high(0x71A29C, 0x75B34C);
        }
        if g(0x75B3C4) == 0x2 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E4]", // fmul    flt_75B6E4
                "fadd dword ptr [{b} + 0x31A29C]", // fadd    dword ptr unk_71A29C
                "fstp dword ptr [{b} + 0x31A29C]"; // fstp    dword ptr unk_71A29C
                b = base()
            );
            hold_low(0x71A29C, 0x75B350);
            hold_high(0x71A29C, 0x75B34C);
        }
        if g(0x75B3C4) == 0x1 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E0]", // fmul    flt_75B6E0
                "fadd dword ptr [{b} + 0x31A290]", // fadd    dword ptr unk_71A290
                "fstp dword ptr [{b} + 0x31A290]"; // fstp    dword ptr unk_71A290
                b = base()
            );
            hold_low(0x71A290, 0x75B350);
            hold_high(0x71A290, 0x75B34C);
        }
        if g(0x75B3C4) == 0x2 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6E0]", // fmul    flt_75B6E0
                "fadd dword ptr [{b} + 0x31A290]", // fadd    dword ptr unk_71A290
                "fstp dword ptr [{b} + 0x31A290]"; // fstp    dword ptr unk_71A290
                b = base()
            );
            hold_low(0x71A290, 0x75B350);
            hold_high(0x71A290, 0x75B34C);
        }
        if g(0x75B3C4) == 0x1 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6EC]", // fmul    flt_75B6EC
                "fadd dword ptr [{b} + 0x31A320]", // fadd    dword ptr unk_71A320
                "fstp dword ptr [{b} + 0x31A320]"; // fstp    dword ptr unk_71A320
                b = base()
            );
            hold_low(0x71A320, 0x75B350);
            hold_high(0x71A320, 0x75B34C);
        }
        if g(0x75B3C4) == 0x2 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6EC]", // fmul    flt_75B6EC
                "fadd dword ptr [{b} + 0x31A320]", // fadd    dword ptr unk_71A320
                "fstp dword ptr [{b} + 0x31A320]"; // fstp    dword ptr unk_71A320
                b = base()
            );
            hold_low(0x71A320, 0x75B350);
            hold_high(0x71A320, 0x75B34C);
        }
        if g(0x75B3C4) == 0x3 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6EC]", // fmul    flt_75B6EC
                "fadd dword ptr [{b} + 0x31A324]", // fadd    dword ptr unk_71A324
                "fstp dword ptr [{b} + 0x31A324]"; // fstp    dword ptr unk_71A324
                b = base()
            );
            hold_low(0x71A324, 0x75B350);
            hold_high(0x71A324, 0x75B34C);
        }
        if g(0x75B3C4) == 0x4 {
            x87_run!(
                "fld dword ptr [{b} + 0x35B3C8]", // fld     flt_75B3C8
                "fmul dword ptr [{b} + 0x35B6EC]", // fmul    flt_75B6EC
                "fadd dword ptr [{b} + 0x31A324]", // fadd    dword ptr unk_71A324
                "fstp dword ptr [{b} + 0x31A324]"; // fstp    dword ptr unk_71A324
                b = base()
            );
            hold_low(0x71A324, 0x75B350);
            hold_high(0x71A324, 0x75B34C);
        }
    }
    x87_run!(
        "fld dword ptr [{b} + 0x35B3CC]", // fld     dword_75B3CC
        "fmul dword ptr [{b} + 0x35B6B0]", // fmul    flt_75B6B0
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A30C]", // fadd    dword ptr unk_71A30C
        "fstp dword ptr [{b} + 0x31A30C]"; // fstp    dword ptr unk_71A30C
        b = base()
    );
    hold_low(0x71A30C, 0x75B350);
    hold_high(0x71A30C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B3D0]", // fld     flt_75B3D0
        "fmul dword ptr [{b} + 0x35B6B0]", // fmul    flt_75B6B0
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A310]", // fadd    dword ptr unk_71A310
        "fstp dword ptr [{b} + 0x31A310]"; // fstp    dword ptr unk_71A310
        b = base()
    );
    hold_low(0x71A310, 0x75B350);
    hold_high(0x71A310, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B3D4]", // fld     flt_75B3D4
        "fmul dword ptr [{b} + 0x35B6B0]", // fmul    flt_75B6B0
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A314]", // fadd    dword ptr unk_71A314
        "fstp dword ptr [{b} + 0x31A314]"; // fstp    dword ptr unk_71A314
        b = base()
    );
    hold_low(0x71A314, 0x75B350);
    hold_high(0x71A314, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B3D8]", // fld     flt_75B3D8
        "fmul dword ptr [{b} + 0x35B6B0]", // fmul    flt_75B6B0
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A318]", // fadd    dword ptr unk_71A318
        "fstp dword ptr [{b} + 0x31A318]"; // fstp    dword ptr unk_71A318
        b = base()
    );
    hold_low(0x71A318, 0x75B350);
    hold_high(0x71A318, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B3DC]", // fld     flt_75B3DC
        "fmul dword ptr [{b} + 0x35B6B4]", // fmul    flt_75B6B4
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A294]", // fadd    dword ptr unk_71A294
        "fstp dword ptr [{b} + 0x31A294]"; // fstp    dword ptr unk_71A294
        b = base()
    );
    hold_low(0x71A294, 0x75B350);
    hold_high(0x71A294, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B3DC]", // fld     flt_75B3DC
        "fmul dword ptr [{b} + 0x35B6B8]", // fmul    flt_75B6B8
        "fmul dword ptr [{b} + 0x35B370]", // fmul    flt_75B370
        "fadd dword ptr [{b} + 0x31A308]", // fadd    dword ptr unk_71A308
        "fstp dword ptr [{b} + 0x31A308]"; // fstp    dword ptr unk_71A308
        b = base()
    );
    hold_low(0x71A308, 0x75B350);
    hold_high(0x71A308, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x31A2EC]", // fld     dword ptr unk_71A2EC
        "fadd dword ptr [{b} + 0x31A2F0]", // fadd    dword ptr unk_71A2F0
        "fadd dword ptr [{b} + 0x31A2F4]", // fadd    dword ptr unk_71A2F4
        "fadd dword ptr [{b} + 0x31A2F8]", // fadd    dword ptr unk_71A2F8
        "fadd dword ptr [{b} + 0x31A300]", // fadd    dword ptr unk_71A300
        "fadd dword ptr [{b} + 0x31A2FC]", // fadd    dword ptr unk_71A2FC
        "fdiv dword ptr [{b} + 0x35B344]", // fdiv    flt_75B344
        "fstp dword ptr [{b} + 0x31A304]", // fstp    dword ptr unk_71A304
        "fld dword ptr [{b} + 0x31A320]", // fld     dword ptr unk_71A320
        "fadd dword ptr [{b} + 0x31A324]", // fadd    dword ptr unk_71A324
        "fmul dword ptr [{b} + 0x35B360]", // fmul    flt_75B360
        "fstp dword ptr [{b} + 0x31A328]", // fstp    dword ptr unk_71A328
        "fld dword ptr [{b} + 0x31A32C]", // fld     dword ptr unk_71A32C
        "fadd dword ptr [{b} + 0x31A330]", // fadd    dword ptr unk_71A330
        "fmul dword ptr [{b} + 0x35B360]", // fmul    flt_75B360
        "fstp dword ptr [{b} + 0x35B440]", // fstp    flt_75B440
        "fld dword ptr [{b} + 0x31A334]", // fld     dword ptr unk_71A334
        "fadd dword ptr [{b} + 0x31A338]", // fadd    dword ptr unk_71A338
        "fmul dword ptr [{b} + 0x35B360]", // fmul    flt_75B360
        "fstp dword ptr [{b} + 0x35B444]", // fstp    flt_75B444
        "fld dword ptr [{b} + 0x35B440]", // fld     flt_75B440
        "fadd dword ptr [{b} + 0x35B444]", // fadd    flt_75B444
        "fmul dword ptr [{b} + 0x35B360]", // fmul    flt_75B360
        "fstp dword ptr [{b} + 0x31A33C]", // fstp    dword ptr unk_71A33C
        "fld dword ptr [{b} + 0x31A2A8]", // fld     dword ptr unk_71A2A8
        "fadd dword ptr [{b} + 0x31A2B0]", // fadd    dword ptr unk_71A2B0
        "fadd dword ptr [{b} + 0x31A2B4]", // fadd    dword ptr unk_71A2B4
        "fadd dword ptr [{b} + 0x31A2B8]", // fadd    dword ptr unk_71A2B8
        "fadd dword ptr [{b} + 0x31A2BC]", // fadd    dword ptr unk_71A2BC
        "fadd dword ptr [{b} + 0x31A2C0]", // fadd    dword ptr unk_71A2C0
        "fadd dword ptr [{b} + 0x31A2C4]", // fadd    dword ptr unk_71A2C4
        "fadd dword ptr [{b} + 0x31A2C8]", // fadd    dword ptr unk_71A2C8
        "fmul dword ptr [{b} + 0x35B360]", // fmul    flt_75B360
        "fmul dword ptr [{b} + 0x35B360]", // fmul    flt_75B360
        "fmul dword ptr [{b} + 0x35B360]", // fmul    flt_75B360
        "fstp dword ptr [{b} + 0x31A2CC]", // fstp    dword ptr unk_71A2CC
        "fld dword ptr [{b} + 0x31A2D0]", // fld     dword ptr unk_71A2D0
        "fadd dword ptr [{b} + 0x31A2D4]", // fadd    dword ptr unk_71A2D4
        "fadd dword ptr [{b} + 0x31A2D8]", // fadd    dword ptr unk_71A2D8
        "fadd dword ptr [{b} + 0x31A2DC]", // fadd    dword ptr unk_71A2DC
        "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
        "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
        "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
        "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
        "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
        "fadd dword ptr [{b} + 0x31A2E0]", // fadd    dword ptr unk_71A2E0
        "fmul dword ptr [{b} + 0x35B368]", // fmul    flt_75B368
        "fstp dword ptr [{b} + 0x31A2E4]", // fstp    dword ptr unk_71A2E4
        "fld dword ptr [{b} + 0x31A30C]", // fld     dword ptr unk_71A30C
        "fadd dword ptr [{b} + 0x31A310]", // fadd    dword ptr unk_71A310
        "fadd dword ptr [{b} + 0x31A314]", // fadd    dword ptr unk_71A314
        "fadd dword ptr [{b} + 0x31A318]", // fadd    dword ptr unk_71A318
        "fmul dword ptr [{b} + 0x35B364]", // fmul    flt_75B364
        "fstp dword ptr [{b} + 0x31A31C]", // fstp    dword ptr unk_71A31C
        "fld dword ptr [{b} + 0x31A420]", // fld     dword ptr unk_71A420
        "fadd dword ptr [{b} + 0x31A424]", // fadd    dword ptr unk_71A424
        "fadd dword ptr [{b} + 0x31A428]", // fadd    dword ptr unk_71A428
        "fadd dword ptr [{b} + 0x31A42C]", // fadd    dword ptr unk_71A42C
        "fmul dword ptr [{b} + 0x35B364]", // fmul    flt_75B364
        "fstp dword ptr [{b} + 0x31A430]"; // fstp    dword ptr unk_71A430
        b = base()
    );
    if g(0x75B3E4) == 0x0 {
        p(0x71A298, 0x0);
    }
    // retn
}

/// The Rust version.
pub unsafe fn damage_gauges(entry: &Regs) -> Left {
    let mut r = *entry;
    steps(&mut r);
    left_of(&r)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { damage_gauges(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    damage_test(&HOOK, call_original, &rust)
}
