//! Two draw-list steps that put up `screens::draw_message_box` when the chosen item is not
//! open ().

use crate::errors::Refused;
use crate::menu::arcade_store::Arcade;
use crate::menu::owner::Menus;

/// A note's box put up (converted: `rc2k_core::screens::locked`). The original's x for the
/// box only reaches the measuring of the text, in a font in measure mode: it changes
/// nothing, so 0.
fn show(note: rc2k_core::screens::locked::Note) -> Result<(), Refused> {
    let gfx = crate::gfx::owner::gfx();
    if note.show {
        crate::screens::draw_message_box::draw_message_box(
            0,
            &mut crate::text::draw_text::GameCanvas,
            gfx,
        )?;
    }
    Ok(())
}

/// `screens::note_locked_item_3` (0x4AAAB8): unless cheat 0x11 is on, with item 3 chosen
/// (`LoopFlags.item`) and `RaceState.settings.a8_game` not 1, the message box.
pub fn note_locked_item_3(menus: &Menus, race: &crate::race::owner::Race) -> Result<(), Refused> {
    show(rc2k_core::screens::locked::item_3(
        menus.play.cheats.cheats.get(),
        menus.screens.loop_flags.item.get(),
        race.shared.state.settings.a8_game.get(),
    ))
}

/// `screens::note_locked_stage` (0x48ADCE): unless cheat 0 is on, with an item chosen other
/// than 1 and 7 whose stage (`Arcade.levels_won[item - 2]`) is not open (1), the message box.
pub fn note_locked_stage(menus: &Menus) -> Result<(), Refused> {
    let fault = crate::menu::event_flow::Fault::default();
    let note = rc2k_core::screens::locked::stage(
        menus.play.cheats.cheats.get(),
        menus.screens.loop_flags.item.get(),
        |index| {
            fault.read_at::<u8>(
                // Bounded to the arcade's store, not the six bytes: item 0 reads before them.
                crate::pointers::GamePtr::to(&menus.play.arcade)
                    .offset(core::mem::offset_of!(Arcade, levels_won) as u32)
                    .offset(index),
                "a stage\'s lock",
            )
        },
    );
    fault.check()?;
    show(note)
}
