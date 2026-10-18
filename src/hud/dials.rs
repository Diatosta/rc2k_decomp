//! The HUD's dials' set-ups (`hud::disc_*`, `hud::needle_*`): each copies fields of the
//! record at ebp and some variables of its own into the shape's variables (one copy only
//! when `dword_6D1502` is 0) and draws the shape (`hud::draw_disc`, `hud::draw_needle`);
//! the copies by `push` / `pop`, so no register changes before the call. The layouts are
//! the HUD's three (the record's +0xF4, `sub_4B1CD0`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, rva};
use crate::service::harness::{Region, at, capture, restore};

/// Where a copy reads.
#[derive(Clone, Copy)]
pub enum From {
    /// A field of the record at ebp.
    Field(u32),
    /// A variable (its RVA).
    Var(usize),
}

/// One set-up: its copies (from, to, only when `dword_6D1502` is 0) and, for the test,
/// the original's length, first bytes, call offset and callee.
pub struct Setup {
    pub copies: &'static [(From, usize, bool)],
    pub len: usize,
    pub entry: [u8; 9],
    pub call: usize,
    pub callee: &'static [(usize, u32)],
}

const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502

/// A call by place: 1 the shape.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version of a set-up (see the header): the registers the shape leaves.
pub unsafe fn set_up(entry: &Regs, s: &Setup, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let flat = game_read::<u32>(FLAT_RVA) == 0;
        for &(from, to, only_flat) in s.copies {
            if only_flat && !flat {
                continue;
            }
            let v = match from {
                From::Field(o) => ((entry.ebp + o) as *const u32).read_unaligned(),
                From::Var(r) => game_read(r),
            };
            game_write(to, v);
        }
    }
    let mut g = *entry;
    part(1, &mut g)?;
    Ok(g)
}

/// The test of a set-up: a copy whose call goes to a stand-in, against the Rust version,
/// on a record of the test's, `dword_6D1502` 0 and 1.
pub fn test(hook: &RawHook, address: usize, s: &Setup) -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match copy(address, s.len, &s.entry, &[s.call], s.callee) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = s.copies.iter().map(|&(_, to, _)| at(to)).collect();
    let mut vars = regions.clone();
    vars.push(at(FLAT_RVA));
    for &(from, _, _) in s.copies {
        if let From::Var(r) = from {
            vars.push(at(r));
        }
    }
    let saved = capture(&vars);
    let record: Vec<u32> = (0..0x40u32)
        .map(|n| n.wrapping_mul(0x0101_0101) ^ 0x5A)
        .collect();
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for flat in [0u32, 1] {
        let before = Regs {
            ebp: record.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(FLAT_RVA, flat);
            for (n, &(from, to, _)) in s.copies.iter().enumerate() {
                game_write(to, 0xAAAA_AAAAu32);
                if let From::Var(r) = from {
                    game_write(r, 0x100 + n as u32);
                }
            }
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            set_up(&before, s, &mut |p, r| {
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
                "flat {flat}: original {:x?} {:x?}; rust {:x?} {:x?}; same state {}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2 == mine.2
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
