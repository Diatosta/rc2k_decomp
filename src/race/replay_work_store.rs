//! The replay's work: the car header (stage, mode, players, car kinds, start place, seed, game-over
//! flags, co-driver sexes), the sealed header part (0x100 bytes, some fields unaligned), the patch
//! switches and the co-driver's door kept in the header, and the replay cameras' wishes and last
//! draw frames. The header fields are written to the replay file in the order they are declared here
//! (`race::replay`).

pub struct ReplayWork {
    /// Unused; nothing reads it.
    pub reserved_640: [u32; 6],
    /// The car header's stage setting, header offset 0 (`race::replay`, `race::race_end`,
    /// `race::viewer`).
    pub header_stage: u32,
    /// The header's mode, offset 4; the viewer copies it to the run kind
    /// (`race::viewer_load_stage`).
    pub header_mode: u32,
    /// The header's players, offset 8 (`race::race_end`).
    pub header_players: u32,
    /// The header's cars setting, offset 0xC (`race::race_end`).
    pub header_cars: u32,
    /// The header's car kind 0 (the first car's, limits 0x16 .. 0x21 checked), offset 0x10
    /// (`race::viewer`, `race::race_end`).
    pub header_kind_0: u32,
    /// The header's car kind 1, offset 0x14 (`race::viewer_load_stage`).
    pub header_kind_1: u32,
    /// The header's car kind 2, offset 0x18 (`race::race_end`).
    pub header_kind_2: u32,
    /// The header's car kind 3, offset 0x1C (`race::race_end`).
    pub header_kind_3: u32,
    /// One player (1) or two, set by `race::record_replay_header`; read by
    /// `drive::write_replay_frames`, the viewer (`race::viewer`).
    pub header_one_player: u32,
    /// The first car's co-driver sex (`race::replay`, `race::viewer_load_stage`).
    pub header_codriver_sex_0: u32,
    /// The second car's co-driver sex (`race::replay`, `race::viewer_load_stage`).
    pub header_codriver_sex_1: u32,
    /// The header's start place, offset 0x2C (`race::race_end`).
    pub header_start_place: u32,
    /// The header's arcade cars, offset 0x30 (`race::race_end`).
    pub header_arcade_cars: u32,
    /// The header's random seed, offset 0x34 (`race::race_end`).
    pub header_seed: u32,
    /// The header's game-over flag of car 0, offset 0x38 (`race::race_end`).
    pub header_game_over_0: u32,
    /// The header's game-over flag of car 1, offset 0x3C (`race::race_end`).
    pub header_game_over_1: u32,
    /// The replay's sealed header part, 0x100 bytes: checksum dword, version byte at offset 4 of the part,
    /// copied settings (some unaligned, `race::replay::copy_header_settings`), the player name at
    /// offset 0xD (0x20 bytes), sealed in blocks of 8 (`race::seal_replay_header`, `race::replay`).
    pub sealed_header: [u32; 64],
    /// Set to 6 when the header holds the patch switches (`race::replay_patches`,
    /// `race::replay`).
    pub patches_in_header: u32,
    /// The 13 patch switches copied into the header (0x34 bytes, `race::replay_patches`,
    /// `race::replay`).
    pub header_patches: [u32; 13],
    /// The low byte of the first dword is the first car's door (race) number the viewer
    /// reads (`race::viewer_load_stage`); the rest is read by no code.
    pub after_header: [u32; 162],
    /// The wished camera per head byte, indexed by a byte of the frame head
    /// (`drive::replay_cameras`).
    pub replay_camera_wishes: [u32; 47],
    /// The frame of the last camera draw for cars 0 and 1; cleared at stage load
    /// (`drive::replay_cameras`, `race::loadlevel`).
    pub camera_last_frame: [u32; 2],
    /// The same for cars 2 and 3 (a replay with the extra tape; `drive::replay_cameras`).
    pub camera_last_frame_extra: [u32; 2],
    /// The same for the cameras the viewer chose for the fifth to the eighth player's layouts
    /// (`drive::replay_cameras`).
    pub camera_last_frame_live: [u32; 4],
}

