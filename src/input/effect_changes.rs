//! The six force-feedback effects' functions (+0 of each effect of an input device's
//! record: `input::effect_0_change` 0x409708 for the first effect, `effect_1_change`
//! 0x4097FA, `effect_2_change` 0x409754, `effect_3_change` 0x4097B4, `effect_4_change`
//! 0x409871, `effect_5_change` 0x4098BF; both records hold the same six), which
//! `input::device_effects` calls with a change waiting: eax the change (1 set up, 2 start,
//! 3 values, 4 stop), ecx and edx its two values (signed), esi the device's record, edi the
//! effect (0xA4 bytes; +0x30 its DirectInput effect, +0x34 its parameters).
//! A value v below is `(v x 10000, as a signed dword) >> 8` (arithmetic).
//! - effect 0: 2 start; 3 +0x6C and +0x70 ecx's, +0x68 edx's, the parameters set with
//!   flags 0x100; 4 stop.
//! - effect 1: 1 +0x68 0; 2 start; 3 by the row ecx of `unk_4F6E08` (four dwords each): +0x68
//!   the row's first two blended by edx (`(b - a) x edx >> 8 + a`, then as a value), +0x74
//!   10000000 / the next two blended likewise (`idiv`); flags 0x100; 4 stop.
//! - effect 2: 1 +0x3C 500000, +0x5C the effect's +0x90, +0xA0 500000, +0x74 100000, +0x68
//!   0; 3 +0x68 ecx's, flags 0x20000100; 4 stop.
//! - effect 3: 1 +0x3C 250000, +0x68 0; 3 +0x68 ecx's, flags 0x20000100; 4 stop.
//! - effect 4: 1 +0x6C and +0x70 0; 2 start; 3 +0x6C and +0x70 ecx's, flags 0x100; 4 stop.
//! - effect 5: 1 +0x3C 200000, +0x80 4, +0x5C the effect's +0x90, +0xA0 200000, +0x68 0, and
//!   with the record's +0x20 above 1 (unsigned) +0xC 0; 3 +0x68 ecx's, flags 0x20000100; 4
//!   stop.
//!
//! Other changes do nothing.
//!
//! In:  as above
//! Out: every register kept (`pusha` .. `popa`)
//! The DirectInput calls (`sub_409AEE` the effect's Download and Start, `sub_409B1F` its
//! SetParameters with eax the flags, `sub_409B0D` its Stop; each only with an effect at
//! +0x30) go through `part` (0, 1, 2), in the game through the game (section 15). Refuses
//! (before anything) for effect 1 a row past the table's 14 and a blend of 0 (the
//! original's `idiv` traps). Shadow mode runs the original alone (DirectInput). Manual
//! patches with private copies (a `jnz` in the first bytes). Self-test: a copy whose calls
//! go to stand-ins, against the Rust version, each change, values of both signs.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};

const START: usize = 0x409AEE; // sub_409AEE
const SET: usize = 0x409B1F; // sub_409B1F
const STOP: usize = 0x409B0D; // sub_409B0D
const CALLEES: [usize; 3] = [START, SET, STOP];
const ROWS_RVA: usize = rva(0x4F6E08); // unk_4F6E08
const ROWS: i32 = 14;

/// One effect's function, as IDA shows it.
#[derive(Clone, Copy)]
pub struct Effect {
    pub number: u32,
    pub address: usize,
    pub len: usize,
    pub entry: [u8; 6],
    pub calls: &'static [usize],
}

