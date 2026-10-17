//! `drive::animate_lit_faces` (0x45EB06): one step of the stage's moving faces, those in
//! `g_litFaces` (0x6D3220, `g_litFaceCount` of them). For each (face record at
//! `g_courseFaces` + 0x40 n):
//! - object 0x3B (byte +1): a countdown at +0x11; at 0 the face leaves the list;
//! - else its distance from car block 0 (+0x134 / +0x13C: `dword_70F520`, `dword_70F528`;
//!   `fistp` of the root of the squares, through `dword_62400C`) into `dword_6D303C`, and
//!   a pace 1 .. 17 from it (closer is faster) into `dword_6D3040`; then by its object's
//!   kind (`byte_97DF4C[g_objectKinds[object] * 0x80]`):
//!   - 1, 9: +0x11 counts up to 0x18 (then cleared, and the face leaves), +0x13 from
//!     `byte_6D1DFC`;
//!   - 2: +0x13 up by the pace, leaving at 0x30;
//!   - 3, 8: +9 stepped (the pace times 4) towards making +8 + +9 + +0x12 a multiple of
//!     0x80; once it is, as 2 up to 0x40;
//!   - 4: while +0x11 is not 0, the face's offset (+0x30, +0x38) moved by
//!     `math::turn_point_xz` of (0, +0x11 << 8) by (+0x12 + 0x80) << 24, its height
//!     (+0x34) set on the ground under it (`physics::find_ground`, last face
//!     `dword_7101EC`; no ground puts the offset back), +0x11 less 1; at 0 it leaves;
//!   - others: nothing.
//!
//! A face leaves by the list's last taking its place (the count less 1), and the next
//! one is looked at (so the one moved in waits a step). By the look of it the stage's
//! animated objects: lights blinking, things falling (a reading).
//!
//! In:  nothing
//! Out: the registers as the last face leaves them (see `animate_at`); no flags read
//!      (`sub_45E9CA` calls `physics::move_slots` next)
//!
//! Caller: `drive::animate_stage` (0x45E9CA).
//! Refuses (before anything) more than 0x100 lit faces (the list's room) and a face
//! number past `g_courseFaceCount`; `find_ground`'s refusals mid-way (what it wrote is
//! put back first).
//! Self-test: in a race only: a copy of the original (its calls the game's) against the
//! Rust version on the game's own stage, with a list of the test's (twelve faces of the
//! stage, their objects and bytes by lot); every game variable it writes saved and put
//! back.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write,
    hook_asm, rva,
};
use crate::physics::ground::Ground;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::animate_lit_faces", rva(0x45EB06), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LIST_RVA: usize = rva(0x6D3220); // g_litFaces
const COUNT_RVA: usize = rva(0x6D3620); // g_litFaceCount
const MOST: u32 = 0x100;
const FACES_RVA: usize = rva(0x5CCCBC); // g_courseFaces
const FACE_COUNT_RVA: usize = rva(0x5CCD1C); // g_courseFaceCount
const POINTS_RVA: usize = rva(0x5CCCC0); // g_scenePoints
const OBJECT_KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
const KIND_TABLE_RVA: usize = rva(0x97DF4C); // byte_97DF4C, by kind * 0x80
const BLINK_RVA: usize = rva(0x6D1DFC); // byte_6D1DFC
const LEAD_X_RVA: usize = rva(0x70F520); // car block 0 +0x134
const LEAD_Z_RVA: usize = rva(0x70F528); // car block 0 +0x13C

unsafe fn at(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

unsafe fn put(p: u32, v: u32) {
    unsafe { (p as *mut u32).write_unaligned(v) }
}

unsafe fn byte(p: u32) -> u8 {
    unsafe { (p as *const u8).read() }
}

unsafe fn set_byte(p: u32, v: u8) {
    unsafe { (p as *mut u8).write(v) }
}

fn list(k: u32) -> usize {
    LIST_RVA + 4 * k as usize
}

/// The faces' records it may change (all those listed now).
unsafe fn face_regions() -> Vec<Region> {
    unsafe {
        let count = game_read::<u32>(COUNT_RVA).min(MOST);
        let faces: u32 = game_read(FACES_RVA);
        (0..count)
            .map(|k| (faces.wrapping_add(game_read::<u32>(list(k)) << 6), 0x40))
            .collect()
    }
}

/// Everything it writes.
pub fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut r = Ground::game().regions();
    r.extend([
        (game_addr(LIST_RVA) as u32, 4 * MOST as usize),
        (game_addr(COUNT_RVA) as u32, 4),
        (a(0x62400C), 4),
        (a(0x6D302C), 0x18),
        (a(0x5C5F4C), 0x0C),
        (a(0x620470), 4),
    ]);
    r.extend(unsafe { face_regions() });
    r
}

