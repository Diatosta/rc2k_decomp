//! `menu::list_step` (0x414A55): the highlight of a list menu (the replay viewer's,
//! `sub_4144CF`'s) moved by a key: 0x66 and 0x6B back and on, 0x68 and 0x69 the other way.
//! With a table of neighbours (ebx, 4 bytes an item: 0x66's, 0x6B's, 0x68's, 0x69's) the
//! item's entry, unless negative; without one (ebx 0) 0x66 and 0x6B go round the list
//! (`[edi+1]` items) and 0x68 and 0x69 do nothing. The item into `dword_51D407`.
//!
//! In:  al the key, ebx the table or 0, edi the list
//! Out: moved: ZF set, carry clear, ecx the item. Not moved: ZF clear; ecx kept for
//!      another key (carry: al below 0x69), else `dword_51D407` with cl the table's entry
//!      (carry clear), or for 0x68 / 0x69 without a table `dword_51D407` (carry set)
//!
//! Callers: `race::replay_viewer` (two), `sub_4144CF` (one), reading ZF. A loop target at
//! +4 (the patch would cut `jz`): private copy, manual patch. Shadow mode compares.
//! Self-test: the original against the Rust version, every key with and without a table,
//! items first, inside and last, table entries negative or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::viewer::{CHOICE, get, put};
use rc2k_core::menu::list_menu::{MOVES, Step, step};

/// What `list_step` gives: the highlight (ecx, with its low byte the new choice) where a
/// move key set it, and the carry and zero flags its callers test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stepped {
    pub choice: Option<u32>,
    pub carry: bool,
    pub zero: bool,
}

impl Stepped {
    /// The registers as the original leaves them, for the register-threaded callers by
    /// address.
    pub fn leave(&self, r: &mut Regs) {
        if let Some(choice) = self.choice {
            r.ecx = choice;
        }
        r.set_carry(self.carry);
        r.set_zero(self.zero);
    }
}

impl Stepped {
    /// The same in a flow's registers (`menu::event_flow::Flow`).
    pub fn carry_on(&self, f: &mut crate::menu::event_flow::Flow) {
        if let Some(choice) = self.choice {
            f.c.measure_x = choice;
        }
        f.c.carry = self.carry;
        f.zero = self.zero;
    }
}

/// `key` (al) moving the highlight of the list menu `list` (edi) by the neighbours table
/// `table` (ebx, 0 for none).
pub unsafe fn list_step(key: u8, table: u32, list: u32) -> Stepped {
    // Converted: `rc2k_core::menu::list_menu::step`.
    let count = unsafe { ((list + 1) as *const u8).read() };
    if !MOVES.contains(&key) {
        // Not a move: the choice is not looked at (`step` answers `Other` or `NoTable`).
        let (carry, zero) = match step(key, 0, count, table != 0, |_| 0) {
            Step::NoTable => (true, false),
            Step::Other { below } => (below, false),
            _ => unreachable!("a move from a key that is not one"),
        };
        return Stepped {
            choice: None,
            carry,
            zero,
        };
    }
    let ecx = get(CHOICE);
    let entry_of = |k: usize| unsafe {
        (table
            .wrapping_add(ecx.wrapping_mul(4))
            .wrapping_add(k as u32) as *const u8)
            .read()
    };
    let with_cl = |cl: u8| ecx & !0xFF | cl as u32;
    let (choice, carry, zero) = match step(key, ecx as u8, count, table != 0, entry_of) {
        Step::Moved(cl) => {
            put(CHOICE, with_cl(cl));
            (with_cl(cl), false, true)
        }
        Step::Blocked(cl) => (with_cl(cl), false, false),
        Step::NoTable => (ecx, true, false),
        Step::Other { below } => (ecx, below, false),
    };
    Stepped {
        choice: Some(choice),
        carry,
        zero,
    }
}
