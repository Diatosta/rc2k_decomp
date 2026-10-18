//! `marks::spawn_square` (0x4C0120): a square item made live: of the 32 items from the
//! cursor `dword_89D166` on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (eax, ebx as floats at +0x20, +0x24), its size from its kind (ecx & 7, times
//! `flt_8BC23A` plus `flt_8BC23E`, +0xC), `flt_8BC242` at +0x10, the clock (`g_sceneClock`,
//! +4), +8 0 and the rest 0; the cursor the item after it. With none free, the cursor
//! moves on 32.
//!
//! In:  eax x, ebx z, ecx the kind
//! Out: ecx the kind & 7 when one was made; the rest kept
//!
//! Caller: `marks::pick_square`. Self-test: the original and the Rust version on the
//! game's items, cursors and fillings by lot, a full window among them; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::expire_squares::{ITEM_LEN, ITEMS, ITEMS_RVA};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::spawn_square", rva(0x4C0120), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CURSOR_RVA: usize = rva(0x89D166); // dword_89D166: an offset into the items
const SIZE_K: [usize; 3] = [rva(0x8BC23A), rva(0x8BC23E), rva(0x8BC242)];
const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const WINDOW: u32 = 32;

/// The Rust version (see the header): ecx as it leaves it.
pub unsafe fn spawn_square(x: u32, z: u32, kind: u32) -> u32 {
    unsafe {
        let base = game_addr(ITEMS_RVA) as u32;
        let end = ITEMS * ITEM_LEN;
        let mut at = game_read::<u32>(CURSOR_RVA);
        let step = |at: &mut u32| {
            *at += ITEM_LEN;
            if *at == end {
                *at = 0;
            }
        };
        for _ in 0..WINDOW {
            let e = base.wrapping_add(at);
            if (e as *const u32).read_unaligned() == 0 {
                let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
                put(0, 1);
                put(0x20, x87::int_to_float(x as i32));
                put(0x24, x87::int_to_float(z as i32));
                let k = kind & 7;
                put(
                    0xC,
                    x87::int_times_plus(k as i32, game_read(SIZE_K[0]), game_read(SIZE_K[1])),
                );
                put(0x10, x87::loaded(game_read(SIZE_K[2])));
                for o in [0x28, 0x2C, 0x38, 0x14, 0x18, 0x1C, 0x30, 0x34] {
                    put(o, 0);
                }
                put(4, game_read(CLOCK_RVA));
                put(8, 0);
                step(&mut at);
                game_write(CURSOR_RVA, at);
                return k;
            }
            step(&mut at);
        }
        game_write(CURSOR_RVA, at);
        kind
    }
}

pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(ITEMS_RVA) as u32, (ITEMS * ITEM_LEN) as usize),
        at(CURSOR_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let ecx = unsafe { spawn_square(r.eax, r.ebx, r.ecx) };
    Ok(left_of(&Regs { ecx, ..*r }))
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
    vars.extend(SIZE_K.map(at));
    let saved = capture(&vars);
    for case in 0..6u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                eax: 0x123 + case,
                ebx: 0x3FF - case,
                ecx: 0x5A + case,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SIZE_K[0], 1.5f32.to_bits());
                game_write(SIZE_K[1], (0.25f32 * case as f32).to_bits());
                game_write(SIZE_K[2], 7.0f32.to_bits());
                let base = game_addr(ITEMS_RVA) as u32;
                std::ptr::write_bytes(base as *mut u8, 0xAA, (ITEMS * ITEM_LEN) as usize);
                let start = if case == 4 { ITEMS - 3 } else { case * 100 };
                for n in 0..ITEMS {
                    let free = case != 3 && n == (start + case * 3 + 2) % ITEMS;
                    ((base + n * ITEM_LEN) as *mut u32).write_unaligned(u32::from(!free));
                }
                game_write(CURSOR_RVA, start * ITEM_LEN);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
