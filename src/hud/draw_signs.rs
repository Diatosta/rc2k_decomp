//! `hud::draw_signs` (0x4B4F2B): up to four signs of the HUD's record (+0xA4 .. +0xB0, 0
//! none): sign v as rectangle 0x4C + (v - 1) & 0xF of `dword_75B940`, centred in its 0x40
//! wide place by its width (`dword_75BF80`[(v - 1) & 7]), the places from +0xB4 on 0x40
//! apart, at 0x36 (0x16 with `dword_6D1502` 0).
//!
//! In:  ebp the HUD's record
//! Out: ebx the last place, ecx the last sign's row (0 when the last is none), eax / esi
//!      the last drawn sign's rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F27). The sprites through `part` (1), in the game
//! `add_sprite`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, signs by
//! lot.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::draw_marker::RECTS_RVA;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_signs", rva(0x4B4F2B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WIDTHS_RVA: usize = rva(0x75BF80); // dword_75BF80: 8
pub const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502

/// A call by place: 1 `add_sprite`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The row: `y`, or 0x20 higher with `dword_6D1502` 0.
pub fn row(y: u32) -> u32 {
    row_by(y, 0x20)
}

/// The row: `y`, or `lift` higher with `dword_6D1502` 0.
pub fn row_by(y: u32, lift: u32) -> u32 {
    rc2k_core::hud::signs::row(y, lift, unsafe { game_read(FLAT_RVA) })
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::signs` (through `signs_with`)
pub unsafe fn draw_signs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { signs_with(entry, 0xA4, 0xB4, 0, 0x36, 0x20, part) }
}

/// Four signs from the record's `first` .. on, the places from its `start` field plus
/// `shift`, at row `y` (shared with `hud::draw_signs_1a` / `_1b`).
// Converted: `rc2k_core::hud::signs::{sign, place, row}`
pub unsafe fn signs_with(
    entry: &Regs,
    first: u32,
    start: u32,
    shift: u32,
    y: u32,
    lift: u32,
    part: Part,
) -> Result<Regs, Refused> {
    use rc2k_core::hud::signs;
    let mut g = *entry;
    unsafe {
        let get = |o: u32| ((g.ebp + o) as *const u32).read_unaligned();
        let places = get(start).wrapping_add(shift);
        for n in 0..signs::SIGNS as u32 {
            g.ebx = signs::place(places, n);
            let v = get(first + 4 * n);
            g.ecx = v;
            let Some(sign) = signs::sign(places, n, v, |k| game_read(WIDTHS_RVA + 4 * k as usize))
            else {
                continue;
            };
            g.eax = sign.rectangle;
            let mut r = Regs {
                ebx: sign.x,
                ecx: row_by(y, lift),
                esi: game_addr(RECTS_RVA) as u32 + g.eax,
                ..g
            };
            part(1, &mut r)?;
            g.ecx = r.ecx;
            g.esi = r.esi;
        }
    }
    Ok(g)
}

/// The test of a signs function: a copy whose calls go to a stand-in, against `rust`.
pub fn test_signs(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    rust: &dyn Fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match copy(address, len, entry, calls, &[(0x4B70EE, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = vec![at(FLAT_RVA)];
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..4u32 {
        let mut record = [0u32; 0x40];
        for n in 0..9u32 {
            record[(0xA4 / 4 + n) as usize] = if (case + n) % 3 == 0 {
                0
            } else {
                n * 5 + case + 1
            };
        }
        record[0xB4 / 4] = 0x100 + case;
        record[0xC8 / 4] = 0x180 + case;
        let before = Regs {
            ebp: record.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(FLAT_RVA, case & 1);
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called());
        prepare();
        let mine = rust(&before, &mut |p, r| {
            note(p, r);
            Ok(())
        });
        let mine = (mine.as_ref().ok().map(regs), take_called());
        t.check(theirs == mine, || {
            format!("case {case}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&vars, &saved);
    t.finish()
}

/// In the game: `add_sprite`'s Rust version.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::hud::draw_marker::game_part(place, r)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_signs(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    // mov ebx, [ebp+0B4h]
    let entry: &[u8] = &[0x8B, 0x9D, 0xB4, 0x00, 0x00, 0x00];
    test_signs(
        &HOOK,
        0x4B4F2B,
        0x154,
        entry,
        &[0x51, 0xA5, 0xF9, 0x14D],
        &|r, p| unsafe { draw_signs(r, p) },
    )
}
