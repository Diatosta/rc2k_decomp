//! The sound spots the frame's scene finds in the course's faces (`race::sound_spots`) and shares
//! with the sound side (`race::frame_draw::share_sound_spots`).

use rc2k_core::race::view::MOST_SPOTS;

/// A spot: the face's point number, the place as float bits (turned by the view's matrix and
/// scaled), and the sound.
#[derive(Clone, Copy, Default)]
pub struct SoundSpot {
    pub point: u32,
    pub place: [u32; 3],
    pub sound: u32,
}

/// The spots made this frame.
pub struct SoundSpots {
    spots: [SoundSpot; MOST_SPOTS as usize],
    count: u32,
}

impl SoundSpots {
    pub const fn new() -> Self {
        Self {
            spots: [SoundSpot {
                point: 0,
                place: [0; 3],
                sound: 0,
            }; MOST_SPOTS as usize],
            count: 0,
        }
    }

    /// The spots made this frame.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Starts the frame's list afresh (the spots stay, as a later list overwrites them).
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// The spots of the table, made or not (the count says which are made).
    pub fn spots(&self) -> &[SoundSpot] {
        &self.spots
    }

    /// Adds a spot at the count's place; `None` when the table is full.
    pub fn push(&mut self, spot: SoundSpot) -> Option<()> {
        *self.spots.get_mut(self.count as usize)? = spot;
        self.count += 1;
        Some(())
    }
}
