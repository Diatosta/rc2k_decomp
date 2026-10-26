//! The replay's file state and index (IDA 0x5BEC78 .. 0x5BF518; phase 7, typed): the recording
//! switches, the replay file's open flag and handle, the viewer's state (mode, slot, stopped,
//! textures kept, stage and fonts flags), then the replay index read and written in place
//! ("RCIB", the seal, the block and frame counts, the (offset, size) per block). `#[repr(C)]` with
//! the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct ReplayFileBlock {
    /// +0x0 (0x5BEC78): `dword_5BEC78`: 1 = replays are recorded (`race::open_replay`, `race::replay`).
    pub replays_on: Dword,
    /// +0x4 (0x5BEC7C): `dword_5BEC7C`: the `noreplay` setting, 0 = replays on (`settings::read_settings`, `race::open_replay`).
    pub no_replay_switch: Dword,
    /// +0x8 (0x5BEC80): `dword_5BEC80`: 1 = the chunks are written unpacked (`race::replay`).
    pub chunks_plain: Dword,
    /// +0xc (0x5BEC84): `dword_5BEC84`: a replay was closed / is there (`race::replay`, `race::replay_mode`, `race::open_viewed_replay`).
    pub replay_closed: Dword,
    /// +0x10 (0x5BEC88): `g_replayOpen`: the replay file is open (`race::replay`, `race::viewer`, `drive::read_replay_frame`, `drive::write_replay_frames`).
    pub replay_open: Dword,
    /// +0x14 (0x5BEC8C): `h`: the replay file's handle (`race::replay`, `drive::read_replay_frame`).
    pub replay_handle: Dword,
    /// +0x18 (0x5BEC90): `g_replayAtEnd`: the last frame was reached (`drive::read_replay_frame`, `race::viewer`).
    pub replay_at_end: Dword,
    /// +0x1c (0x5BEC94): `g_holdCarRevs`: the cars' revs are held (`race::viewer`, `physics::off_line_warnings`, `physics::pace_notes_step`, `sound::car_engine_state`, `hud::draw_view_strips`, `race::credits_race`).
    pub hold_car_revs: Dword,
    /// +0x20 (0x5BEC98): `g_viewerStopped`: 1 = the viewer is not playing (`race::viewer`).
    pub viewer_stopped: Dword,
    /// +0x24 (0x5BEC9C): `g_viewerMode`: the viewer's mode, the eax it is started with (`race::viewer`).
    pub viewer_mode: Dword,
    /// +0x28 (0x5BECA0): `g_viewerSlot`: the replay slot being viewed, 222 the last race's (`race::viewer`).
    pub viewer_slot: Dword,
    /// +0x2c (0x5BECA4): `g_viewerSound`: the menu sound to play (`race::viewer`).
    pub viewer_sound: Dword,
    /// +0x30 (0x5BECA8): `g_viewerEditing`: the `replaykeys` setting, the editing keys work (`race::viewer`, `settings::read_settings`).
    pub viewer_editing: Dword,
    /// +0x34 (0x5BECAC): `unk_5BECAC`: the `seasoncode` setting (`settings::read_settings`, `race::replay`: the header's season code) (a reading).
    pub season_code: Dword,
    /// +0x38 (0x5BECB0): `dword_5BECB0`: kind 0x35's first texture kept (`race::viewer_palettes`).
    pub viewer_first_texture_a: Dword,
    /// +0x3c (0x5BECB4): `dword_5BECB4`: kind 0x35's first slot kept; the list's texture (`race::viewer_palettes`, `race::viewer_frame`).
    pub viewer_list_texture: Dword,
    /// +0x40 (0x5BECB8): `dword_5BECB8`: kind 0x2A's first texture kept; the panel's texture (`race::viewer_palettes`, `race::draw_viewer_panel`).
    pub viewer_panel_texture: Dword,
    /// +0x44 (0x5BECBC): `dword_5BECBC`: kind 0x2A's first slot kept (`race::viewer_palettes`, `race::draw_viewer_panel`).
    pub viewer_device_texture: Dword,
    /// +0x48 (0x5BECC0): `g_viewerStageLoaded`: the viewer's stage is loaded (`race::viewer_load_stage`, `race::viewer_tidy_stage`).
    pub viewer_stage_loaded: Dword,
    /// +0x4c (0x5BECC4): `g_viewerFontsBuilt`: the viewer built the fonts (`race::viewer_load_stage`).
    pub viewer_fonts_built: Dword,
    /// +0x50 (0x5BECC8): `aRcib`: "RCIB", the index's first dword; the index is this dword .. the places, 0x3850 bytes, read and written in place (`race::replay`, `race::open_viewed_replay`).
    pub index_magic: Dword,
    /// +0x54 (0x5BECCC): The dword after "RCIB" (inside IDA's string item): the race record's seal (`race::seal_stage_time`).
    pub index_seal: Dword,
    /// +0x58 (0x5BECD0): `dword_5BECD0`: blocks in the replay (`race::replay`, `drive::read_replay_frame`, `race::viewer`).
    pub index_block_count: Dword,
    /// +0x5c (0x5BECD4): `dword_5BECD4`: frames in the replay (`race::replay`, `drive::write_replay_frames`).
    pub index_frame_count: Dword,
    /// +0x60 (0x5BECD8): `lDistanceToMove` and `dword_5BECDC` (int[3599]): (offset, size) per block, 8 bytes a block, the rest of the index (`race::replay`, `drive::read_replay_frame`).
    pub block_places: [Dword; 3600],
}

/// The piece: the race block's bytes at IDA 0x5BEC78 seen as `ReplayFileBlock`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = ReplayFileBlock;
    fn deref(&self) -> &ReplayFileBlock {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const ReplayFileBlock) }
    }
}

pub static REPLAY_FILE: InRaceBlock = InRaceBlock;

/// IDA 0x5BEC78, the first byte of the piece.
pub const START: usize = 0x5BEC78;
/// IDA 0x5C2518, the end of the piece.
pub const END: usize = 0x5C2518;

const _: () = assert!(END - START == core::mem::size_of::<ReplayFileBlock>());

/// Where IDA 0x5BEC78 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(ReplayFileBlock, replays_on), 0x5BEC78 - START);
        assert_eq!(offset_of!(ReplayFileBlock, replay_at_end), 0x5BEC90 - START);
        assert_eq!(
            offset_of!(ReplayFileBlock, viewer_editing),
            0x5BECA8 - START
        );
        assert_eq!(
            offset_of!(ReplayFileBlock, viewer_stage_loaded),
            0x5BECC0 - START
        );
        assert_eq!(offset_of!(ReplayFileBlock, block_places), 0x5BECD8 - START);
    }
}