/// The refusals, worked out before anything is written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if count > MOST {
            return Some("more than 0x100 lit faces (past g_litFaces)");
        }
        let faces: u32 = game_read(FACE_COUNT_RVA);
        for k in 0..count {
            if game_read::<u32>(list(k)) >= faces {
                return Some("a lit face past g_courseFaceCount");
            }
        }
        None
    }
}

/// The Rust core; `r` as the original leaves it.
pub unsafe fn animate_at(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal() {
            return Err(BadInput(why).into());
        }
        if game_read::<u32>(COUNT_RVA) == 0 {
            return Ok(());
        }
        let all = regions();
        let saved = capture(&all);
        let result = animate_faces(r);
        if result.is_err() {
            restore(&all, &saved);
        }
        result
    }
}

unsafe fn animate_faces(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let faces: u32 = game_read(FACES_RVA);
        let points: u32 = game_read(POINTS_RVA);
        let point = |n: u32| points.wrapping_add(n.wrapping_mul(12));
        r.ecx = 0;
        loop {
            r.ebx = game_read(list(r.ecx));
            let f = faces.wrapping_add(r.ebx << 6);
            r.esi = f;
            let leave = 'face: {
                let object = byte(f + 1);
                if object == 0x3B {
                    let n = byte(f + 0x11).wrapping_sub(1);
                    set_byte(f + 0x11, n);
                    break 'face n == 0;
                }
                r.ebx = at(f + 4);
                let p = point(r.ebx);
                let dx = at(p).wrapping_sub(game_read(LEAD_X_RVA));
                game_write(rva(0x62400C), dx);
                let dz = at(p + 8).wrapping_sub(game_read(LEAD_Z_RVA));
                game_write(rva(0x62400C), dz);
                let d = crate::math::x87::int_hypot(dx as i32, dz as i32) as u32;
                game_write(rva(0x6D303C), d);
                r.edx = if d > 0x2_0000 {
                    1
                } else {
                    ((0x2_0000 - d) >> 13) + 1
                };
                game_write(rva(0x6D3040), r.edx);
                let pace = r.edx;
                let kind_of: u32 = game_read(OBJECT_KINDS_RVA + 4 * object as usize);
                r.eax = u32::from(game_read::<u8>(
                    KIND_TABLE_RVA + (kind_of.wrapping_mul(0x80)) as usize,
                ));
                match r.eax {
                    1 | 9 => {
                        let n = byte(f + 0x11).wrapping_add(1);
                        set_byte(f + 0x11, n);
                        if n >= 0x18 {
                            set_byte(f + 0x11, 0);
                            set_byte(f + 0x13, 0);
                            break 'face true;
                        }
                        r.eax = u32::from(game_read::<u8>(BLINK_RVA + n as usize));
                        set_byte(f + 0x13, r.eax as u8);
                        false
                    }
                    2 => {
                        r.edx = pace;
                        loop {
                            if byte(f + 0x13) == 0x30 {
                                break 'face true;
                            }
                            set_byte(f + 0x13, byte(f + 0x13).wrapping_add(1));
                            r.edx -= 1;
                            if r.edx == 0 {
                                break 'face false;
                            }
                        }
                    }
                    3 | 8 => {
                        r.edx = pace << 2;
                        loop {
                            let al = byte(f + 9)
                                .wrapping_add(byte(f + 8))
                                .wrapping_add(byte(f + 0x12))
                                & 0x7F;
                            r.eax = (r.eax & !0xFF) | u32::from(al);
                            if al == 0 {
                                break;
                            }
                            let mut b9 = byte(f + 9).wrapping_sub(1);
                            if al >= 0x40 {
                                b9 = b9.wrapping_add(2);
                            }
                            set_byte(f + 9, b9);
                            r.edx -= 1;
                            if r.edx == 0 {
                                break 'face false;
                            }
                        }
                        r.edx = pace;
                        loop {
                            if byte(f + 0x13) == 0x40 {
                                break 'face true;
                            }
                            set_byte(f + 0x13, byte(f + 0x13).wrapping_add(1));
                            r.edx -= 1;
                            if r.edx == 0 {
                                break 'face false;
                            }
                        }
                    }
                    4 => {
                        if byte(f + 0x11) == 0 {
                            break 'face true;
                        }
                        let ecx = u32::from(byte(f + 0x11)) << 8;
                        let edx = (u32::from(byte(f + 0x12)) + 0x80) << 24;
                        let [x, _, z, _] = crate::math::turn_point_xz::turn_point_xz(
                            0,
                            r.ebx as i32,
                            ecx as i32,
                            edx,
                        );
                        game_write(rva(0x6D302C), at(f + 0x30));
                        game_write(rva(0x6D3030), at(f + 0x38));
                        put(f + 0x30, at(f + 0x30).wrapping_add(x));
                        put(f + 0x38, at(f + 0x38).wrapping_add(z));
                        let p = point(at(f + 4));
                        game_write(rva(0x639824), at(p).wrapping_add(at(f + 0x30)));
                        game_write(rva(0x63982C), at(p + 8).wrapping_add(at(f + 0x38)));
                        game_write(rva(0x5CCC78), game_read::<u32>(rva(0x7101EC)));
                        crate::physics::find_ground::find_ground()?;
                        put(
                            f + 0x34,
                            game_read::<u32>(rva(0x639828)).wrapping_sub(at(p + 4)),
                        );
                        if game_read::<u32>(rva(0x5CCC74)) == u32::MAX {
                            put(f + 0x30, game_read(rva(0x6D302C)));
                            put(f + 0x38, game_read(rva(0x6D3030)));
                        }
                        set_byte(f + 0x11, byte(f + 0x11).wrapping_sub(1));
                        false
                    }
                    _ => false,
                }
            };
            if leave {
                let count: u32 = game_read(COUNT_RVA);
                r.ebx = count.wrapping_sub(1);
                r.eax = game_read(list(r.ebx));
                game_write(list(r.ecx), r.eax);
                game_write(COUNT_RVA, r.ebx);
            }
            r.ecx += 1;
            if r.ecx >= game_read::<u32>(COUNT_RVA) {
                return Ok(());
            }
        }
    }
}

