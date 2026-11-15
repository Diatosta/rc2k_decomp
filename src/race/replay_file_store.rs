//! The replay's file state and index: the recording switches, the replay file's open flag and
//! handle, the viewer's state (mode, slot, stopped, textures kept, stage and fonts flags), then the
//! replay index read and written in place ("RCIB", the seal, the block and frame counts, the
//! (offset, size) per block). `#[repr(C)]` with a fixed layout.

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainDword, PlainFlag};

#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ReplayFileBlock {
    /// 1 = replays are recorded (`race::open_replay`, `race::replay`).
    pub replays_on: PlainDword,
    /// The `noreplay` setting, 0 = replays on (`settings::read_settings`,
    /// `race::open_replay`).
    pub no_replay_switch: PlainDword,
    /// 1 = the chunks are written unpacked (`race::replay`).
    pub chunks_plain: PlainDword,
    /// A replay was closed / is there (`race::replay`, `race::replay_mode`,
    /// `race::open_viewed_replay`).
    pub replay_closed: PlainDword,
    /// The replay file is open (`race::replay`, `race::viewer`, `drive::read_replay_frame`,
    /// `drive::write_replay_frames`).
    pub replay_open: PlainFlag,
    /// The replay file's handle (`race::replay`, `drive::read_replay_frame`).
    pub replay_handle: PlainDword,
    /// The last frame was reached (`drive::read_replay_frame`, `race::viewer`).
    pub replay_at_end: PlainFlag,
    /// The cars' revs are held (`race::viewer`, `physics::off_line_warnings`,
    /// `physics::pace_notes_step`, `sound::car_engine_state`, `hud::draw_view_strips`,
    /// `race::credits_race`).
    pub hold_car_revs: PlainDword,
    /// 1 = the viewer is not playing (`race::viewer`).
    pub viewer_stopped: PlainFlag,
    /// The viewer's mode, the mode it is started with (`race::viewer`).
    pub viewer_mode: PlainDword,
    /// The replay slot being viewed, 222 the last race's (`race::viewer`).
    pub viewer_slot: PlainDword,
    /// The menu sound to play (`race::viewer`).
    pub viewer_sound: PlainDword,
    /// The `replaykeys` setting, the editing keys work (`race::viewer`,
    /// `settings::read_settings`).
    pub viewer_editing: PlainDword,
    /// The `seasoncode` setting (`settings::read_settings`, `race::replay`: the header's
    /// season code).
    pub season_code: PlainDword,
    /// Kind 0x35's first texture kept (`race::viewer_palettes`).
    pub viewer_first_texture_a: PlainDword,
    /// Kind 0x35's first slot kept; the list's texture (`race::viewer_palettes`,
    /// `race::viewer_frame`).
    pub viewer_list_texture: PlainDword,
    /// Kind 0x2A's first texture kept; the panel's texture (`race::viewer_palettes`,
    /// `race::draw_viewer_panel`).
    pub viewer_panel_texture: PlainDword,
    /// Kind 0x2A's first slot kept (`race::viewer_palettes`, `race::draw_viewer_panel`).
    pub viewer_device_texture: PlainDword,
    /// The viewer's stage is loaded (`race::viewer_load_stage`, `race::viewer_tidy_stage`).
    pub viewer_stage_loaded: PlainDword,
    /// The viewer built the fonts (`race::viewer_load_stage`).
    pub viewer_fonts_built: PlainDword,
    /// The replay index, "RCIB" through the places, 0x3850 bytes, read and written in place
    /// (`race::replay`, `race::open_viewed_replay`).
    pub index: ReplayIndex,
}

/// The replay index: "RCIB", the seal, the counts, the places; the dwords of the file's first
/// 0x3850 bytes in order.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub struct ReplayIndex {
    /// "RCIB", the index's first dword.
    pub magic: PlainDword,
    /// The dword after "RCIB": the race record's seal
    /// (`race::seal_stage_time`).
    pub seal: PlainDword,
    /// Blocks in the replay (`race::replay`, `drive::read_replay_frame`, `race::viewer`).
    pub block_count: PlainDword,
    /// Frames in the replay (`race::replay`, `drive::write_replay_frames`).
    pub frame_count: PlainDword,
    /// (Offset, size) per block, 8 bytes a block, and the rest of the index (`race::replay`,
    /// `drive::read_replay_frame`).
    pub block_places: [PlainDword; 3600],
}

impl ReplayFileBlock {
    /// The piece, with its first values.
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
        crate::memory::cells::plain_dwords_in(self, 0)
    }
}

/// The index's dwords: "RCIB" through the places, 0x3850 bytes.
pub const INDEX_DWORDS: usize = 4 + 3600;