impl ReplayWork {
    /// The work with its first values.
    pub const fn new() -> Self {
        Self {
            reserved_640: [0x3C0, 0x1A10, 0x640, 0x400, 0x2810, 0x2810],
            replay_camera_wishes: [
                8, 0xA, 7, 9, 0, 1, 2, 3, 4, 5, 6, 0x2E, 0x2F, 0x30, 0x31, 0x23, 0x24, 0x25, 0x10,
                0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
                0x1F, 0x20, 0x21, 0x22, 0x32, 0x33, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D,
            ],
            header_stage: 0,
            header_mode: 0,
            header_players: 0,
            header_cars: 0,
            header_kind_0: 0,
            header_kind_1: 0,
            header_kind_2: 0,
            header_kind_3: 0,
            header_one_player: 0,
            header_codriver_sex_0: 0,
            header_codriver_sex_1: 0,
            header_start_place: 0,
            header_arcade_cars: 0,
            header_seed: 0,
            header_game_over_0: 0,
            header_game_over_1: 0,
            sealed_header: [0; 64],
            patches_in_header: 0,
            header_patches: [0; 13],
            after_header: [0; 162],
            camera_last_frame: [0; 2],
            camera_last_frame_extra: [0; 2],
            camera_last_frame_live: [0; 4],
        }
    }
}

impl ReplayWork {
    /// The dwords of the car header in the file's order: the sixteen settings, the sealed part,
    /// the patches flag and switches, the unread rest (256 dwords, 0x400 bytes).
    pub fn header_words(&self) -> Vec<u32> {
        let singles = [
            self.header_stage,
            self.header_mode,
            self.header_players,
            self.header_cars,
            self.header_kind_0,
            self.header_kind_1,
            self.header_kind_2,
            self.header_kind_3,
            self.header_one_player,
            self.header_codriver_sex_0,
            self.header_codriver_sex_1,
            self.header_start_place,
            self.header_arcade_cars,
            self.header_seed,
            self.header_game_over_0,
            self.header_game_over_1,
        ];
        singles
            .into_iter()
            .chain(self.sealed_header)
            .chain(std::iter::once(self.patches_in_header))
            .chain(self.header_patches)
            .chain(self.after_header)
            .collect()
    }

    /// Puts `words` into the car header's dwords in the file's order (as many as there are).
    pub fn set_header_words(&mut self, words: impl IntoIterator<Item = u32>) {
        let singles = [
            &mut self.header_stage,
            &mut self.header_mode,
            &mut self.header_players,
            &mut self.header_cars,
            &mut self.header_kind_0,
            &mut self.header_kind_1,
            &mut self.header_kind_2,
            &mut self.header_kind_3,
            &mut self.header_one_player,
            &mut self.header_codriver_sex_0,
            &mut self.header_codriver_sex_1,
            &mut self.header_start_place,
            &mut self.header_arcade_cars,
            &mut self.header_seed,
            &mut self.header_game_over_0,
            &mut self.header_game_over_1,
        ];
        let fields = singles
            .into_iter()
            .chain(&mut self.sealed_header)
            .chain(std::iter::once(&mut self.patches_in_header))
            .chain(&mut self.header_patches)
            .chain(&mut self.after_header);
        for (field, word) in fields.zip(words) {
            *field = word;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_words_go_in_and_come_out_in_order() {
        let mut work = ReplayWork::new();
        work.set_header_words(0..300);
        let words = work.header_words();
        assert_eq!(words.len(), 256);
        assert!(words.iter().copied().eq(0..256));
        assert_eq!(work.header_stage, 0);
        assert_eq!(work.sealed_header[0], 16);
        assert_eq!(work.patches_in_header, 80);
        assert_eq!(work.header_patches[0], 81);
        assert_eq!(work.after_header[0], 94);
        assert_eq!(work.after_header[161], 255);
    }
}
