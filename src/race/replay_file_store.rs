//! The replay's file state and index (IDA 0x5BEC78 .. 0x5BF518; phase 7, typed): the recording
//! switches, the replay file's open flag and handle, the viewer's state (mode, slot, stopped,
//! textures kept, stage and fonts flags), then the replay index read and written in place
//! ("RCIB", the seal, the block and frame counts, the (offset, size) per block). `#[repr(C)]` with
//! the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::cells::ZeroCells;

use crate::session::{PlainDword, PlainFlag};

#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ReplayFileBlock {
    /// +0x0 (0x5BEC78): `dword_5BEC78`: 1 = replays are recorded (`race::open_replay`, `race::replay`).
    pub replays_on: PlainDword,
    /// +0x4 (0x5BEC7C): `dword_5BEC7C`: the `noreplay` setting, 0 = replays on (`settings::read_settings`, `race::open_replay`).
    pub no_replay_switch: PlainDword,
    /// +0x8 (0x5BEC80): `dword_5BEC80`: 1 = the chunks are written unpacked (`race::replay`).
    pub chunks_plain: PlainDword,
    /// +0xc (0x5BEC84): `dword_5BEC84`: a replay was closed / is there (`race::replay`, `race::replay_mode`, `race::open_viewed_replay`).
    pub replay_closed: PlainDword,
    /// +0x10 (0x5BEC88): `g_replayOpen`: the replay file is open (`race::replay`, `race::viewer`, `drive::read_replay_frame`, `drive::write_replay_frames`).
    pub replay_open: PlainFlag,
    /// +0x14 (0x5BEC8C): `h`: the replay file's handle (`race::replay`, `drive::read_replay_frame`).
    pub replay_handle: PlainDword,
    /// +0x18 (0x5BEC90): `g_replayAtEnd`: the last frame was reached (`drive::read_replay_frame`, `race::viewer`).
    pub replay_at_end: PlainFlag,
    /// +0x1c (0x5BEC94): `g_holdCarRevs`: the cars' revs are held (`race::viewer`, `physics::off_line_warnings`, `physics::pace_notes_step`, `sound::car_engine_state`, `hud::draw_view_strips`, `race::credits_race`).
    pub hold_car_revs: PlainDword,
    /// +0x20 (0x5BEC98): `g_viewerStopped`: 1 = the viewer is not playing (`race::viewer`).
    pub viewer_stopped: PlainFlag,
    /// +0x24 (0x5BEC9C): `g_viewerMode`: the viewer's mode, the mode it is started with (`race::viewer`).
    pub viewer_mode: PlainDword,
    /// +0x28 (0x5BECA0): `g_viewerSlot`: the replay slot being viewed, 222 the last race's (`race::viewer`).
    pub viewer_slot: PlainDword,
    /// +0x2c (0x5BECA4): `g_viewerSound`: the menu sound to play (`race::viewer`).
    pub viewer_sound: PlainDword,
    /// +0x30 (0x5BECA8): `g_viewerEditing`: the `replaykeys` setting, the editing keys work (`race::viewer`, `settings::read_settings`).
    pub viewer_editing: PlainDword,
    /// +0x34 (0x5BECAC): `unk_5BECAC`: the `seasoncode` setting (`settings::read_settings`, `race::replay`: the header's season code).
    pub season_code: PlainDword,
    /// +0x38 (0x5BECB0): `dword_5BECB0`: kind 0x35's first texture kept (`race::viewer_palettes`).
    pub viewer_first_texture_a: PlainDword,
    /// +0x3c (0x5BECB4): `dword_5BECB4`: kind 0x35's first slot kept; the list's texture (`race::viewer_palettes`, `race::viewer_frame`).
    pub viewer_list_texture: PlainDword,
    /// +0x40 (0x5BECB8): `dword_5BECB8`: kind 0x2A's first texture kept; the panel's texture (`race::viewer_palettes`, `race::draw_viewer_panel`).
    pub viewer_panel_texture: PlainDword,
    /// +0x44 (0x5BECBC): `dword_5BECBC`: kind 0x2A's first slot kept (`race::viewer_palettes`, `race::draw_viewer_panel`).
    pub viewer_device_texture: PlainDword,
    /// +0x48 (0x5BECC0): `g_viewerStageLoaded`: the viewer's stage is loaded (`race::viewer_load_stage`, `race::viewer_tidy_stage`).
    pub viewer_stage_loaded: PlainDword,
    /// +0x4c (0x5BECC4): `g_viewerFontsBuilt`: the viewer built the fonts (`race::viewer_load_stage`).
    pub viewer_fonts_built: PlainDword,
    /// +0x50 (0x5BECC8): the replay index, "RCIB" .. the places, 0x3850 bytes, read and written in place (`race::replay`, `race::open_viewed_replay`).
    pub index: ReplayIndex,
}

/// The replay index: "RCIB", the seal, the counts, the places; the dwords of the file's first
/// 0x3850 bytes in order.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub struct ReplayIndex {
    /// +0x50 (0x5BECC8): `aRcib`: "RCIB", the index's first dword.
    pub magic: PlainDword,
    /// +0x54 (0x5BECCC): The dword after "RCIB" (inside IDA's string item): the race record's seal (`race::seal_stage_time`).
    pub seal: PlainDword,
    /// +0x58 (0x5BECD0): `dword_5BECD0`: blocks in the replay (`race::replay`, `drive::read_replay_frame`, `race::viewer`).
    pub block_count: PlainDword,
    /// +0x5c (0x5BECD4): `dword_5BECD4`: frames in the replay (`race::replay`, `drive::write_replay_frames`).
    pub frame_count: PlainDword,
    /// +0x60 (0x5BECD8): `lDistanceToMove` and `dword_5BECDC` (int[3599]): (offset, size) per block, 8 bytes a block, the rest of the index (`race::replay`, `drive::read_replay_frame`).
    pub block_places: [PlainDword; 3600],
}

impl ReplayFileBlock {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        Self {
            replays_on: PlainDword::new(1),
            viewer_slot: PlainDword::new(0xFFFFFFFF),
            viewer_panel_texture: PlainDword::new(1),
            viewer_device_texture: PlainDword::new(1),
            index: ReplayIndex {
                magic: PlainDword::new(0x42494352),
                ..ZeroCells::ZERO
            },
            ..ZeroCells::ZERO
        }
    }
}

impl ReplayIndex {
    /// The index ("RCIB", the seal, the counts, the places) as its dwords in order, read and
    /// written in place.
    pub fn dwords(&self) -> &[PlainDword; INDEX_DWORDS] {
        crate::cells::plain_dwords_in(self, 0)
    }
}

/// The index's dwords: "RCIB" .. the places, 0x3850 bytes.
pub const INDEX_DWORDS: usize = 4 + 3600;
