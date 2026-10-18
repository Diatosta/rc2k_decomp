//! `hud::add_scaled_sprite` (0x4B716A): as `hud::add_sprite` with the width times st(0)
//! and the height times st(1) (`fistp`ed through `dword_75C060`) before they are made
//! positive; st(0) and st(1) are read, not popped.
//!
//! In:  esi the rectangle, ebx x, ecx y, st(0) / st(1) the scales (kept)
//! Out: all kept (`pusha` .. `popa`)
//!
//! Callers: the HUD's drawers (code at 0x4B28B0, ...). The quad through `part` (1), in the
//! game its Rust version. Thunk by `two_kept_hook!`. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose call goes to a stand-in, against the Rust version,
//! rectangles and scales by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva, shadow};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::add_sprite::{positive, put_places};
use crate::scene::matrix3::extended;
use crate::scene::vector3::{Pair80, call_with_two_kept, two_kept_hook};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PAIR: Pair80 = Pair80::new();

pub static HOOK: RawHook =
    RawHook::new("hud::add_scaled_sprite", rva(0x4B716A), thunk, &ORIGINAL).self_test(self_test);

two_kept_hook!(
    thunk,
    call_original,
    original_with_two,
    dispatch = dispatch,
    original = ORIGINAL,
    pair = PAIR
);

const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060

/// A call by place: 1 `add_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `fild i; fmul st, st(scale)` with the scale extended; `fistp`.
fn int_times_80_to_int(i: u32, scale: [u8; 10]) -> u32 {
    let (v, mut out) = (i, 0u32);
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{s}]",
            "fild dword ptr [{v}]",
            "fmul st, st(1)",
            "fistp dword ptr [{o}]",
            "fstp st(0)",
            s = in(reg) scale.as_ptr(),
            v = in(reg) &raw const v,
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _,
            options(nostack),
        )
    };
    out
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn add_scaled_sprite(
    entry: &Regs,
    scales: [[u8; 10]; 2],
    part: Part,
) -> Result<Regs, Refused> {
    unsafe {
        let [_, _, w, h] = put_places(entry.esi);
        game_write(QUAD_RVA + 4, entry.ebx);
        let w = int_times_80_to_int(w, scales[0]);
        game_write(SCRATCH_RVA, w);
        game_write(QUAD_RVA + 12, positive(w).wrapping_add(entry.ebx));
        game_write(QUAD_RVA + 8, entry.ecx);
        let h = int_times_80_to_int(h, scales[1]);
        game_write(SCRATCH_RVA, h);
        game_write(QUAD_RVA + 16, positive(h).wrapping_add(entry.ecx));
    }
    let mut r = Regs {
        eax: unsafe { game_read(QUAD_RVA + 16) },
        ..*entry
    };
    part(1, &mut r)?;
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { original_with_two(r) };
        return;
    }
    match unsafe {
        add_scaled_sprite(
            r,
            PAIR.inputs(),
            &mut crate::hud::add_triangle::game_part_quad,
        )
    } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, original_with_two) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov eax, [esi]; shl eax, 10h
    let entry: &[u8] = &[0x60, 0x8B, 0x06, 0xC1, 0xE0, 0x10];
    let copy = match copy(0x4B716A, 0xAC, entry, &[0xA5], &[(0x4B72D1, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![(game_addr(QUAD_RVA) as u32, 0x38), at(SCRATCH_RVA)];
    let saved = capture(&regions);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let rect: [i32; 4] = [case as i32 * 8, 16, if case == 2 { -24 } else { 24 }, 13];
        let scales = [
            extended((0.75f32 + case as f32 * 0.5).to_bits()),
            extended((if case == 4 { -1.5f32 } else { 2.5 }).to_bits()),
        ];
        let before = Regs {
            esi: rect.as_ptr() as u32,
            ebx: 100 + case,
            ecx: 50 * case,
            ..Regs::SENTINEL
        };
        let prepare =
            || unsafe { std::ptr::write_bytes(game_addr(QUAD_RVA) as *mut u8, 0xAA, 0x38) };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { call_with_two_kept(&mut ro, copy.addr(), &scales) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            add_scaled_sprite(&before, scales, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?} {:x?}; rust {:x?} {:x?}; state same {}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2 == mine.2
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
