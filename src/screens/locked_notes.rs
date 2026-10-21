//! Two draw-list steps that put up `screens::draw_message_box` when the chosen item is not
//! open (stream A, part F1, 2026-10-18).

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, game_read, rva};
use crate::screens::harness::Left;

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// What a note leaves, with the box put up through `part` (converted:
/// `rc2k_core::screens::locked`).
fn leave(
    note: rc2k_core::screens::locked::Note,
    r: &mut Regs,
    part: crate::race::frame::Part,
) -> Result<Left, Refused> {
    let Some(eax) = note.eax else {
        return Ok(Left::nothing());
    };
    r.eax = eax;
    if note.show {
        part(1, r)?;
    }
    Ok(Left::nothing().with(EAX, r.eax))
}

/// `screens::note_locked_item_3` (0x4AAAB8): unless cheat 0x11 is on, with item 3 chosen
/// (`dword_755525`) and `dword_75ACCC` not 1, the message box (part 1). Gives eax as the
/// original leaves it (the item, when it looks at it).
pub unsafe fn note_locked_item_3(
    r: &mut Regs,
    part: crate::race::frame::Part,
) -> Result<Left, Refused> {
    let note = rc2k_core::screens::locked::item_3(k(0x75870C), k(0x755525), k(0x75ACCC));
    leave(note, r, part)
}

/// `screens::note_locked_stage` (0x48ADCE): unless cheat 0 is on, with an item chosen other
/// than 1 and 7 whose stage (`byte_758800[item - 2]`) is not open (1), the message box
/// (part 1). Gives eax as the original leaves it.
pub unsafe fn note_locked_stage(
    r: &mut Regs,
    part: crate::race::frame::Part,
) -> Result<Left, Refused> {
    let note = rc2k_core::screens::locked::stage(k(0x75870C), k(0x755525), |index| unsafe {
        ((game_addr(rva(0x758800)) as u32).wrapping_add(index) as *const u8).read()
    });
    leave(note, r, part)
}

/// Their part in the game: the message box (x the caller's ecx).
pub fn box_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::screens::draw_message_box::draw_message_box(
            r.ecx as i32,
            &mut crate::text::draw_text::GameCanvas,
        )
    }
}
