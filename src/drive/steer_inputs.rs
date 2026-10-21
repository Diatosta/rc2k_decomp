//! `drive::steer_inputs` (0x439DB1): the work car's steering from the input word
//! `dword_613890`. Key steering (the word 0, or a car not driven by device, `unk_71A234`
//! not 2, with the keys' bit 0x1000 of `unk_71A440`): the step 0xA00000 x `flt_614C0C`
//! (`fistp`) moves the two steering values `unk_71A620`, `unk_71A61C` toward the side the
//! word says, held at ± 0x20000000, or, with the word 0, back toward 0 at twice the step
//! (stopping at 0; each by the sign of the result, as the original). Device steering:
//! the word x 0x20000000 >> 8 (`imul`, `shrd`) x `flt_614C08`, held ± 0x20000000, into
//! both (`unk_71A61C` 0 first). Then the trim (0x20000000 x `unk_71A3E0`, negated with
//! `unk_719FE8` bit 0x10) into `unk_71A1E0`; `unk_71A624` = `unk_71A61C` + it,
//! `unk_71A628` = `unk_71A620` + 4 x it; both 0 unless `unk_71A308` is below
//! `flt_5C5D14` (or unordered).
//!
//! In:  nothing
//! Out: eax `unk_71A628` before the last test, ebx `unk_71A440` or 0x20000000 (device),
//!      edx the product's high dword (device); the rest kept; no flags read
//!
//! Callers: `drive::car_controls`, `sub_460835`, `sub_460B2F`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) by lot: both kinds of steering, words left, right and 0, values about their
//! ends, against the converted version, exact at double precision.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, base, c0, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::{
    Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::{capture, restore};
use rc2k_core::drive::steering::{SteerInputs, steer as steer_core};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::steer_inputs", rva(0x439DB1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WORD: usize = 0x613890;
const LEFT: u32 = 0xE000_0000;
const RIGHT: u32 = 0x2000_0000;

fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// The key step: 0xA00000 x `flt_614C0C`, through `dword_62400C`.
fn key_step() -> u32 {
    p(0x62400C, 0xA0_0000);
    x87_run!(
        "fild dword ptr [{b} + 0x22400C]",
        "fmul dword ptr [{b} + 0x214C0C]",
        "fistp dword ptr [{b} + 0x22400C]";
        b = base()
    );
    g(0x62400C)
}

/// One steering value moved by the key step (0x439DFA .. / 0x439E95 ..).
fn steer(value: usize, step: u32) {
    let word = g(WORD);
    let v = g(value);
    if word != 0 {
        if sign(word) {
            let n = v.wrapping_sub(step);
            p(value, if sign(n.wrapping_sub(LEFT)) { LEFT } else { n });
        } else {
            let n = v.wrapping_add(step);
            p(
                value,
                if sign(n.wrapping_sub(RIGHT)) {
                    n
                } else {
                    RIGHT
                },
            );
        }
        return;
    }
    let step = step.wrapping_add(step);
    if v == 0 {
        return;
    }
    if sign(v) {
        let n = v.wrapping_add(step);
        p(value, if sign(n) { n } else { 0 });
    } else {
        let n = v.wrapping_sub(step);
        p(value, if sign(n) { 0 } else { n });
    }
}

/// The Rust version (the exact one, for `drive::simple_car_step`, `drive::replay_step`
/// and `race::step_replay_cars`).
pub unsafe fn steer_inputs(entry: &Regs) -> Left {
    let mut r = *entry;
    let mut keys = g(WORD) == 0;
    if !keys && g(0x71A234) != 2 {
        r.ebx = g(0x71A440);
        keys = r.ebx & 0x1000 != 0;
    }
    if keys {
        steer(0x71A620, key_step());
        steer(0x71A61C, key_step());
    } else {
        p(0x71A61C, 0);
        let product = i64::from(g(WORD) as i32) * i64::from(RIGHT as i32);
        r.ebx = RIGHT;
        r.edx = (product >> 32) as u32;
        p(0x62400C, (product >> 8) as u32);
        x87_run!(
            "fild dword ptr [{b} + 0x22400C]",
            "fmul dword ptr [{b} + 0x214C08]",
            "fistp dword ptr [{b} + 0x22400C]";
            b = base()
        );
        if sign(g(0x62400C).wrapping_sub(LEFT)) {
            p(0x62400C, LEFT);
        }
        if !sign(g(0x62400C).wrapping_sub(RIGHT)) {
            p(0x62400C, RIGHT);
        }
        p(0x71A620, g(0x62400C));
        p(0x71A61C, g(0x62400C));
    }
    // 0x439F7D: the trim.
    p(0x62400C, RIGHT);
    x87_run!(
        "fild dword ptr [{b} + 0x22400C]",
        "fmul dword ptr [{b} + 0x31A3E0]",
        "fistp dword ptr [{b} + 0x31A1E0]";
        b = base()
    );
    if g(0x719FE8) & 0x10 != 0 {
        p(0x71A1E0, g(0x71A1E0).wrapping_neg());
    }
    let trim = g(0x71A1E0);
    p(0x71A624, g(0x71A61C).wrapping_add(trim));
    r.eax = g(0x71A620).wrapping_add(trim.wrapping_mul(4));
    p(0x71A628, r.eax);
    if !c0(g(0x71A308), g(0x5C5D14)) {
        p(0x71A624, 0);
        p(0x71A628, 0);
    }
    Left::nothing()
        .with(EAX, r.eax)
        .with(EBX, r.ebx)
        .with(EDX, r.edx)
}