/// The Rust version: `r` as the original leaves it.
pub unsafe fn animate_lit_faces(r: &mut Regs) -> Result<(), Refused> {
    unsafe { animate_at(r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { animate_lit_faces(&mut g) }?;
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::physics::move_slot::in_race() {
        return t.skip("not in a race (no stage's faces)");
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45EB06,
            len: 0x26A,
            entry: &[0x83, 0x3D, 0x20, 0x36, 0x6D, 0x00, 0x00],
            calls: &[0x1BC, 0x20D],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let face_count: u32 = unsafe { game_read(FACE_COUNT_RVA) };
    let faces: u32 = unsafe { game_read(FACES_RVA) };
    // The faces the test lists: twelve of the stage's, spread out.
    let chosen: Vec<u32> = (0..12u32)
        .map(|k| (k * 7919 + 13) % face_count.max(1))
        .collect();
    let set_list = |seed: u32| unsafe {
        let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
        let mut next = move || {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            x >> 8
        };
        game_write(COUNT_RVA, chosen.len() as u32);
        for (k, &n) in chosen.iter().enumerate() {
            game_write(list(k as u32), n);
            let f = faces + (n << 6);
            let mut object = if next() % 4 == 0 {
                0x3B
            } else {
                (next() % 0x60) as u8
            };
            // Only objects whose kind is a small number (the kind table's rows).
            if game_read::<u32>(OBJECT_KINDS_RVA + 4 * object as usize) >= 0x40 {
                object = 0x3B;
            }
            set_byte(f + 1, object);
            for o in [8u32, 9, 0x11, 0x12, 0x13] {
                set_byte(f + o, (next() % 0x50) as u8);
            }
        }
    };
    // What the cases change: the list, the chosen faces, and the rest it writes.
    let mut all = regions();
    all.extend(chosen.iter().map(|&n| (faces + (n << 6), 0x40)));
    let saved = capture(&all);
    for seed in 1..=8u32 {
        set_list(seed);
        let input = capture(&all);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        restore(&all, &input);
        let mut g = Regs::SENTINEL;
        let mine = unsafe { animate_at(&mut g) };
        let state = capture(&all);
        restore(&all, &saved);
        let regs_ok = mine.is_ok()
            && Left::nothing()
                .with(EAX, g.eax)
                .with(EBX, g.ebx)
                .with(ECX, g.ecx)
                .with(EDX, g.edx)
                .with(ESI, g.esi)
                .matches(&Regs::SENTINEL, &ro);
        t.check(regs_ok && theirs == state, || {
            let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
            format!("seed {seed}: regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}")
        });
    }
    drop(copy);
    t.finish()
}
