//! The name the replay viewer shows for a second or so when the highlight moves to another
//! button: which button has the highlight, the own text that names what pressing it does, and how
//! long the name stays and how it fades. The game's side draws it.

use crate::strings::own::id;

/// What has the highlight in the stopped viewer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Highlight {
    /// A button of the two rows, by its number (0 ..= 25).
    Button(u32),
    /// The "render to video" button under the rows.
    Render,
    /// A camera is being edited (1 or 2): the highlight stays on its button, but the keys do
    /// something else.
    Editing(u32),
}

/// The buttons of the two rows.
pub const BUTTONS: u32 = 26;
/// The lower row's button that cycles the replay speed, and the two that show or hide a car's HUD.
pub const SPEED_BUTTON: u32 = 0x10;
pub const HUD_BUTTONS: [u32; 2] = [0xE, 0x18];

/// How long the name stays at full strength, and how long it then takes to fade (milliseconds).
pub(crate) const STAY_MS: u32 = 1000;
pub(crate) const FADE_MS: u32 = 250;

/// The strength of the name `elapsed_ms` after the highlight moved, 0 ..= 255; none once it has
/// faded.
pub fn strength(elapsed_ms: u32) -> Option<u8> {
    if elapsed_ms < STAY_MS {
        Some(255)
    } else if elapsed_ms < STAY_MS + FADE_MS {
        let left = STAY_MS + FADE_MS - elapsed_ms;
        Some((left * 255 / FADE_MS) as u8)
    } else {
        None
    }
}

/// A canvas pixel the 2D layer has not written (the GPU's canvas under the frame).
const UNWRITTEN: u32 = crate::gfx::atlas::UNWRITTEN_PIXEL;

/// The colour `own` (0x00RRGGBB) at `strength` over the canvas pixel `under`: blended into it
/// where it holds the frame's own pixel (a software frame), or `own` with the alpha in its top
/// byte where it is unwritten (the GPU's overlay reads that byte as the alpha).
pub fn over(own: u32, under: u32, strength: u8) -> u32 {
    if strength == 255 {
        return own;
    }
    if under == UNWRITTEN {
        return (own & 0x00FF_FFFF) | (u32::from(strength) << 24);
    }
    let a = u32::from(strength);
    let channel = |shift: u32| {
        let (o, u) = ((own >> shift) & 0xFF, (under >> shift) & 0xFF);
        (o * a + u * (255 - a) + 127) / 255
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

/// The replay speeds a block's speed byte stands for: normal, half and a quarter.
pub(crate) const SPEEDS: u32 = 3;

/// The text of a button's name: its number picks one of the `BUTTONS` texts that follow
/// `id::VIEWER_TIPS`. A number past the buttons is none.
pub fn button_text(button: u32) -> Option<u32> {
    (button < BUTTONS).then(|| id::VIEWER_TIPS + button)
}

/// The text of the speed the speed button goes to when pressed with the byte `current`.
pub fn speed_text(current: u8) -> u32 {
    let next = crate::race::viewer::next_byte(4, current, false, 0);
    id::VIEWER_TIP_SPEEDS + u32::from(next).min(SPEEDS - 1)
}

/// The text of a HUD button's name: showing the car's HUD when it is off, hiding it when on.
pub fn hud_text(on: bool) -> u32 {
    if on {
        id::VIEWER_TIP_HUD_HIDE
    } else {
        id::VIEWER_TIP_HUD_SHOW
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_stays_then_fades_then_goes() {
        assert_eq!(strength(0), Some(255));
        assert_eq!(strength(STAY_MS - 1), Some(255));
        let mid = strength(STAY_MS + FADE_MS / 2).unwrap();
        assert!((120..=135).contains(&mid), "{mid}");
        assert!(strength(STAY_MS + FADE_MS - 1).unwrap() < 5);
        assert_eq!(strength(STAY_MS + FADE_MS), None);
    }

    #[test]
    fn every_button_has_a_text_and_the_texts_are_own() {
        for button in 0..BUTTONS {
            let text = button_text(button).unwrap();
            assert!(crate::strings::own::is_own(text));
        }
        assert_eq!(button_text(BUTTONS), None);
    }

    #[test]
    fn a_fading_pixel_blends_or_carries_its_alpha() {
        assert_eq!(over(0x102030, 0x405060, 255), 0x102030);
        assert_eq!(over(0x102030, 0x405060, 0), 0x405060);
        assert_eq!(over(0x102030, UNWRITTEN, 100), 0x6410_2030);
        let half = over(0xFF_FFFF, 0, 128);
        assert!((0x80..=0x81).contains(&(half & 0xFF)));
    }

    #[test]
    fn the_speed_goes_round() {
        let after = |b| speed_text(b) - id::VIEWER_TIP_SPEEDS;
        assert_eq!((after(0), after(1), after(2)), (1, 2, 0));
    }
}
