//! `race::update_mud_texture` (0x44A12E): the mud thrown and splashed since the last turn
//! put onto the texture worked on (`dword_623D5C`, n). The texture's slot (kind 0x100's
//! first slot + n) forgotten (`scene::forget_slot_texture`), `dword_6D1508` + 0x10000; the
//! texture (kind 0x100's first texture + n, into `dword_623D54`; none: nothing more)
//! locked (`d3d::lock_texture`, the platform's; carry: nothing more). Then:
//! - the throws: `dword_623D60[n]` (the wheels' thrown mud, a float) rounded (`fistp`)
//!   into `dword_623920`, which read back as a float (sic) and below 0.0 becomes 0.0's
//!   bits: negative counts are 0. Unless the texture's count `dword_623D80[n]` is past
//!   0x60000: held so that its low word plus the throws stays below 0x1000000, taken off
//!   `dword_623D60[n]` (`fisub`), and above 0 that many `race::throw_mud_texel`;
//! - the splashes: the same from `dword_623D70[n]` into `dword_623924`, unless the count
//!   `dword_623D90[n]` has reached 0x1000 (also checked before each): `race::splash_mud`;
//!
//! Then the texture is unlocked (`d3d::unlock_texture`).
//!
//! In:  nothing
//! Out: the registers as the unlock leaves them (or the lock, or `forget_slot_texture`,
//!      eax the texture, on the early ends); no flags read (`race::mud_texture_turn`
//!      returns)
//!
//! See `race/mud.rs`. One caller, `race::mud_texture_turn`. Its calls through `part`: 0
//! the forgetting (eax), 1 the lock (eax; carry), 2 the unlock (eax); in the game the
//! forgetting's Rust version and the platform's two through the game (section 15).
//! Refuses (before anything) a texture past 3 and no mud, and its callees' refusals (what
//! the counts, the throws and the random state were put back). Shadow mode runs the
//! original alone (it tells the device). Self-test: a copy whose three calls go to
//! stand-ins (the lock failing or not), against the Rust version, on the mud tests'
//! set-up with patterns of the test's, throws and splashes of each sign and size, under
//! the three precisions; the surface, the counts, the floats and the random state compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::physics::flow::{base, c0, x87_run};
use crate::race::muddy_texel::texture_and_mud;
use crate::race::splash_mud::{LAST_SPLASHED, SPLASHED_RVA, splash_mud, splash_regions};
use crate::race::throw_mud_texel::{LAST_THROWN, THROWN_RVA, throw_mud_texel, throw_regions};
use crate::service::harness::at;

const FIRST_SLOT_RVA: usize = rva(0x5D1EE8); // g_kindFirstSlots[0x100]
const FIRST_TEXTURE_RVA: usize = rva(0x5D0EE8); // g_kindFirstTextures[0x100]
const CHANGES_RVA: usize = rva(0x6D1508); // dword_6D1508
const LOCKED_RVA: usize = rva(0x623D54); // dword_623D54: the texture locked
const THROWS_IN_RVA: usize = rva(0x623D60); // dword_623D60: each texture's thrown mud (float)
const SPLASHES_IN_RVA: usize = rva(0x623D70); // dword_623D70: each texture's splashes (float)
const THROWS_RVA: usize = rva(0x623920); // dword_623920
const SPLASHES_RVA: usize = rva(0x623924); // dword_623924
const ZERO_RVA: usize = rva(0x5C5D90); // flt_5C5D90: 0.0
const LOCK_RVA: usize = rva(0x41AA4B); // d3d::lock_texture (the platform's)
const UNLOCK_RVA: usize = rva(0x41AB02); // d3d::unlock_texture (the platform's)

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `fld [from]; fistp [to]`, then `fld zero; fld [to]; fcompp` and, with C0, `fld zero;
/// fstp [to]`: the units of a float count, negatives 0 (as the original, by bits).
fn whole_units(from: usize, to: usize) -> i32 {
    x87_run!(
        "fld dword ptr [{b} + {f}]",
        "fistp dword ptr [{b} + {t}]";
        b = base(), f = from as u32, t = to as u32
    );
    let zero: u32 = unsafe { game_read(ZERO_RVA) };
    if c0(unsafe { game_read(to) }, zero) {
        unsafe { game_write(to, zero) };
    }
    unsafe { game_read(to) }
}

