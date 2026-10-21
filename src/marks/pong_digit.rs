//! `marks::pong_digit` (0x4B8418): one score digit: `marks::pong_cell` for its 4 x 5
//! pattern (`unk_768704`, 0x14 bytes a digit; ebx the column, ecx the row, eax walking
//! the pattern), then `dword_7687D4` on 0x40.
//!
//! In:  eax the digit
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `marks::pong_score` (six times). The cells through `part` (1), in the game
//! their Rust version. Refuses a digit past 9 (the patterns' end), before anything.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose cell calls go
//! to a stand-in, against the Rust version, every digit.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::pong;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::pong_digit", rva(0x4B8418), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place: 1 `pong_cell`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::pong::{pattern, cells}`
pub unsafe fn pong_digit(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use rc2k_core::marks::pong as core;
    let mut r = *entry;
    r.eax = game_addr(pong::PATTERNS_RVA) as u32 + core::pattern(entry.eax)?;
    for (col, row) in core::cells() {
        r.ebx = col;
        r.ecx = row;
        part(1, &mut r)?;
    }
    unsafe {
        let at: u32 = game_read(pong::DIGIT_AT[0]);
        game_write(pong::DIGIT_AT[0], at.wrapping_add(core::DIGIT_STEP));
    }
    Ok(*entry)
}

/// In the game: `pong_cell`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::pong_cell as pc;
    *r = unsafe { pc::pong_cell(r, &mut pc::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { pong_digit(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = (0..20).map(|k| 0x14 + 0xF * k).collect();
    // pusha; imul eax, 14h; lea eax, unk_768704[eax]
    let entry: &[u8] = &[0x60, 0x6B, 0xC0, 0x14, 0x8D, 0x80, 0x04, 0x87, 0x76, 0x00];
    let copy = match copy(0x4B8418, 0x13F, entry, &calls, &[(0x4B8557, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = [at(pong::DIGIT_AT[0])];
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for digit in 0..10u32 {
        let before = Regs {
            eax: digit,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(pong::DIGIT_AT[0], 0x100 + digit);
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&vars));
        prepare();
        let mine = unsafe {
            pong_digit(&before, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (mine.as_ref().ok().map(regs), take_called(), capture(&vars));
        t.check(theirs == mine, || {
            format!(
                "digit {digit}: original {:x?} {} calls; rust {:x?} {} calls; same calls {}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.1 == mine.1
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
