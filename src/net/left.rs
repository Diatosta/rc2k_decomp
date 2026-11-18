//! A player who leaves a network race. The receiving thread notes who left (a player destroyed in
//! the session, or every other player when the session is lost); the car's own step then takes
//! the car out of the race, unless it had finished: the car is made no driver's (as the cars
//! past the race's count are), so it is not stepped, not drawn, has no collisions and no sound.
//! The others are told on the race's frame ("<name> left the race"): in the race's notice font
//! like the game's own messages, blinking for a few seconds, one message after another.
//!
//! The flags are cleared when a network race begins and when it ends, so no race that follows
//! (a single machine's included) ever sees one.

use crate::net::owner::Net;
use crate::race::owner::Race;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The players a session has room for.
const PLAYERS: usize = rc2k_core::net::players::MOST as usize;

/// How long a message stays on the frame (messages follow one another).
const SHOWN: std::time::Duration = std::time::Duration::from_secs(5);
/// How long the message is lit and how long dark while it blinks.
const BLINK: std::time::Duration = std::time::Duration::from_millis(400);
/// The most messages waiting (the oldest go first).
const MOST_SHOWN: usize = 4;

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
    /// When it began to be shown (it waits for the ones before it).
    since: Option<std::time::Instant>,
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

/// The session was lost during a race or in the lobby (the receiving thread): the host has gone and with it the
/// session. The race ends here as a player's own leaving ends it, and the player is told once the
/// menus are back.
pub fn session_lost(net: &Net) {
    if !net.race_slot.is_on() {
        return;
    }
    // In the lobby the session's end is the lobby's: it closes at its next frame, and the
    // player is told the same way.
    if RACING.load(Ordering::SeqCst) {
        ENDED.store(true, Ordering::SeqCst);
    }
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
            crate::error_dialog::notice_own_each_time(
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
        since: None,
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

/// Whether a message begun at `since` is lit at `now`: it blinks, lit first.
fn lit(since: std::time::Instant, now: std::time::Instant) -> bool {
    (now.duration_since(since).as_millis() / BLINK.as_millis()).is_multiple_of(2)
}

/// The message to show now: the first of the queue, begun now when it was waiting, and the ones
/// whose time is up taken off. Its text, and whether it is lit.
fn current(now: std::time::Instant) -> Option<(Vec<u8>, u32, bool)> {
    let mut shown = notices();
    while let Some(first) = shown.first_mut() {
        let since = *first.since.get_or_insert(now);
        if now.duration_since(since) >= SHOWN {
            shown.remove(0);
            continue;
        }
        return Some((first.name.clone(), first.id, lit(since, now)));
    }
    None
}

/// The message drawn on the race's frame (the surface locked) as the game's own race messages
/// look: in the race's notice font, centred under the top, on a plate, enlarged with the screen
/// as the quit question is, blinking for a few seconds. Several leavers follow one another.
pub fn draw(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    let Some((name, id, lit)) = current(std::time::Instant::now()) else {
        return Ok(());
    };
    if !lit {
        return Ok(());
    }
    let mut line = text_of(
        &crate::strings::append_string_by_id::own_text(o.strings, id),
        &name,
    );
    // The race's notice font has capitals, as the game's messages are written.
    line.make_ascii_uppercase();
    let mut font = o.gfx.fonts.font_block.game_fonts[2].get();
    font.mode = rc2k_core::strings::text::MODE_AT_X;
    let mut canvas = crate::text::draw_text::GameCanvas::new(o.gfx);
    let mut measure = font;
    measure.mode = rc2k_core::strings::text::MODE_MEASURE;
    let text_width =
        crate::text::measure_text::measure_text(&mut measure, &line[..], 0, &mut canvas)
            .map_or(0, |m| m.width as i32);
    let pad = 6;
    let (plate_w, plate_h) = (text_width + 2 * pad, font.line_height + 2 * pad);
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
    let grow = (h as f32 / 600.0).max(1.0).min(w as f32 / plate_w as f32);
    let scaled = |v: i32| (v as f32 * grow).round() as i32;
    let (final_w, final_h) = (scaled(plate_w), scaled(plate_h));
    let (left, top) = ((w - final_w) / 2, (h * 22 / 100 - scaled(pad)).max(0));
    crate::input::osk_draw::fill(
        o,
        [left, top, plate_w, plate_h],
        crate::drive::quit_question::PLATE,
    );
    crate::text::draw_text::draw_text(&mut font, &line[..], left + pad, top + pad, &mut canvas)?;
    crate::gfx::scaled_region::enlarge(
        o.gfx,
        [left, top, plate_w, plate_h],
        (left, top),
        (final_w, final_h),
    );
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
