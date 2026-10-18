//! `physics::object_turn_matrix` (0x45E2D2): the turn of the object hit (`dword_6D321C`)
//! into `flt_5EA444`. `scene::set_turn_y` by minus `dword_63503C`, copied (16 dwords);
//! with `dword_635040` set also: the turn about y by minus `dword_635044` times it into
//! `g_litMatrix` (`scene::multiply_matrices`), the turn about x by minus `dword_635040`
//! times that into `g_turnedMatrix`, the turn about y by `dword_635044` times that into
//! `flt_5EA444`. Then each of its 16 floats times `dword_635038` / 16 (`fild`, `fld1`
//! doubled four times, `fdivp`; the share through `dword_62400C`).
//!
//! In:  nothing
//! Out: the registers as the steps leave them (eax, ebx `dword_635038`, ecx 0, edx, esi
//!      past the matrix, edi, ebp); no flags read (the caller returns)
//!
//! Caller: `sub_45E755`.
//! Refuses (before anything) an object past 0xFF.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): angles by lot, the x turn 0 and not, scales, under three precisions.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, ext_div2, int_80, with_control_word};
use crate::scene::matrix::{multiply_matrices, set_turn_x, set_turn_y};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::object_turn_matrix",
    rva(0x45E2D2),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

const OUT: usize = 0x5EA444; // flt_5EA444
const TURN_Y: usize = 0x5E9FCC; // g_turnYMatrix
const TURN_X: usize = 0x5E9F8C; // g_turnXMatrix
const LIT: usize = 0x5EA14C; // g_litMatrix
const TURNED: usize = 0x5EA10C; // g_turnedMatrix

/// The Rust version.
pub unsafe fn object_turn_matrix(entry: &Regs) -> Result<Left, Refused> {
    let object = g(0x6D321C);
    if object > 0xFF {
        return Err(BadInput("an object past 0xFF").into());
    }
    let mut r = *entry;
    unsafe {
        let kind = g(0x5CD27C + 4 * object as usize);
        r.eax = a(0x97DF30).wrapping_add(kind.wrapping_mul(0x80));
        r.edx = g(0x63503C).wrapping_neg();
        set_turn_y(r.edx).apply(&mut r);
        for k in 0..16 {
            p(OUT + 4 * k, g(TURN_Y + 4 * k));
        }
        r.ecx = 0;
        r.esi = a(TURN_Y) + 0x40;
        r.edi = a(OUT) + 0x40;
        if g(0x635040) != 0 {
            r.edx = g(0x635044).wrapping_neg();
            set_turn_y(r.edx).apply(&mut r);
            multiply_matrices(a(OUT) as usize, a(TURN_Y) as usize, a(LIT) as usize).apply(&mut r);
            r.esi = a(OUT);
            r.edi = a(TURN_Y);
            r.ebp = a(LIT);
            r.edx = g(0x635040).wrapping_neg();
            set_turn_x(r.edx).apply(&mut r);
            multiply_matrices(a(LIT) as usize, a(TURN_X) as usize, a(TURNED) as usize)
                .apply(&mut r);
            r.esi = a(LIT);
            r.edi = a(TURN_X);
            r.ebp = a(TURNED);
            r.edx = g(0x635044);
            set_turn_y(r.edx).apply(&mut r);
            multiply_matrices(a(TURNED) as usize, a(TURN_Y) as usize, a(OUT) as usize)
                .apply(&mut r);
            r.esi = a(TURNED);
            r.edi = a(TURN_Y);
            r.ebp = a(OUT);
        }
        r.ebx = g(0x635038);
        p(0x62400C, r.ebx);
        let sixteen = 16.0f32.to_bits();
        let share = ext_div2(int_80(r.ebx as i32, false), sixteen, 0x3F80_0000);
        p(0x62400C, share);
        for k in 0..16 {
            p(OUT + 4 * k, x87::mul(g(OUT + 4 * k), share));
        }
        r.esi = a(OUT) + 0x40;
        r.ecx = 0;
    }
    Ok(crate::race::stage_prefs::left_of(&r))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { object_turn_matrix(r) }
}

pub fn regions() -> Vec<Region> {
    vec![
        (a(OUT), 0x40),
        (a(TURN_Y), 0x40),
        (a(TURN_X), 0x40),
        (a(LIT), 0x40),
        (a(TURNED), 0x40),
        (a(0x62400C), 4),
        (a(0x624148), 16), // the turns' work (sine, cosine)
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend([(a(0x6D321C), 4), (a(0x635038), 16)]);
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..6u32 {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| {
                        p(0x6D321C, seed * 9);
                        p(0x635038, 4 + seed * 5);
                        p(0x63503C, seed.wrapping_mul(0x2345_6789));
                        p(
                            0x635040,
                            if seed % 2 == 0 {
                                0
                            } else {
                                seed.wrapping_mul(0x1111_1111)
                            },
                        );
                        p(0x635044, seed.wrapping_mul(0x0F0F_0F0F));
                    },
                    &all,
                    &rust,
                );
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
