//! Two draw-list steps that put up `screens::draw_message_box` when the chosen item is not
//! open (stream A, part F1, 2026-10-18).

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// A note's box put up (converted: `rc2k_core::screens::locked`). The original's x for the
/// box is the caller's ecx, which only the measuring of the text gets, in a font in measure
/// mode (`dword_51E824`'s mode 4): it changes nothing, so 0. What the original leaves (eax)
/// is read by no list entry after it.
fn show(note: rc2k_core::screens::locked::Note) -> Result<(), Refused> {
    if note.eax.is_some() && note.show {
        unsafe {
            crate::screens::draw_message_box::draw_message_box(
                0,
                &mut crate::text::draw_text::GameCanvas,
            )?
        };
    }
    Ok(())
}

/// `screens::note_locked_item_3` (0x4AAAB8): unless cheat 0x11 is on, with item 3 chosen
/// (`dword_755525`) and `dword_75ACCC` not 1, the message box.
pub fn note_locked_item_3() -> Result<(), Refused> {
    show(rc2k_core::screens::locked::item_3(
        k(0x75870C),
        k(0x755525),
        k(0x75ACCC),
    ))
}

/// `screens::note_locked_stage` (0x48ADCE): unless cheat 0 is on, with an item chosen other
/// than 1 and 7 whose stage (`byte_758800[item - 2]`) is not open (1), the message box.
pub fn note_locked_stage() -> Result<(), Refused> {
    show(rc2k_core::screens::locked::stage(
        k(0x75870C),
        k(0x755525),
        |index| unsafe {
            ((game_addr(rva(0x758800)) as u32).wrapping_add(index) as *const u8).read()
        },
    ))
}