/// `fld [at]; fisub [units]; fstp [at]`.
fn take_units(at: usize, units: usize) {
    x87_run!(
        "fld dword ptr [{b} + {a}]",
        "fisub dword ptr [{b} + {u}]",
        "fstp dword ptr [{b} + {a}]";
        b = base(), a = at as u32, u = units as u32
    );
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn update_mud_texture(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (n, _) = texture_and_mud()?;
    let k = 4 * n as usize;
    let _all = [
        &throw_regions()[..],
        &splash_regions()[..],
        &[
            at(CHANGES_RVA),
            at(LOCKED_RVA),
            (game_addr(THROWS_IN_RVA) as u32, 0x20),
            (game_addr(THROWS_RVA) as u32, 8),
        ],
    ]
    .concat();
    (|| -> Result<_, crate::errors::Refused> {
        unsafe {
            let mut r = *entry;
            r.eax = game_read::<u32>(FIRST_SLOT_RVA).wrapping_add(n);
            part(0, &mut r)?;
            game_write(
                CHANGES_RVA,
                game_read::<u32>(CHANGES_RVA).wrapping_add(0x1_0000),
            );
            r.eax = game_read::<u32>(FIRST_TEXTURE_RVA).wrapping_add(n);
            game_write(LOCKED_RVA, r.eax);
            if r.eax == 0 {
                return Ok(r);
            }
            part(1, &mut r)?;
            if r.carry() {
                return Ok(r);
            }
            // The throws.
            r.ebx = n;
            let throws = whole_units(THROWS_IN_RVA + k, THROWS_RVA);
            let count: u32 = game_read(THROWN_RVA + k);
            r.eax = count;
            if count <= LAST_THROWN {
                r.eax = (count & 0xFFFF).wrapping_add(throws as u32);
                if (r.eax.wrapping_sub(0x100_0000) as i32) >= 0 {
                    r.eax = r.eax.wrapping_sub(0x100_0000);
                    game_write(THROWS_RVA, (throws as u32).wrapping_sub(r.eax));
                }
                take_units(THROWS_IN_RVA + k, THROWS_RVA);
                let throws: i32 = game_read(THROWS_RVA);
                if throws > 0 {
                    let mut ebp = 0u32;
                    loop {
                        r = throw_mud_texel(&Regs { ebp, ..r })?;
                        ebp = ebp.wrapping_add(1);
                        r.ebp = ebp;
                        if ebp == game_read::<u32>(THROWS_RVA) {
                            break;
                        }
                    }
                }
            }
            // The splashes.
            r.ebx = n;
            whole_units(SPLASHES_IN_RVA + k, SPLASHES_RVA);
            r.eax = game_read(SPLASHED_RVA + k);
            if r.eax < LAST_SPLASHED {
                take_units(SPLASHES_IN_RVA + k, SPLASHES_RVA);
                let splashes: i32 = game_read(SPLASHES_RVA);
                if splashes > 0 {
                    let mut ebp = 0u32;
                    r.ebp = ebp;
                    loop {
                        r.ebx = n;
                        r.eax = game_read(SPLASHED_RVA + k);
                        if r.eax >= LAST_SPLASHED {
                            break;
                        }
                        r = splash_mud(&Regs { ebp, ..r })?;
                        ebp = ebp.wrapping_add(1);
                        r.ebp = ebp;
                        if ebp == game_read::<u32>(SPLASHES_RVA) {
                            break;
                        }
                    }
                }
            }
            r.eax = game_read(LOCKED_RVA);
            part(2, &mut r)?;
            Ok(r)
        }
    })()
}

/// In the game: the forgetting's Rust version, the platform's lock and unlock through the
/// game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => crate::scene::forget_slot_texture::forget_slot_texture(r.eax)?,
            1 => crate::calls::call(LOCK_RVA, r),
            _ => crate::calls::call(UNLOCK_RVA, r),
        }
    }
    Ok(())
}

// mov eax, g_kindFirstSlots+400h
// forget, lock, throw, splash, unlock
