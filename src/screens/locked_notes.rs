//! Two draw-list steps that put up `screens::draw_message_box` when the chosen item is not
//! open.

use crate::errors::Refused;

/// The index of item 0 (`item - 2`, wrapping): two bytes before the levels won.
const NOT_IN_TABLE: u32 = u32::MAX - 1;

/// A note's box put up (converted: `rc2k_core::screens::locked`). The box's x only reaches the
/// measuring of the text, in a font in measure mode: it changes nothing, so 0.
fn show(
    o: crate::owners::Owners<'static>,
    note: rc2k_core::screens::locked::Note,
) -> Result<(), Refused> {
    if note.show {
        crate::screens::draw_message_box::draw_message_box(
            o,
            0,
            &mut crate::text::draw_text::GameCanvas::new(o.gfx),
            o.gfx,
        )?;
    }
    Ok(())
}

/// Unless cheat 0x11 is on, with item 3 chosen (`LoopFlags.item`) and `RaceState.settings.a8_game`
/// not 1, the message box.
pub fn note_locked_item_3(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    show(
        o,
        rc2k_core::screens::locked::item_3(
            o.menus.play.cheats.cheats.get(),
            o.menus.screens.loop_flags.item.get(),
            o.race.shared.state.settings.a8_game.get(),
        ),
    )
}

/// Unless cheat 0 is on, with an item chosen other than 1 and 7 whose stage
/// (`Arcade.levels_won[item - 2]`) is not open (1), the message box.
pub fn note_locked_stage(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    let fault = crate::errors::Fault::default();
    let note = rc2k_core::screens::locked::stage(
        menus.play.cheats.cheats.get(),
        menus.screens.loop_flags.item.get(),
        |index| {
            // The screen has eight items: item n is the stage `levels_won[n - 2]`, items 1 and
            // 7 are not asked about. Item 0 (index -2) lies before the table, in bytes nothing
            // writes: not open.
            match index {
                0..8 => menus.play.arcade.opened(index as usize),
                NOT_IN_TABLE => 0,
                _ => fault.ok(Err(crate::errors::BadInput(
                    "a stage's lock past the table",
                ))),
            }
        },
    );
    fault.check()?;
    show(o, note)
}
