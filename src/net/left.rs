//! A player who leaves a network race. The receiving thread notes who left (a player destroyed in
//! the session, or every other player when the session is lost); the car's own step then takes
//! the car out of the race, unless it had finished: the car is made no driver's (as the cars
//! past the race's count are), so it is not stepped, not drawn, has no collisions and no sound.
//! The others are told on the race's frame ("<name> left the race") for a few seconds.
//!
//! The flags are cleared when a network race begins and when it ends, so no race that follows
//! (a single machine's included) ever sees one.

use crate::net::owner::Net;
use crate::race::owner::Race;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The players a session has room for.
const PLAYERS: usize = rc2k_core::net::players::MOST as usize;

/// How long a message stays on the frame.
const SHOWN: std::time::Duration = std::time::Duration::from_secs(6);
/// The most messages shown at once (the oldest go first).
const MOST_SHOWN: usize = 3;

/// A network race is being played: a player destroyed in the lobby leaves no mark.
static RACING: AtomicBool = AtomicBool::new(false);

/// Players (by car number) who left the race and whose cars are not yet taken out.
static LEFT: [AtomicBool; PLAYERS] = [const { AtomicBool::new(false) }; PLAYERS];

/// The host's leaving ended the session during the race: the race ends on this machine too.
static ENDED: AtomicBool = AtomicBool::new(false);

/// A message on the frame.
struct Notice {
    /// The own text's id and the name that goes in it.
    id: u32,
    name: Vec<u8>,
    until: std::time::Instant,
}

static NOTICES: Mutex<Vec<Notice>> = Mutex::new(Vec::new());

fn notices() -> std::sync::MutexGuard<'static, Vec<Notice>> {
    NOTICES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A network race begins: nobody has left it.
pub fn begin_race() {
    forget();
    RACING.store(true, Ordering::SeqCst);
}

/// The network race is over.
pub fn end_race() {
    RACING.store(false, Ordering::SeqCst);
    forget();
}

fn forget() {
    for flag in &LEFT {
        flag.store(false, Ordering::SeqCst);
    }
    ENDED.store(false, Ordering::SeqCst);
    notices().clear();
}

/// The session's player with DirectPlay id `id` was destroyed (the receiving thread): in a race,
/// that player has left it.
pub fn player_destroyed(net: &Net, id: u32) {
    if !RACING.load(Ordering::SeqCst) || !net.race_slot.is_on() {
        return;
    }
    for n in 0..net.race_slot.players().min(PLAYERS as u32) {
        let known = net.vars.player_block(n).is_ok_and(|b| b.id.get() == id);
        if known && n != net.race_slot.player() {
            LEFT[n as usize].store(true, Ordering::SeqCst);
        }
    }
}

/// The session was lost during a race (the receiving thread): the host has gone and with it the
/// session. The race ends here as a player's own leaving ends it, and the player is told once the
/// menus are back.
pub fn session_lost(net: &Net) {
    if !RACING.load(Ordering::SeqCst) || !net.race_slot.is_on() {
        return;
    }
    ENDED.store(true, Ordering::SeqCst);
    for n in 0..net.race_slot.players().min(PLAYERS as u32) {
        let Ok(block) = net.vars.player_block(n) else {
            continue;
        };
        if block.record.state().is_host() {
            let name = rc2k_core::files::ansi::ansi_text(
                block
                    .name
                    .text()
                    .split(|&b| b == 0)
                    .next()
                    .unwrap_or_default(),
            );
            crate::error_dialog::notice_own_later(
                "net:host_left",
                rc2k_core::strings::own::id::HOST_LEFT,
                &name,
            );
        }
    }
}

/// Whether the host's leaving has ended the session under the race being played.
pub fn session_ended() -> bool {
    ENDED.load(Ordering::SeqCst)
}

/// The step of the car being stepped, a remote player's: when its player has left, and the car
/// had not finished, it is taken out of the race and the others are told.
pub fn apply(race: &Race, net: &Net) {
    let number = race.shared.work.get().current_car.get() as usize;
    if number >= PLAYERS || net.race_slot.player() as usize == number {
        return;
    }
    if !LEFT[number].swap(false, Ordering::SeqCst) {
        return;
    }
    let car = race.shared.car();
    // A player who finished keeps the car and the time.
    if car.finish_flag.get() != 0 {
        return;
    }
    car.driver_kind.set(0u32);
    let block = net.vars.player_block(number as u32);
    let name = block.map(|b| b.name.text().to_vec()).unwrap_or_default();
    let id = rc2k_core::strings::own::id::LEFT_RACE;
    let mut shown = notices();
    shown.push(Notice {
        id,
        name,
        until: std::time::Instant::now() + SHOWN,
    });
    if shown.len() > MOST_SHOWN {
        shown.remove(0);
    }
}

/// The text of a message: the own text with its "{}" replaced by the name.
fn text_of(template: &[u8], name: &[u8]) -> Vec<u8> {
    let name = name.split(|&b| b == 0).next().unwrap_or_default();
    let mut text = Vec::new();
    let mut rest = template;
    while let Some(at) = rest.windows(2).position(|w| w == b"{}") {
        text.extend_from_slice(&rest[..at]);
        text.extend_from_slice(name);
        rest = &rest[at + 2..];
    }
    text.extend_from_slice(rest);
    text.push(0);
    text
}

/// The messages drawn on the race's frame (the surface locked): one line each, in the middle
/// of the screen's width near the top.
pub fn draw(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    let now = std::time::Instant::now();
    let lines: Vec<Vec<u8>> = {
        let mut shown = notices();
        shown.retain(|n| n.until > now);
        shown
            .iter()
            .map(|n| {
                text_of(
                    &crate::strings::append_string_by_id::own_text(o.strings, n.id),
                    &n.name,
                )
            })
            .collect()
    };
    if lines.is_empty() {
        return Ok(());
    }
    let font = crate::gfx::typefaces::fonts::LIST;
    let screen = o.system.session.screen.get();
    let (width, height) = (screen.width.get() as i32, screen.height.get() as i32);
    let line_height = font.object(&o.gfx.fonts.typefaces).line_height.get() as i32;
    let mut canvas = crate::text::draw_text::GameCanvas::new(o.gfx);
    let mut y = height / 10;
    for line in lines {
        let text_width =
            crate::gfx::button_hints::width_of(font, &line[..line.len() - 1], &mut canvas);
        crate::text::draw_text::draw_text_at(
            font,
            &line,
            (width - text_width) / 2,
            y,
            &mut canvas,
        )?;
        y += line_height;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_has_the_name_where_the_braces_are() {
        assert_eq!(
            text_of(b"{} left the race", b"Ana\0junk"),
            b"Ana left the race\0"
        );
        assert_eq!(text_of(b"no braces", b"Ana"), b"no braces\0");
    }

    #[test]
    fn flags_are_cleared_when_a_race_begins_or_ends() {
        let _lock = crate::test_support::test_lock();
        LEFT[3].store(true, Ordering::SeqCst);
        begin_race();
        assert!(!LEFT[3].load(Ordering::SeqCst));
        LEFT[3].store(true, Ordering::SeqCst);
        end_race();
        assert!(!LEFT[3].load(Ordering::SeqCst));
        assert!(!RACING.load(Ordering::SeqCst));
    }
}
