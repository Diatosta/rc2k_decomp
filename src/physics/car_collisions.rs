//! `physics::car_collisions` (0x45F615): the work car against the other cars, in race
//! modes 0 and 2 (`dword_62389C`; else nothing). `dword_6D3BB4` = 0; for each car n
//! below `g_raceCars` but the current one (`g_currentCar`), whose block
//! (`g_carBlocks[n]`, into `dword_6D3BB8`, n into `dword_6D3CD4`) has +0x25C set: the
//! places' difference as the place (`scene::set_place`), `g_crashMatrix` x that into
//! `g_turnedMatrix`, that x the block's matrix (+0x9CC) into `flt_6D3C40`
//! (`scene::multiply_matrices`); then for its eight corners (`dword_6D3BE0[k]`,
//! `dword_6D3C00[k]`, `dword_6D3C20[k]` into `dword_6D3BD4` ..) `physics::corner_hits_car`.
//! Every step in `pusha` / `popa`: only ebx, the count, is left.
//!
//! In:  nothing
//! Out: ebx the cars' count (at least 1) in modes 0 and 2; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`.
//! Refuses (before anything) more than eight cars or a null block among them (the
//! original reads past `g_carBlocks` or through null), and what `corner_hits_car`
//! refuses (what was written put back).
//! Self-test (`test_bump`): the original and the Rust version on the game's variables
//! (saved and put back) with three test car blocks (`g_carBlocks` pointed at them), the
//! modes, every current car, places close enough for corners to meet, under three
//! precisions.

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::corner_hits_car::{corner_hits_car, corner_inside};
use crate::physics::flow::{a, g, guard, p};
use crate::scene::matrix::{multiply_matrices, set_place};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{
    Region, Rust, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::car_collisions", rva(0x45F615), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BLOCKS: usize = 0x71B3B8; // g_carBlocks (eight)
const CARS: usize = 0x63DB90; // g_raceCars
const CURRENT: usize = 0x71B3F8; // g_currentCar

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The cars looked at: `g_raceCars`, at least one (the loop runs once).
fn cars() -> u32 {
    g(CARS).max(1)
}

/// The Rust version.
pub unsafe fn car_collisions(entry: &Regs) -> Result<Left, Refused> {
    if !matches!(g(0x62389C), 0 | 2) {
        return Ok(Left::nothing());
    }
    let n = cars();
    if n > 8 {
        return Err(BadInput("more than eight cars (g_carBlocks' room)").into());
    }
    if (0..n).any(|k| k != g(CURRENT) && g(BLOCKS + 4 * k as usize) == 0) {
        return Err(BadInput("a null car block").into());
    }
    guard(&regions(), || {
        p(0x6D3BB4, 0);
        for car in 0..n {
            if car == g(CURRENT) {
                continue;
            }
            p(0x6D3CD4, car);
            let block = g(BLOCKS + 4 * car as usize);
            p(0x6D3BB8, block);
            if rd(block + 0x25C) == 0 {
                continue;
            }
            unsafe {
                set_place(
                    [0, 4, 8].map(|o| g(0x71A10C + o).wrapping_sub(rd(block + 0x134 + o as u32))),
                );
                multiply_matrices(
                    a(0x71A08C) as usize,
                    a(0x5EA04C) as usize,
                    a(0x5EA10C) as usize,
                );
                multiply_matrices(
                    a(0x5EA10C) as usize,
                    (block + 0x9CC) as usize,
                    a(0x6D3C40) as usize,
                );
            }
            for k in 0..8 {
                p(0x6D3BD4, g(0x6D3BE0 + 4 * k));
                p(0x6D3BD8, g(0x6D3C00 + 4 * k));
                p(0x6D3BDC, g(0x6D3C20 + 4 * k));
                unsafe {
                    corner_hits_car(&Regs {
                        edi: block,
                        ..*entry
                    })
                }?;
            }
        }
        Ok(Left::nothing().with(EBX, n))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { car_collisions(r) }
}

/// Everything the collisions and their calls write.
pub fn regions() -> Vec<Region> {
    let mut all = vec![
        (a(0x6D3090), 12),
        (a(0x6D3B88), 0x6D3CE4 - 0x6D3B88),
        (a(0x614B10), 12),
        (a(0x71A20C), 4),
        (a(0x71A170), 12),
        (a(0x71A17C), 12),
        (a(0x62400C), 4),
        (a(0x5EA04C), 0x40),
        (a(0x5EA10C), 0x40),
        (a(0x6D1AD4), 4),
        // `restore_saved_car`'s
        (a(0x614738), 4),
        (a(0x71A10C), 12),
        (a(0x71A08C), 0x40),
        (a(0x71ADC8), 0x1F0),
    ];
    all.extend(crate::physics::crash_contact::sound_regions());
    for k in 0..g(CARS).clamp(1, 8) {
        let block = g(BLOCKS + 4 * k as usize);
        if block != 0 {
            all.extend([(block + 0x198, 0x18), (block + 0x234, 4)]);
        }
    }
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The tests of the three: `whole` the original's `call_original`, the Rust version, and
/// which of them (`corner`: `corner_hits_car`, entered with a block in edi; neither:
/// `bump_cars`, which the corner set up; `car_collisions` by its hook's name).
pub fn test_bump(
    hook: &RawHook,
    call_original: CallOriginal,
    rust: Rust,
    corner: bool,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let whole = std::ptr::eq(hook, &HOOK);
    let car = crate::physics::car_sounds::CURRENT;
    let mut all = regions();
    all.extend([
        (a(BLOCKS), 0x20),
        (a(CARS), 4),
        (a(car), 4),
        (a(crate::physics::car_sounds::LISTED), 0x20),
        (a(0x62389C), 4),
        (a(0x71A238), 4),
        (a(0x6141E0), 0x50), // the saved place and matrix
        (a(0x71A9A4), 0x40), // the work car's matrix (+0x9CC)
        (a(0x6D3BE0), 0x60), // the corners
    ]);
    let saved = capture(&all);
    let mut blocks = vec![0u32; 3 * 0x3F8];
    let base = blocks.as_mut_ptr() as u32;
    let block = |k: u32| base + 0xFE0 * k;
    all.push((base, 3 * 0xFE0));
    let mut inside = 0;
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                let set = || {
                    let mut f = vec![0u32; 3 * 0x3F8 + 0x100];
                    test_floats(seed * 17 + 9, &mut f);
                    for (k, v) in f.iter().take(3 * 0x3F8).enumerate() {
                        unsafe { ((base + 4 * k as u32) as *mut u32).write(*v) };
                    }
                    let extra = &f[3 * 0x3F8..];
                    p(CARS, 3);
                    p(car, seed % 3);
                    // `restore_saved_car`'s count (`dword_614738`): the same start for both.
                    p(0x614738, seed);
                    for k in 0..8 {
                        p(crate::physics::car_sounds::LISTED + 4 * k, u32::MAX);
                    }
                    p(0x62389C, [0u32, 2, 0, 1][(seed % 4) as usize]);
                    p(0x71A238, 4);
                    for o in 0..3 {
                        p(
                            0x71A10C + 4 * o,
                            (extra[0x68 + o] as i32 % 0x10_0000) as u32,
                        );
                    }
                    for k in 0..3u32 {
                        let b = block(k);
                        p(BLOCKS + 4 * k as usize, b);
                        unsafe {
                            ((b + 0x25C) as *mut u32).write((seed + k) % 4);
                            for o in 0..3u32 {
                                let near = g(0x71A10C + 4 * o as usize)
                                    .wrapping_add((extra[o as usize] as i32 % 0x20000) as u32);
                                ((b + 0x134 + 4 * o) as *mut u32).write(near);
                            }
                        }
                    }
                    for k in 0..0x18 {
                        // The corners, small.
                        p(
                            0x6D3BE0 + 4 * k,
                            (f32::from_bits(extra[8 + k]) % 2.0).to_bits(),
                        );
                    }
                    for k in 0..3 {
                        p(0x6D3BBC + 4 * k, (2.0 + (seed % 3) as f32).to_bits());
                    }
                    for k in 0..4 {
                        p(0x6D3B88 + 4 * k, extra[0x20 + k]);
                        p(0x6D3BAC + 4 * k, extra[0x24 + k]);
                    }
                    for k in 0..0x10 {
                        p(0x71A08C + 4 * k, extra[0x30 + k]);
                        p(0x71A9A4 + 4 * k, extra[0x40 + k]);
                        p(0x6141E0 + 4 * k, extra[0x50 + k]);
                    }
                    for k in 0..3 {
                        p(0x71A170 + 4 * k, extra[0x60 + k]);
                    }
                    if !whole {
                        // One corner of car 1, set up as `car_collisions` would.
                        let b = block(1);
                        p(0x6D3CD4, 1);
                        p(0x6D3BB8, b);
                        for k in 0..0x10 {
                            p(0x6D3C40 + 4 * k, extra[0x70 + k]);
                        }
                        if seed % 2 == 0 {
                            // An unturned, unmoved car: its corners (below 2) fall inside
                            // the box (2 .. 4), so the inside branch is reached.
                            for k in 0..0x10 {
                                let one = [0, 5, 10].contains(&k);
                                p(0x6D3C40 + 4 * k, if one { 1f32.to_bits() } else { 0 });
                            }
                        }
                        for k in 0..3 {
                            p(
                                0x6D3BD4 + 4 * k,
                                (f32::from_bits(extra[0x80 + k]) % 2.0).to_bits(),
                            );
                        }
                        if !corner {
                            for k in 0..3 {
                                p(0x6D3BC8 + 4 * k, extra[0x84 + k]);
                            }
                        }
                    }
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs {
                        edi: block(1),
                        ..Regs::SENTINEL
                    },
                    &set,
                    &all,
                    rust,
                );
                if corner {
                    set();
                    inside += corner_inside() as u32;
                }
            }
        });
    }
    if corner {
        t.check(inside > 0, || "no corner inside".into());
    }
    all.pop();
    restore(&all, &saved);
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_bump(&HOOK, call_original, &rust, false)
}
