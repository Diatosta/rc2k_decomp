//! The host's chat command "/kick <name>": the first other player of that name (case ignored)
//! is put out of the session (`dp_session::Session::kick`). A line that starts so is never sent
//! to the others; the host sees the answer as a line in its own chat. On a machine that is not
//! the host the command says so.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::add_chat_line::add_chat_line;
use crate::net::owner::Net;

const COMMAND: &[u8] = b"/kick ";

/// Whether `typed` is the kick command (case ignored, with a space after it).
pub(super) fn is_kick(typed: &[u8]) -> bool {
    typed.len() >= COMMAND.len() && typed[..COMMAND.len()].eq_ignore_ascii_case(COMMAND)
}

/// The name a kick command asks for: the rest of the line without its spaces at both ends.
fn kick_name(typed: &[u8]) -> &[u8] {
    typed[COMMAND.len()..].trim_ascii()
}

/// Runs the command in `typed` (the typed line up to its end): the player named is put out when
/// this machine hosts; a line tells the result.
pub(super) fn kick_command(
    net: &Net,
    strings: &crate::strings::owner::Strings,
    gfx: &'static Gfx,
    typed: &[u8],
) -> Result<(), Refused> {
    let name = kick_name(typed);
    let say = |n: u32| {
        crate::strings::append_string_by_id::own_text(
            strings,
            rc2k_core::strings::own::id::KICK_ANSWERS + n,
        )
    };
    let mut answer = if !net.vars.own.state().is_host() {
        say(0)
    } else {
        let own = net.race_slot.player();
        let mut found = None;
        for n in 0..net.race_slot.players() {
            let block = net.vars.player_block(n)?;
            if n != own && block.name.text().trim_ascii().eq_ignore_ascii_case(name) {
                found = Some(block.id.get());
                break;
            }
        }
        match found {
            None => say(1),
            Some(id) if crate::net::dp_objects::kick(net.link.dplay(), id) == 0 => say(2),
            Some(_) => say(3),
        }
    };
    // The chat takes a text up to its NUL.
    answer.push(0);
    let _received = net.lock_received();
    add_chat_line(net, gfx, net.race_slot.player(), &answer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_and_its_name() {
        assert!(is_kick(b"/kick Ann"));
        assert!(is_kick(b"/KICK  Ann "));
        assert!(!is_kick(b"/kick"));
        assert!(!is_kick(b"kick Ann"));
        assert!(!is_kick(b"/kicker Ann"));
        assert_eq!(kick_name(b"/kick  Ann Lee "), b"Ann Lee");
    }
}
