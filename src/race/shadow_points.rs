//! `race::shadow_points` (0x458078): the car's shadow's points (see `race/car_shadow.rs`):
//! the ground under the work car (`physics::find_ground`), the sun's slant, each corner of
//! the car turned, moved along the slant, its angle (`fpatan`), distance, the farthest,
//! as whole numbers on the ground; x87 as the original.
//!
//! In:  nothing
//! Out: eax the ground's height (`dword_641AA0`), ebx the count, esi `unk_71A818`, edi past
//!      the last point (no `pusha`); ecx, edx, ebp kept
//!
//! Refuses (before anything) what `find_ground` refuses (its inputs put back) and no
//! corners or more than 0x40. Shadow mode: by state. Self-test: the work car's corners,
//! turn and place set by pattern, the sun's angles; all put back (skipped without a
//! race: the ground is the stage's).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDI, ESI, Regs, game_addr, game_read, game_write, rva};
use crate::race::car_shadow::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

pub fn points_regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut v = crate::physics::probe_ground::regions(&crate::physics::ground::Ground::game());
    v.extend([
        g(0x641AA4, 0x20 * 0x40),
        g(0x643AB8, 0x14),
        at(GROUND_RVA),
        g(0x639824, 0xC),
        at(rva(0x5CCC78)),
        at(rva(0x62400C)),
    ]);
    v
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { shadow_points()? };
    Ok(left())
}

/// The registers it leaves (no `pusha`): eax the ground's height (`dword_641AA0`, its last
/// `mov eax`), ebx the count, esi the work car's turn (`unk_71A818`), edi past the last point.
pub fn left() -> Left {
    let count: u32 = unsafe { game_read(COUNT_RVA) };
    Left::nothing()
        .with(EAX, unsafe { game_read(GROUND_RVA) })
        .with(EBX, count)
        .with(ESI, game_addr(rva(0x71A818)) as u32)
        .with(EDI, game_addr(rva(0x641AA4)) as u32 + 0x20 * count)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The work car's shadow inputs a test sets.
pub fn car_shadow_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x71A10C, 0xC),
        g(0x71A818, 0x40),
        g(0x71AAC8, 0x300),
        at(rva(0x71ADD8)),
        at(COUNT_RVA),
        at(rva(0x60933C)),
        at(rva(0x609338)),
    ]
}

/// Sets the work car's corners (a box), turn and place by `seed`.
pub fn set_car_shadow_inputs(seed: u32, count: u32) {
    unsafe {
        let car = game_addr(rva(0x719FD8)) as u32;
        let w = |o: u32, v: u32| ((car + o) as *mut u32).write_unaligned(v);
        let mut s = seed;
        let m = crate::scene::matrix::test_floats(&mut s);
        for (k, &v) in m.iter().enumerate().take(12) {
            w(0x840 + 4 * k as u32, v);
        }
        for k in 0..count {
            let x = if k & 1 == 0 { -1.0f32 } else { 1.0 };
            let y = if k & 2 == 0 { 0.2f32 } else { 1.1 };
            let z = if k & 4 == 0 { -2.0f32 } else { 2.0 };
            w(0xAF0 + 4 * k, x.to_bits());
            w(0xBF0 + 4 * k, y.to_bits());
            w(0xCF0 + 4 * k, z.to_bits());
        }
        game_write(COUNT_RVA, count);
        game_write(rva(0x60933C), 0x20 + seed % 0x30);
        game_write(rva(0x609338), seed.wrapping_mul(0x37) & 0xFF);
    }
}
