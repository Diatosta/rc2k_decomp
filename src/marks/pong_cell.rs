//! `marks::pong_cell` (0x4B8557): one cell of a score digit: unless the pattern byte at
//! eax is 0, `marks::draw_pong_box` at (ebx times `flt_7691FA` plus `dword_7687D4`, ecx
//! times `flt_7691FE` plus `dword_7687D8`) sized `flt_769202` by `flt_769206`; eax on one.
//!
//! In:  eax the pattern byte, ebx the column, ecx the row
//! Out: eax plus 1; the rest kept (`pusha` .. `popa`)
//!
//! Caller: `marks::pong_digit` (twenty times). The box through `part` (1 with its four
//! values), in the game its Rust version. Hooked through a private copy (a `jz` in the
//! first bytes). Shadow mode runs the original alone (it draws). Self-test: a copy whose
//! box goes to a stand-in taking the values off, against the Rust version.
//!
//! Converted 2026-10-21: the box's place through `rc2k_core::marks::pong::cell_place` (a
//! whole number times a float plus a whole number: exact in `f64`, as the x87's at double
//! precision; at extended too while the product fits 53 bits, as the score's cells do).

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::pong;
use crate::scene::matrix3::extended;
use crate::service::harness::{at, capture, restore};
use rc2k_core::math::extended::from_f64;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::pong_cell", rva(0x4B8557), thunk, &ORIGINAL)
    .private_copy(0x42)
    .relocate_calls(&[0x3A])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const STEP_RVA: [usize; 2] = [rva(0x7691FA), rva(0x7691FE)]; // flt_7691FA, flt_7691FE
const SIZE_RVA: [usize; 2] = [rva(0x769202), rva(0x769206)]; // flt_769202, flt_769206

/// A call by place: 1 `draw_pong_box`, with its four values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::pong::cell_place`
pub unsafe fn pong_cell(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if (g.eax as *const u8).read() != 0 {
            let step = STEP_RVA.map(|r| f32::from_bits(game_read(r)));
            let at = pong::DIGIT_AT.map(|r| game_read::<i32>(r));
            let [x, y] = rc2k_core::marks::pong::cell_place(g.ebx as i32, g.ecx as i32, step, at);
            let values = [
                extended(game_read(SIZE_RVA[1])),
                extended(game_read(SIZE_RVA[0])),
                from_f64(y),
                from_f64(x),
            ];
            let mut r = g;
            part(1, &mut r, &values)?;
        }
    }
    g.eax = g.eax.wrapping_add(1);
    Ok(g)
}

/// In the game: `draw_pong_box`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::marks::draw_pong_box as pb;
    unsafe { pb::draw_pong_box(r, [v[0], v[1], v[2], v[3]], &mut pb::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { pong_cell(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4B8557)),
            0x42,
            // pusha; cmp byte [eax], 0; jz
            &[0x60, 0x80, 0x38, 0x00, 0x74, 0x39],
            &[0x3A],
            &|c| (c + 0x40_0000 == 0x4B8599).then_some(pong::box_stand_in as *const () as usize),
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = [
        STEP_RVA[0],
        STEP_RVA[1],
        SIZE_RVA[0],
        SIZE_RVA[1],
        pong::DIGIT_AT[0],
        pong::DIGIT_AT[1],
    ]
    .map(at);
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let pattern = [1u8, 0, 7, 0];
    for case in 0..4u32 {
        let before = Regs {
            eax: pattern.as_ptr() as u32 + case,
            ebx: case % 4,
            ecx: case + 1,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(STEP_RVA[0], 5.5f32.to_bits());
            game_write(STEP_RVA[1], (4.0f32 + case as f32).to_bits());
            game_write(SIZE_RVA[0], 5.0f32.to_bits());
            game_write(SIZE_RVA[1], 3.0f32.to_bits());
            game_write(pong::DIGIT_AT[0], 0x100 + case * 0x20);
            game_write(pong::DIGIT_AT[1], 0x384u32);
        };
        prepare();
        pong::take();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), pong::take());
        prepare();
        let mine = unsafe {
            pong_cell(&before, &mut |_, r, v| {
                pong::note(pong::BOX_PLACE, r, v);
                Ok(())
            })
        };
        let mine = (mine.as_ref().ok().map(regs), pong::take());
        t.check(theirs == mine, || {
            format!("case {case}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&vars, &saved);
    t.finish()
}