fn put(effect: u32, o: u32, v: u32) {
    unsafe { ((effect + o) as *mut u32).write_unaligned(v) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}
fn value(v: u32) -> u32 {
    ((v as i32).wrapping_mul(10000) >> 8) as u32
}
fn blend(a: u32, b: u32, by: u32) -> u32 {
    (((b.wrapping_sub(a) as i32).wrapping_mul(by as i32)) >> 8) as u32
}

/// The Rust version: the registers it leaves.
pub unsafe fn effect_change(entry: &Regs, number: u32, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let e = entry.edi;
    let (ecx, edx) = (entry.ecx, entry.edx);
    match (entry.eax, number) {
        (1, 1) => put(e, 0x68, 0),
        (1, 2 | 3 | 5) => {
            let long = [0, 0, 500_000, 250_000, 0, 200_000][number as usize];
            put(e, 0x3C, long);
            if number == 5 {
                put(e, 0x80, 4);
            }
            if number != 3 {
                put(e, 0x5C, e + 0x90);
                put(e, 0xA0, long);
            }
            if number == 2 {
                put(e, 0x74, 100_000);
            }
            put(e, 0x68, 0);
            if number == 5 && get(entry.esi + 0x20) > 1 {
                put(e, 0xC, 0);
            }
        }
        (1, 4) => {
            put(e, 0x6C, 0);
            put(e, 0x70, 0);
        }
        (2, 0 | 1 | 4) => part(0, &mut r)?,
        (3, 0) => {
            put(e, 0x6C, value(ecx));
            put(e, 0x70, value(ecx));
            put(e, 0x68, value(edx));
            r.eax = 0x100;
            part(1, &mut r)?;
        }
        (3, 1) => {
            if !(0..ROWS).contains(&(ecx as i32)) {
                return Err(BadInput("a row past the effect's table").into());
            }
            let row = game_addr(ROWS_RVA) as u32 + 16 * ecx;
            let first = blend(get(row), get(row + 8), edx).wrapping_add(get(row));
            let second = blend(get(row + 4), get(row + 12), edx).wrapping_add(get(row + 4));
            if second == 0 {
                return Err(BadInput("a blend of 0 (the original's idiv traps)").into());
            }
            put(e, 0x68, value(first));
            put(e, 0x74, (10_000_000i32 / second as i32) as u32);
            // The registers at the call: the row, the divisor and the `idiv`'s remainder.
            r.ebx = row;
            r.ecx = second;
            r.edx = (10_000_000i32 % second as i32) as u32;
            r.eax = 0x100;
            part(1, &mut r)?;
        }
        (3, 2 | 3 | 5) => {
            put(e, 0x68, value(ecx));
            r.eax = 0x2000_0100;
            part(1, &mut r)?;
        }
        (3, 4) => {
            put(e, 0x6C, value(ecx));
            put(e, 0x70, value(ecx));
            r.eax = 0x100;
            part(1, &mut r)?;
        }
        (4, _) => part(2, &mut r)?,
        _ => {}
    }
    Ok(*entry)
}

/// In the game: DirectInput's calls through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { call_game(r, game_addr(rva(CALLEES[place as usize]))) };
    Ok(())
}

pub fn dispatch_effect(
    hook: &RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    number: u32,
) {
    dispatch_parted(
        hook,
        r,
        call_original,
        &|r, part| unsafe { effect_change(r, number, part) },
        game_part,
    );
}

pub fn test_effect(hook: &RawHook, e: Effect) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match copy(e.address, e.len, &e.entry, e.calls, &CALLEES, &[]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut effect = [0u8; 0xA4];
    let mut record = [0u8; 0x40];
    let (at_effect, at_record) = (effect.as_mut_ptr() as u32, record.as_mut_ptr() as u32);
    for change in 0..=5u32 {
        for (ecx, edx, kind) in [
            (0u32, 0u32, 0u32),
            (3, 0x80, 1),
            (0xFFFF_FF80, 0x7F, 2),
            (13, 0xFFFF_FFFF, 3),
            (0x7FF, 0x1FF, 1),
        ] {
            if e.number == 1 && change == 3 {
                // Only rows of the table, and blends that do not divide by 0.
                let row = game_addr(ROWS_RVA) as u32 + 16 * ecx;
                if ecx as i32 >= ROWS || ecx as i32 <= -1 {
                    continue;
                }
                if blend(get(row + 4), get(row + 12), edx).wrapping_add(get(row + 4)) == 0 {
                    continue;
                }
            }
            run_both(
                &mut t,
                &format!("change {change}, ecx {ecx:#x}, edx {edx:#x}, device kind {kind}"),
                &copy,
                &[(at_effect, 0xA4), (at_record, 0x40)],
                &|| unsafe {
                    for k in 0..0xA4u32 {
                        ((at_effect + k) as *mut u8).write((k * 7 + change) as u8);
                    }
                    std::ptr::write_bytes(at_record as *mut u8, 0x33, 0x40);
                    ((at_record + 0x20) as *mut u32).write_unaligned(kind);
                },
                Regs {
                    eax: change,
                    ecx,
                    edx,
                    esi: at_record,
                    edi: at_effect,
                    ..Regs::SENTINEL
                },
                false,
                &|r| unsafe { effect_change(r, e.number, &mut test_part) },
            );
        }
    }
    let _ = (&mut effect, &mut record);
    t.finish()
}
