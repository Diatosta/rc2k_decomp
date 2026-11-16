//! What the front-end pages copy into their work values before they are drawn: the audio
//! settings, the stage options' texts and the picked player's controller setting.

use crate::audio::modes::AudioOptions;
use crate::menu::player::Player;

/// The advanced audio page's six work values (`MenuOptions.audio_quality` ..
/// `MenuOptions.channels`: quality, DS3D, EAX, A3D, Dolby, channels) from the options as last
/// applied.
pub fn audio_page(options: &AudioOptions) -> [u32; 6] {
    let a = options.applied; // quality, channels, DS3D, EAX, A3D, Dolby
    [a[0], a[2], a[3], a[4], a[5], a[1]]
}

/// The stage options page's items 1 and 2: their strings by the two switches (1:
/// the first string).
pub fn stage_options_items(first: u32, second: u32) -> (u32, u32) {
    (
        if first == 1 { 0x19C } else { 0x19B },
        if second == 1 { 0x19E } else { 0x19D },
    )
}

/// The stage options page's caption: string 0x4FC + the page's number.
pub fn stage_options_caption(page: u32) -> u32 {
    0x4FC_u32.wrapping_add(page)
}

/// The stage pages' line (`<label> <n>/6: <track>`) may fill at most this much of its
/// buffer, its NUL with it.
pub const STAGE_LINE_ROOM: usize = 0x100;

/// The setting the controller pages copy from the player picked: its +0x2C and the 0x2C
/// bytes from +0x50.
pub const PICKED_LEN: usize = 0x2C;

impl Player {
    /// `pick_player_n`'s copy.
    pub fn picked(&self) -> (u32, [u8; PICKED_LEN]) {
        (self.controller, self.bindings.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio() {
        let o = AudioOptions {
            applied: [1, 2, 3, 4, 5, 6],
            ..AudioOptions::default()
        };
        assert_eq!(audio_page(&o), [1, 3, 4, 5, 6, 2]);
    }

    #[test]
    fn items() {
        assert_eq!(stage_options_items(1, 0), (0x19C, 0x19D));
        assert_eq!(stage_options_items(2, 1), (0x19B, 0x19E));
        assert_eq!(stage_options_caption(3), 0x4FF);
    }

    #[test]
    fn picked() {
        let mut bytes = [0u8; std::mem::size_of::<Player>()];
        for (n, b) in bytes.iter_mut().enumerate() {
            *b = n as u8;
        }
        let p: Player = unsafe { std::mem::transmute(bytes) };
        let (value, copy) = p.picked();
        assert_eq!(value, u32::from_le_bytes([0x2C, 0x2D, 0x2E, 0x2F]));
        assert_eq!(copy[0], 0x50);
        assert_eq!(copy[PICKED_LEN - 1], 0x7B);
    }
}
