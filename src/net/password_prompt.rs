//! The question a joiner is asked when the session chosen is locked: its password typed on the
//! same entry the settings' texts use (the keyboard, or the on-screen keyboard from a pad), shown
//! as stars. Enter answers, Escape goes back.

use crate::errors::Refused;
use crate::net::begin_edit::{EditedText, begin_edit};
use crate::net::lobby::{
    FONT, PALETTE, copy_picture, draw_string, list_key, lock_surface, push_id, run_frame,
    store_text,
};
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::{editing, screens};

/// The item number the entry is kept under (any but "not editing").
const ENTRY: u32 = 0;

/// Where the question and the typed stars stand: high on the screen, so the on-screen keyboard
/// (below the field) leaves both in view.
const QUESTION_Y: u32 = 0x80;
const FIELD_Y: u32 = 0xB0;

/// The question's frame: the background, the question and the stars typed so far with the caret.
fn frame(o: crate::owners::Owners<'static>, question: u32) -> Result<(), Refused> {
    if !lock_surface(o.gfx, o.system) {
        return Ok(());
    }
    copy_picture(o.gfx, o.system)?;
    draw_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        question,
        crate::net::message_frame::FONT_B,
        0x190,
        QUESTION_Y,
        o.gfx,
    )?;
    crate::input::typed_text::note_field_y(FIELD_Y as i32);
    crate::input::typed_text::note_secret();
    let typed = o.net.settings().most_players_text.read();
    let length = typed.iter().position(|&b| b == 0).unwrap_or(typed.len());
    let mut shown = vec![b'*'; length];
    push_id(o, &mut shown, screens::CARET)?;
    let shown = store_text(o.strings, shown)?;
    crate::text::draw_text::draw_text_at(
        FONT,
        &shown,
        0x17C,
        FIELD_Y as i32,
        &mut GameCanvas::new(o.gfx),
    )?;
    crate::platform::unlock_surface(o.gfx);
    crate::platform::show_frame_now(o);
    Ok(())
}

/// Asks for the password (`wrong`: the last answer was refused). The answer, or `None` when the
/// player goes back (Escape, or the game closing).
pub fn ask(o: crate::owners::Owners<'static>, wrong: bool) -> Result<Option<Vec<u8>>, Refused> {
    use rc2k_core::strings::own::id::{PASSWORD_PROMPT, PASSWORD_WRONG};
    let (net, system) = (o.net, o.system);
    let list_state = &o.menus.list;
    let question = if wrong {
        PASSWORD_WRONG
    } else {
        PASSWORD_PROMPT
    };
    net.settings()
        .most_players_text
        .write_at(0, &[0], "a text line's bytes")?;
    begin_edit(
        net,
        &net.settings().edit,
        ENTRY,
        EditedText::MostPlayers,
        editing::WIDTH,
        PALETTE,
    )?;
    let now = system.session.key_clock.ticks.get();
    list_state.move_clock.set(now);
    list_state.key_clock.set(now);
    let answer = loop {
        run_frame(o, || frame(o, question))?;
        list_state
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(list_state, o.input)?;
        let went_back = editing::EditKey::of(key) == editing::EditKey::Escape
            || system.session.flags.quitting.is_set();
        let ended = crate::net::edit_key::edit_key(net, &net.settings().edit, key, o.gfx)?;
        if went_back {
            break None;
        }
        if ended {
            let typed = net.settings().most_players_text.read();
            let end = typed.iter().position(|&b| b == 0).unwrap_or(typed.len());
            break Some(typed[..end].to_vec());
        }
    };
    net.settings().edit.edited.set(editing::NOT_EDITING);
    net.settings()
        .most_players_text
        .write_at(0, &[0; 64], "a text line's bytes")?;
    Ok(answer)
}