/// The converted version (`rc2k_core::drive::steering`, plain floats): the hook's.
// Converted: `rc2k_core::drive::steering::steer`.
pub unsafe fn steer_inputs_plain(entry: &Regs) -> Left {
    let f = |x: usize| f32::from_bits(g(x));
    let s = steer_core(
        &SteerInputs {
            word: g(WORD),
            mode: g(0x71A234),
            keys: g(0x71A440),
            device_factor: f(0x614C08),
            key_factor: f(0x614C0C),
            trim: f(0x71A3E0),
            flags: g(0x719FE8),
            figure: f(0x71A308),
            limit: f(0x5C5D14),
        },
        [g(0x71A61C), g(0x71A620)],
    );
    p(0x71A61C, s.values[0]);
    p(0x71A620, s.values[1]);
    p(0x62400C, s.scratch);
    p(0x71A1E0, s.trim);
    p(0x71A624, s.trimmed[0]);
    p(0x71A628, s.trimmed[1]);
    Left::nothing()
        .with(EAX, s.eax)
        .with(EBX, s.ebx.unwrap_or(entry.ebx))
        .with(EDX, s.edx.unwrap_or(entry.edx))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { steer_inputs_plain(r) })
}

/// The whole numbers made from floats, held to `ROUNDED` at extended precision.
fn floats() -> [(crate::service::harness::Region, Tolerance); 2] {
    [((a(0x71A61C), 0x10), ROUNDED), ((a(0x71A1E0), 4), ROUNDED)]
}

fn regions() -> [crate::service::harness::Region; 3] {
    [(a(0x71A61C), 0x10), (a(0x71A1E0), 4), (a(0x62400C), 4)]
}

extern "C" fn dispatch(r: &mut Regs) {
    let memory = floats();
    let floats = Floats {
        memory: &memory,
        regs: &[(EAX, ROUNDED)],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(0x71A61C), 0x10),
        (a(0x71A1E0), 4),
        (a(0x62400C), 4),
        (a(WORD), 4),
        (a(0x71A234), 4),
        (a(0x71A440), 4),
        (a(0x614C08), 8),
        (a(0x71A3E0), 4),
        (a(0x719FE8), 4),
        (a(0x71A308), 4),
    ];
    let saved = capture(&all);
    let memory = floats();
    let tolerant = Floats {
        memory: &memory,
        regs: &[(EAX, ROUNDED)],
    };
    at_test_precisions(&tolerant, &mut |word, fl| {
        for case in 0..40u32 {
            let set = || {
                p(
                    WORD,
                    [0u32, 0x80, 0xFFFF_FF80, 0x100, 0xFFFF_FF00][(case % 5) as usize],
                );
                p(0x71A234, if case % 3 == 0 { 2 } else { 1 });
                p(0x71A440, if case % 4 < 2 { 0x1000 } else { 0 });
                p(0x614C08, 0.7f32.to_bits());
                p(0x614C0C, [0.5f32, 3.0, 40.0][(case % 3) as usize].to_bits());
                for (k, x) in [0x71A61C, 0x71A620].iter().enumerate() {
                    p(
                        *x,
                        [0u32, 0x1F00_0000, 0xE100_0000, 0x0010_0000, 0xFFF0_0000]
                            [((case + k as u32) % 5) as usize],
                    );
                }
                p(0x71A3E0, (0.01f32 * (case % 7) as f32 - 0.03).to_bits());
                p(0x719FE8, case % 2 * 0x10);
                p(0x71A308, [1.0f32, 50.0][(case / 5 % 2) as usize].to_bits());
                p(0x5C5D14, g(0x5C5D14));
            };
            check_state_floats(
                &mut t,
                "drive::steer_inputs",
                &format!("control word {word:#x}, case {case}"),
                call_original,
                Regs::SENTINEL,
                &set,
                &regions(),
                fl,
                &rust,
            );
        }
    });
    restore(&all, &saved);
    t.finish()
}
