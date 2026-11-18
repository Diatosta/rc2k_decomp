//! The name of the highlighted button, shown under the viewer's render button for a second or so
//! when the highlight moves (and when the viewer opens on its play button), then faded out over
//! a quarter of a second. It is drawn on the 2D layer over the finished frame in the race's notice
//! style (`gfx::notice_text`: the warnings' font and outline, no box), only while the viewer is
//! stopped and not while a video is rendered. The choice of the text and the timing are `rc2k_core::race::viewer_tip`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::race::render_row;
use rc2k_core::race::viewer_tip::{
    BUTTONS, HUD_BUTTONS, Highlight, SPEED_BUTTON, button_text, hud_text, speed_text, strength,
};
use rc2k_core::strings::own::id;
use std::sync::Mutex;
use std::time::Instant;

/// What had the highlight when the name was last started, and when.
struct Shown {
    highlight: Highlight,
    since: Instant,
}

static SHOWN: Mutex<Option<Shown>> = Mutex::new(None);

/// The viewer opens: the first highlight shown names itself.
pub fn reset() {
    *SHOWN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// Where the highlight is now.
fn highlight(race: &Race, choice: u32) -> Highlight {
    let editing = race.viewer_buffers().camera_edit.get();
    if editing != 0 {
        Highlight::Editing(editing)
    } else if render_row::selected() {
        Highlight::Render
    } else {
        Highlight::Button(choice.min(BUTTONS - 1))
    }
}

/// The text that names `highlight`, in the language in use ("{}" filled in).
fn text_of(o: Owners<'static>, highlight: Highlight) -> String {
    let race = o.race;
    let block = race.replay.replay_view().replay_block.get() as usize;
    let camera_byte =
        |byte: usize| crate::race::viewer::camera_byte_at(race, byte.wrapping_add(8 * block)).ok();
    let (text, with) = match highlight {
        Highlight::Render => (id::VIEWER_TIP_RENDER, None),
        Highlight::Editing(camera) => (id::VIEWER_TIP_EDITING, Some(camera)),
        Highlight::Button(button) => {
            let hud = HUD_BUTTONS.iter().position(|&b| b == button);
            match (
                hud,
                camera_byte(if button == SPEED_BUTTON {
                    4
                } else {
                    2 + hud.unwrap_or(0)
                }),
            ) {
                (Some(car), Some(byte)) => (hud_text(byte != 0), Some(car as u32 + 1)),
                (None, Some(byte)) if button == SPEED_BUTTON => (speed_text(byte), None),
                _ => (button_text(button).unwrap_or(id::BLANK), None),
            }
        }
    };
    let template = rc2k_core::files::ansi::ansi_text(
        &crate::strings::append_string_by_id::own_text(o.strings, text),
    );
    template.replace("{}", &with.map(|n| n.to_string()).unwrap_or_default())
}

/// The box and the name drawn over the frame, if the name is still to show.
pub fn draw(o: Owners<'static>) -> Result<(), Refused> {
    let highlight = highlight(o.race, o.menus.list.choice.get());
    let elapsed = {
        let mut shown = SHOWN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        match shown.as_mut() {
            Some(s) if s.highlight == highlight => {}
            _ => {
                *shown = Some(Shown {
                    highlight,
                    since: now,
                });
            }
        }
        shown
            .as_ref()
            .map_or(0, |s| s.since.elapsed().as_millis() as u32)
    };
    let Some(strength) = strength(elapsed) else {
        return Ok(());
    };
    let text = text_of(o, highlight);
    draw_label(o, &rc2k_core::files::ansi::ansi_bytes(&text), strength);
    Ok(())
}

/// Where the label's letters start, in the panel's 800 x 600 units: just under the render button
/// (the button ends at layout y 227), so the name hides nothing of the panel.
const LABEL_TOP: i32 = 235;

fn draw_label(o: Owners<'static>, text: &[u8], strength: u8) {
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return;
    }
    crate::gfx::notice_text::draw(o, text, LABEL_TOP, strength);
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
}
